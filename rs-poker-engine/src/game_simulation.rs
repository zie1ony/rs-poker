use std::collections::BTreeMap;

use rs_poker::arena::GameState;
use rs_poker_types::game::{Decision, GameId, GameSettings, PossibleAction};
use rs_poker_types::game_event::{
    Award, FailedPlayerActionEvent, ForcedBetEvent, GameEndedEvent, GameEvent, GameStartedEvent,
    PlayerActionEvent, ShowCommunityCardsEvent,
};
use rs_poker_types::player::{Player, PlayerName};
use rs_poker_types::tournament::{self, TournamentId};
use tracing::{Level, debug_span, event, instrument, trace_span};

use rs_poker::arena::action::{FailedActionPayload, PlayedActionPayload};
use rs_poker::arena::game_state::Round;
use rs_poker::core::{Card, Rank, Rankable};

use rs_poker::arena::action::{
    Action, AgentAction, AwardPayload, DealStartingHandPayload, ForcedBetPayload, ForcedBetType,
    GameStartPayload, PlayerSitPayload,
};

#[derive(Debug, Clone, PartialEq)]
pub enum GameActionRequired {
    PlayerToAct {
        idx: usize,
        possible_actions: Vec<PossibleAction>,
    },
    NoActionRequired,
}

impl GameActionRequired {
    pub fn no_action_needed(&self) -> bool {
        matches!(self, GameActionRequired::NoActionRequired)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GameSimulation {
    pub game_id: GameId,
    pub game_state: GameState,
    pub actions: Vec<Action>,
    pub events: Vec<GameEvent>,
    pub hands: Vec<[Card; 2]>,
    pub community_cards: [Card; 5],
    pub player_names: Vec<PlayerName>,
}

impl GameSimulation {
    pub fn new(config: GameSettings) -> Self {
        let game_state = GameState::new_starting(
            config.stacks.clone(),
            config.big_blind(),
            config.small_blind,
            0.0,
            config.dealer_index,
        );

        let game_id = config
            .game_id
            .clone()
            .expect("game_id must be set at this point");
        let hands = config
            .hands
            .clone()
            .expect("hands must be set at this point");
        let community_cards = config
            .community_cards
            .clone()
            .expect("community_cards must be set at this point");

        // Emit a game started event
        let game_start_event = GameEvent::GameStarted(GameStartedEvent {
            game_id: game_id.clone(),
            settings: config.clone(),
        });

        GameSimulation {
            game_id,
            game_state,
            actions: Vec::new(),
            events: vec![game_start_event],
            hands,
            community_cards,
            player_names: config.player_names(),
        }
    }

    pub fn more_rounds(&self) -> bool {
        !matches!(self.game_state.round, Round::Complete)
    }

    /// Run the simulation all the way to completion. This will mutate the
    /// current state.
    pub fn run(&mut self) -> GameActionRequired {
        let span = debug_span!("run",
            game_state = ?self.game_state);
        let _enter = span.enter();

        while self.more_rounds() {
            let result = self.run_round();
            if !result.no_action_needed() {
                return result;
            }
        }
        GameActionRequired::NoActionRequired
    }

    pub fn run_round(&mut self) -> GameActionRequired {
        let span = trace_span!("run_round");
        let _enter = span.enter();

        match self.game_state.round {
            // Dealing the user hand is dealt with as its own round
            // in order to use the per round active bit set
            // for iterating players
            Round::Starting => self.start(),
            Round::Ante => self.ante(),

            Round::DealPreflop => self.deal_preflop(),
            Round::Preflop => self.preflop(),

            Round::DealFlop => self.deal_flop(),
            Round::Flop => self.flop(),

            Round::DealTurn => self.deal_turn(),
            Round::Turn => self.turn(),

            Round::DealRiver => self.deal_river(),
            Round::River => self.river(),

            Round::Showdown => self.showdown(),

            // There's nothing left to do to this.
            Round::Complete => GameActionRequired::NoActionRequired,
        }
    }

    fn start(&mut self) -> GameActionRequired {
        let span = trace_span!("start");
        let _enter = span.enter();

        // Add an action to record the ante, sb and bb
        // This should allow recreating starting game state
        // together with PlayerSit actions.
        self.record_action(Action::GameStart(GameStartPayload {
            ante: self.game_state.ante,
            small_blind: self.game_state.small_blind,
            big_blind: self.game_state.big_blind,
        }));

        while self.game_state.current_round_num_active_players() > 0 {
            let idx = self.game_state.to_act_idx();

            // Add an action that records starting stack for each player
            // Starting with to the left of the dealer
            // and ending with dealer button.
            self.record_action(Action::PlayerSit(PlayerSitPayload {
                player_stack: self.game_state.stacks[idx],
                idx,
            }));

            // set the active bit on the player to false.
            // This allows us to not deal to players that
            // are sitting out, while also going in the same
            // order of dealing
            self.game_state.round_data.needs_action.disable(idx);

            self.game_state.round_data.advance_action();
        }

        // We're done with the non-betting dealing only round
        self.advance_round();
        GameActionRequired::NoActionRequired
    }

    fn ante(&mut self) -> GameActionRequired {
        let span = trace_span!("ante");
        let _enter = span.enter();

        let ante = self.game_state.ante;
        if ante > 0.0 {
            // Force the ante from each active player.
            while self.game_state.current_round_num_active_players() > 0 {
                let idx = self.game_state.to_act_idx();

                self.game_state.do_bet(ante, true).unwrap();
                self.record_action(Action::ForcedBet(ForcedBetPayload {
                    bet: ante,
                    idx,
                    player_stack: self.game_state.stacks[idx],
                    forced_bet_type: ForcedBetType::Ante,
                }));

                self.game_state.round_data.needs_action.disable(idx);
            }
        }
        self.advance_round();
        GameActionRequired::NoActionRequired
    }

    fn deal_preflop(&mut self) -> GameActionRequired {
        let span = trace_span!("deal_preflop");
        let _enter = span.enter();

        // Check if we need to deal cards to any player
        while self.game_state.current_round_num_active_players() > 0 {
            let idx = self.game_state.to_act_idx();
            let cards = self.hands[idx].to_vec();
            self.deal_player_cards(cards);
            // Move to next player or advance round if all players have been dealt
            self.game_state.round_data.needs_action.disable(idx);
            self.game_state.round_data.advance_action();
        }

        self.advance_round();
        GameActionRequired::NoActionRequired
    }

    fn preflop(&mut self) -> GameActionRequired {
        let span = trace_span!("preflop");
        let _enter = span.enter();

        // Force the small blind and the big blind.
        if !self.game_state.sb_posted {
            let sb = self.game_state.small_blind;
            let sb_idx = self.game_state.to_act_idx();
            self.game_state.do_bet(sb, true).unwrap();
            self.game_state.sb_posted = true;

            self.record_action(Action::ForcedBet(ForcedBetPayload {
                bet: sb,
                idx: sb_idx,
                forced_bet_type: ForcedBetType::SmallBlind,
                player_stack: self.game_state.stacks[sb_idx],
            }));
            self.record_event(GameEvent::ForcedBet(ForcedBetEvent {
                player_name: self.player_name(sb_idx),
                player_idx: sb_idx,
                bet: sb,
                stack_after: self.game_state.stacks[sb_idx],
                pot_after: self.game_state.total_pot,
                bet_kind: rs_poker_types::game_event::ForcedBetKind::SmallBlind,
            }));
        }

        if !self.game_state.bb_posted {
            let bb = self.game_state.big_blind;
            let bb_idx = self.game_state.to_act_idx();
            self.game_state.do_bet(bb, true).unwrap();
            self.game_state.bb_posted = true;
            self.record_action(Action::ForcedBet(ForcedBetPayload {
                bet: bb,
                idx: bb_idx,
                forced_bet_type: ForcedBetType::BigBlind,
                player_stack: self.game_state.stacks[bb_idx],
            }));
            self.record_event(GameEvent::ForcedBet(ForcedBetEvent {
                player_name: self.player_name(bb_idx),
                player_idx: bb_idx,
                bet: bb,
                stack_after: self.game_state.stacks[bb_idx],
                pot_after: self.game_state.total_pot,
                bet_kind: rs_poker_types::game_event::ForcedBetKind::BigBlind,
            }));
        }

        let result = self.run_betting_round();
        if !result.no_action_needed() {
            return result;
        }
        self.advance_round();
        GameActionRequired::NoActionRequired
    }

    fn deal_flop(&mut self) -> GameActionRequired {
        let span = trace_span!("deal_flop");
        let _enter = span.enter();
        self.deal_community_cards(Round::Flop, self.community_cards[0..3].to_vec());
        self.advance_round();
        GameActionRequired::NoActionRequired
    }

    fn flop(&mut self) -> GameActionRequired {
        let span = trace_span!("flop");
        let _enter = span.enter();

        let result = self.run_betting_round();
        if !result.no_action_needed() {
            return result;
        }
        self.advance_round();
        GameActionRequired::NoActionRequired
    }

    fn deal_turn(&mut self) -> GameActionRequired {
        let span = trace_span!("turn");
        let _enter = span.enter();

        self.deal_community_cards(Round::Turn, self.community_cards[3..4].to_vec());
        self.advance_round();
        GameActionRequired::NoActionRequired
    }

    fn turn(&mut self) -> GameActionRequired {
        let span = trace_span!("turn");
        let _enter = span.enter();

        let result = self.run_betting_round();
        if !result.no_action_needed() {
            return result;
        }
        self.advance_round();
        GameActionRequired::NoActionRequired
    }

    fn deal_river(&mut self) -> GameActionRequired {
        let span = trace_span!("river");
        let _enter = span.enter();

        self.deal_community_cards(Round::River, self.community_cards[4..5].to_vec());
        self.advance_round();
        GameActionRequired::NoActionRequired
    }

    fn river(&mut self) -> GameActionRequired {
        let span = trace_span!("river");
        let _enter = span.enter();

        let result = self.run_betting_round();
        if !result.no_action_needed() {
            return result;
        }
        self.advance_round();
        GameActionRequired::NoActionRequired
    }

    fn showdown(&mut self) -> GameActionRequired {
        let span = trace_span!("showdown");
        let _enter = span.enter();

        // Rank each player that still has a chance.
        let active = self.game_state.player_active | self.game_state.player_all_in;

        let mut bets = self.game_state.player_bet.clone();

        // Create a map where the keys are the ranks of hands and
        // the values are vectors of player index, for players that had that hand
        let ranks = active
            .ones()
            .map(|idx| (idx, self.game_state.hands[idx].rank()))
            .fold(
                BTreeMap::new(),
                |mut map: BTreeMap<Rank, Vec<usize>>, (idx, rank)| {
                    map.entry(rank)
                        .and_modify(|m| {
                            m.push(idx);
                            m.sort_by(|a, b| bets[*a].partial_cmp(&bets[*b]).unwrap());
                        })
                        .or_insert_with(|| vec![idx]);

                    map
                },
            );
        // There can be bets that players made but didn't take to showdown they should
        // be added to the main pot. Keep them here and then split them up
        // between the winners of the first rank pot. resetting the ammount to
        // zero.
        let mut folded_pot = bets
            .iter()
            .enumerate()
            .filter(|(idx, _)| !active.get(*idx))
            .map(|(_, bet)| *bet)
            .sum::<f32>();
        bets = bets
            .iter()
            .enumerate()
            .map(|(idx, v)| if active.get(idx) { *v } else { 0.0 })
            .collect();

        let mut awards = Vec::new();

        // By default the map gives keys in assending order. We want them descending.
        // The actual player vector is sorted in ascending order according to bet size.

        for (rank, players) in ranks.into_iter().rev() {
            let mut start_idx = 0;
            let end_idx = players.len();

            // We'll conitune until every player has been given the matching money
            // up to their wager. However since some players might have gone allin
            // earlier we keep removing from the pot and splitting it equally to all
            // those players still left in the pot.
            while start_idx < end_idx {
                // Becasue our lists are ordered from smallest bets to largest
                // we can just assume the first one is the smallest
                //
                // Here we use that property to find the max bet that this pot
                // will give for this round of splitting ties.
                let max_wager = bets[players[start_idx]];
                let mut pot: f64 = f64::from(folded_pot);
                folded_pot = 0.0;

                // Most common is that ties will
                // be for wagers that are all the same.
                // So check if there's no more
                // bets to award for this player.
                if max_wager <= 0.0 {
                    start_idx += 1;
                    continue;
                }

                // Take all the wagers remaining into a
                // side pot. However this side pot might
                // be the only pot if there were no allins
                for b in bets.iter_mut() {
                    let w: f32 = (*b).min(max_wager);
                    *b -= w;
                    pot += w as f64;
                }

                // Now all the winning players get
                // an equal share of the side pot
                let num_players = (end_idx - start_idx) as f64;
                let split = pot / num_players;

                for idx in &players[start_idx..end_idx] {
                    // Record that this player won something
                    event!(parent: &span, Level::INFO, idx, split, pot, ?rank, "pot_awarded");
                    self.game_state.award(*idx, split as f32);
                    self.record_action(Action::Award(AwardPayload {
                        idx: *idx,
                        total_pot: pot as f32,
                        award_amount: split as f32,
                        // Since we had a showdown we cen copy the hand
                        // and the resulting rank.
                        rank: Some(rank),
                        hand: Some(self.game_state.hands[*idx]),
                    }));
                    awards.push(Award {
                        player_idx: *idx,
                        player_name: self.player_name(*idx),
                        won_pot: split as f32,
                        stack_after: self.game_state.stacks[*idx],
                        rank: Some(rank),
                        hand: Some(self.game_state.hands[*idx]),
                    });
                }

                // Since the first player is bet size
                // that we used. They have won everything that they're eligible for.
                start_idx += 1;
            }
        }

        self.record_event(GameEvent::GameEnded(GameEndedEvent {
            final_round: self.game_state.round,
            awards,
        }));

        self.end_game();
        GameActionRequired::NoActionRequired
    }

    fn deal_player_cards(&mut self, cards: Vec<Card>) {
        for c in &cards {
            self.record_action(Action::DealStartingHand(DealStartingHandPayload {
                card: *c,
                idx: self.game_state.to_act_idx(),
            }));
        }

        let idx = self.game_state.to_act_idx();
        self.game_state.hands[idx].extend(cards);
    }

    fn deal_community_cards(&mut self, round: Round, mut community_cards: Vec<Card>) {
        for c in &community_cards {
            self.record_action(Action::DealCommunity(*c));
        }
        self.record_event(GameEvent::ShowCommunityCards(ShowCommunityCardsEvent {
            round,
            cards: community_cards.clone(),
        }));
        // Add all the cards to the hands as well.
        for h in &mut self.game_state.hands {
            h.extend(community_cards.to_owned());
        }
        // Drain the community_cards vec into the game_state board.
        self.game_state.board.append(&mut community_cards);
    }

    /// This runs betting for the round to completion. It will run until
    /// everyone has acted or until the round has been completed because no one
    /// can act anymore.
    fn run_betting_round(&mut self) -> GameActionRequired {
        let current_round = self.game_state.round;
        while self.needs_action() && self.game_state.round == current_round {
            let result = self.run_single_agent();
            if !result.no_action_needed() {
                return result;
            }
        }
        GameActionRequired::NoActionRequired
    }

    fn needs_action(&self) -> bool {
        let active_players = self.game_state.player_active;

        let players_needing_action = self.game_state.round_data.needs_action;

        let active_players_needing_action = active_players & players_needing_action;
        !active_players_needing_action.empty()
    }

    /// Run the next agent in the game state to act.
    fn run_single_agent(&mut self) -> GameActionRequired {
        let idx = self.game_state.to_act_idx();
        let span = trace_span!("run_agent", idx);
        let _enter = span.enter();

        // Instead of asking the agent directly, return that we need player action
        // Generate possible actions for this player
        let possible_actions = self.get_possible_actions_for_current_player();

        GameActionRequired::PlayerToAct {
            idx,
            possible_actions,
        }
    }

    /// Generate possible actions for a player based on current game state
    pub fn get_possible_actions_for_current_player(&self) -> Vec<PossibleAction> {
        let mut actions = Vec::new();

        let to_call = self.game_state.current_round_bet()
            - self.game_state.current_round_current_player_bet();

        // Can fold if there's money to call
        if to_call > 0.0 {
            actions.push(PossibleAction::Fold);
        }

        // Can always call/check
        actions.push(PossibleAction::Call);

        // Can bet/raise - provide the range of valid bet amounts
        let current_bet = self.game_state.current_round_bet();
        let min_raise = self.game_state.current_round_min_raise();
        let min_bet = current_bet + min_raise;
        let player_stack = self.game_state.current_player_stack();
        let current_player_bet = self.game_state.current_round_current_player_bet();
        let max_bet = current_player_bet + player_stack;

        if min_bet <= max_bet {
            actions.push(PossibleAction::Bet {
                min: min_bet,
                max: max_bet,
            });
        }

        // Can go all-in if we have more money than the current bet
        if max_bet > current_bet {
            actions.push(PossibleAction::AllIn);
        }

        actions
    }

    /// Given the action that an agent wants to take, this function will
    /// determine if the action is valid and then apply it to the game state.
    /// If the action is invalid, the agent will be forced to fold.
    pub fn run_agent_action(&mut self, decision: Decision) {
        event!(Level::TRACE, ?decision, "run_agent_action");

        let agent_action = decision.action.clone();
        let idx = self.game_state.to_act_idx();
        let starting_bet = self.game_state.current_round_bet();
        let starting_player_bet = self.game_state.current_round_player_bet(idx);
        let starting_min_raise = self.game_state.current_round_min_raise();
        let starting_pot = self.game_state.total_pot;

        match agent_action {
            AgentAction::Fold => {
                // It plays hell on verifying games if there are players that quit when there's
                // no money in being asked for. For now do exact equality, but
                // this might be a place we should use approxiate compaison
                // crate.
                if starting_player_bet == starting_bet {
                    event!(Level::WARN, "fold_error");

                    let new_action = AgentAction::Bet(starting_bet);

                    self.game_state.do_bet(starting_bet, false).unwrap();
                    self.record_action(Action::FailedAction(FailedActionPayload {
                        action: agent_action,
                        result: PlayedActionPayload {
                            action: new_action.clone(),
                            player_stack: self.game_state.stacks[idx],
                            idx,
                            round: self.game_state.round,
                            starting_bet,
                            final_bet: starting_bet,
                            starting_min_raise,
                            final_min_raise: self.game_state.current_round_min_raise(),
                            starting_player_bet,
                            final_player_bet: starting_player_bet,
                            players_active: self.game_state.player_active,
                            players_all_in: self.game_state.player_all_in,
                            starting_pot,
                            final_pot: self.game_state.total_pot,
                        },
                    }));
                    self.record_event(GameEvent::FailedPlayerAction(FailedPlayerActionEvent {
                        player_name: self.player_name(idx),
                        player_idx: idx,
                        player_decision: decision,
                        action: new_action,
                        stack_after: self.game_state.stacks[idx],
                        pot_after: self.game_state.total_pot,
                    }));
                } else {
                    self.record_action(Action::PlayedAction(PlayedActionPayload {
                        action: agent_action,
                        player_stack: self.game_state.stacks[idx],
                        idx,
                        round: self.game_state.round,
                        // None of the bets move if this is a fold
                        starting_bet,
                        final_bet: starting_bet,
                        starting_min_raise,
                        final_min_raise: self.game_state.current_round_min_raise(),
                        starting_player_bet,
                        final_player_bet: starting_player_bet,
                        players_active: self.game_state.player_active,
                        players_all_in: self.game_state.player_all_in,
                        starting_pot,
                        final_pot: self.game_state.total_pot,
                    }));
                    self.record_event(GameEvent::PlayerAction(PlayerActionEvent {
                        player_name: self.player_name(idx),
                        player_idx: idx,
                        player_decision: decision,
                        stack_after: self.game_state.stacks[idx],
                        pot_after: self.game_state.total_pot,
                    }));

                    self.player_fold();
                }
            }
            AgentAction::Bet(bet_amount) => {
                let bet_result = self.game_state.do_bet(bet_amount, false);

                match bet_result {
                    Err(error) => {
                        // If the agent failed to give us a good bet then they lose this round.
                        //
                        // We emit the error as an event
                        // Assume that game_state.do_bet() will have changed nothing since it
                        // errored out Add an action that shows the user was
                        // force folded. Actually fold the user
                        event!(Level::WARN, ?error, "bet_error");

                        // Record this errant action
                        self.record_action(Action::FailedAction(FailedActionPayload {
                            action: agent_action,
                            result: PlayedActionPayload {
                                action: AgentAction::Fold,
                                player_stack: self.game_state.stacks[idx],
                                idx,
                                round: self.game_state.round,
                                starting_bet,
                                final_bet: starting_bet,
                                starting_min_raise,
                                final_min_raise: self.game_state.current_round_min_raise(),
                                starting_player_bet,
                                final_player_bet: starting_player_bet,
                                players_active: self.game_state.player_active,
                                players_all_in: self.game_state.player_all_in,
                                // What's the pot worth
                                starting_pot,
                                final_pot: self.game_state.total_pot,
                            },
                        }));

                        self.record_event(GameEvent::FailedPlayerAction(FailedPlayerActionEvent {
                            player_name: self.player_name(idx),
                            player_idx: idx,
                            player_decision: decision,
                            action: AgentAction::Fold,
                            stack_after: self.game_state.stacks[idx],
                            pot_after: self.game_state.total_pot,
                        }));

                        // Actually fold the user
                        self.player_fold();
                    }
                    Ok(_added) => {
                        let player_bet = self.game_state.current_round_player_bet(idx);

                        let new_action = match agent_action {
                            AgentAction::Bet(_) => AgentAction::Bet(player_bet),
                            AgentAction::Call => AgentAction::Call,
                            AgentAction::Fold => AgentAction::Fold,
                            AgentAction::AllIn => AgentAction::AllIn,
                        };
                        // If the game_state.do_bet function returned Ok then
                        // the state is already changed so record the action as played.
                        self.record_action(Action::PlayedAction(PlayedActionPayload {
                            action: new_action,
                            player_stack: self.game_state.stacks[idx],
                            idx,
                            round: self.game_state.round,
                            starting_bet,
                            final_bet: self.game_state.current_round_bet(),
                            starting_min_raise,
                            final_min_raise: self.game_state.current_round_min_raise(),
                            starting_player_bet,
                            final_player_bet: player_bet,
                            // Keep track of who's in a
                            players_active: self.game_state.player_active,
                            players_all_in: self.game_state.player_all_in,

                            // What's the pot worth
                            starting_pot,
                            final_pot: self.game_state.total_pot,
                        }));

                        self.record_event(GameEvent::PlayerAction(PlayerActionEvent {
                            player_name: self.player_name(idx),
                            player_idx: idx,
                            player_decision: decision,
                            stack_after: self.game_state.stacks[idx],
                            pot_after: self.game_state.total_pot,
                        }));
                    }
                }
            }
            AgentAction::Call => {
                let call_amount = self.game_state.current_round_bet();
                let bet_result = self.game_state.do_bet(call_amount, false);

                match bet_result {
                    Err(error) => {
                        // If the agent failed to give us a good bet then they lose this round.
                        //
                        // We emit the error as an event
                        // Assume that game_state.do_bet() will have changed nothing since it
                        // errored out Add an action that shows the user was
                        // force folded. Actually fold the user
                        event!(Level::WARN, ?error, "bet_error");

                        // Record this errant action
                        self.record_action(Action::FailedAction(FailedActionPayload {
                            action: agent_action,
                            result: PlayedActionPayload {
                                action: AgentAction::Fold,
                                player_stack: self.game_state.stacks[idx],
                                idx,
                                round: self.game_state.round,
                                starting_bet,
                                final_bet: starting_bet,
                                starting_min_raise,
                                final_min_raise: self.game_state.current_round_min_raise(),
                                starting_player_bet,
                                final_player_bet: starting_player_bet,
                                players_active: self.game_state.player_active,
                                players_all_in: self.game_state.player_all_in,
                                // What's the pot worth
                                starting_pot,
                                final_pot: self.game_state.total_pot,
                            },
                        }));

                        self.record_event(GameEvent::FailedPlayerAction(FailedPlayerActionEvent {
                            player_name: self.player_name(idx),
                            player_idx: idx,
                            player_decision: decision,
                            action: AgentAction::Fold,
                            stack_after: self.game_state.stacks[idx],
                            pot_after: self.game_state.total_pot,
                        }));

                        // Actually fold the user
                        self.player_fold();
                    }
                    Ok(_added) => {
                        let player_bet = self.game_state.current_round_player_bet(idx);

                        let new_action = match agent_action {
                            AgentAction::Call => AgentAction::Call,
                            AgentAction::Bet(_) => AgentAction::Bet(player_bet),
                            AgentAction::Fold => AgentAction::Fold,
                            AgentAction::AllIn => AgentAction::AllIn,
                        };
                        // If the game_state.do_bet function returned Ok then
                        // the state is already changed so record the action as played.
                        self.record_action(Action::PlayedAction(PlayedActionPayload {
                            action: new_action,
                            player_stack: self.game_state.stacks[idx],
                            idx,
                            round: self.game_state.round,
                            starting_bet,
                            final_bet: self.game_state.current_round_bet(),
                            starting_min_raise,
                            final_min_raise: self.game_state.current_round_min_raise(),
                            starting_player_bet,
                            final_player_bet: player_bet,
                            // Keep track of who's in a
                            players_active: self.game_state.player_active,
                            players_all_in: self.game_state.player_all_in,
                            // What's the pot worth
                            starting_pot,
                            final_pot: self.game_state.total_pot,
                        }));

                        self.record_event(GameEvent::PlayerAction(PlayerActionEvent {
                            player_name: self.player_name(idx),
                            player_idx: idx,
                            player_decision: decision,
                            stack_after: self.game_state.stacks[idx],
                            pot_after: self.game_state.total_pot,
                        }));
                    }
                }
            }
            AgentAction::AllIn => {
                let all_in_amount = self.game_state.current_round_current_player_bet()
                    + self.game_state.current_player_stack();
                let bet_result = self.game_state.do_bet(all_in_amount, false);

                match bet_result {
                    Err(error) => {
                        // If the agent failed to give us a good bet then they lose this round.
                        //
                        // We emit the error as an event
                        // Assume that game_state.do_bet() will have changed nothing since it
                        // errored out Add an action that shows the user was
                        // force folded. Actually fold the user
                        event!(Level::WARN, ?error, "bet_error");

                        // Record this errant action
                        self.record_action(Action::FailedAction(FailedActionPayload {
                            action: agent_action,
                            result: PlayedActionPayload {
                                action: AgentAction::Fold,
                                player_stack: self.game_state.stacks[idx],
                                idx,
                                round: self.game_state.round,
                                starting_bet,
                                final_bet: starting_bet,
                                starting_min_raise,
                                final_min_raise: self.game_state.current_round_min_raise(),
                                starting_player_bet,
                                final_player_bet: starting_player_bet,
                                players_active: self.game_state.player_active,
                                players_all_in: self.game_state.player_all_in,
                                // What's the pot worth
                                starting_pot,
                                final_pot: self.game_state.total_pot,
                            },
                        }));

                        self.record_event(GameEvent::FailedPlayerAction(FailedPlayerActionEvent {
                            player_name: self.player_name(idx),
                            player_idx: idx,
                            player_decision: decision,
                            action: AgentAction::Fold,
                            stack_after: self.game_state.stacks[idx],
                            pot_after: self.game_state.total_pot,
                        }));

                        // Actually fold the user
                        self.player_fold();
                    }
                    Ok(_added) => {
                        let player_bet = self.game_state.current_round_player_bet(idx);

                        let new_action = match agent_action {
                            AgentAction::Bet(_) => AgentAction::Bet(player_bet),
                            AgentAction::Fold => AgentAction::Fold,
                            AgentAction::Call => AgentAction::Call,
                            AgentAction::AllIn => AgentAction::AllIn,
                        };
                        // If the game_state.do_bet function returned Ok then
                        // the state is already changed so record the action as played.
                        self.record_action(Action::PlayedAction(PlayedActionPayload {
                            action: new_action,
                            player_stack: self.game_state.stacks[idx],
                            idx,
                            round: self.game_state.round,
                            starting_bet,
                            final_bet: self.game_state.current_round_bet(),
                            starting_min_raise,
                            final_min_raise: self.game_state.current_round_min_raise(),
                            starting_player_bet,
                            final_player_bet: player_bet,
                            // Keep track of who's in a
                            players_active: self.game_state.player_active,
                            players_all_in: self.game_state.player_all_in,

                            // What's the pot worth
                            starting_pot,
                            final_pot: self.game_state.total_pot,
                        }));

                        self.record_event(GameEvent::PlayerAction(PlayerActionEvent {
                            player_name: self.player_name(idx),
                            player_idx: idx,
                            player_decision: decision,
                            stack_after: self.game_state.stacks[idx],
                            pot_after: self.game_state.total_pot,
                        }));
                    }
                }
            }
        }
    }

    #[instrument]
    fn player_fold(&mut self) {
        self.game_state.fold();
        let left = self.game_state.player_active | self.game_state.player_all_in;

        // If there's only one person left then they win.
        // If there's no one left, and one person went all in they win.
        //
        if left.count() <= 1 {
            if let Some(winning_idx) = left.ones().next() {
                let total_pot = self.game_state.total_pot;
                event!(Level::INFO, winning_idx, total_pot, "folded_to_winner");
                self.game_state.award(winning_idx, total_pot);
                self.record_action(Action::Award(AwardPayload {
                    idx: winning_idx,
                    total_pot,
                    award_amount: total_pot,
                    rank: None,
                    hand: None,
                }));
                self.record_event(GameEvent::GameEnded(GameEndedEvent {
                    final_round: self.game_state.round,
                    awards: vec![Award {
                        player_idx: winning_idx,
                        player_name: self.player_name(winning_idx),
                        won_pot: total_pot,
                        stack_after: self.game_state.stacks[winning_idx],
                        rank: None,
                        hand: None,
                    }],
                }));
            }

            self.end_game();
        }
    }

    #[instrument]
    fn end_game(&mut self) {
        let current_round = self.game_state.round;
        self.game_state.complete();
        if current_round != self.game_state.round {
            self.record_action(Action::RoundAdvance(self.game_state.round));
        }
    }

    #[instrument]
    fn advance_round(&mut self) {
        let current_round = self.game_state.round;
        self.game_state.advance_round();
        if self.game_state.round != current_round {
            self.record_action(Action::RoundAdvance(self.game_state.round));
            self.record_event(GameEvent::RoundAdvance(self.game_state.round));
        }
    }

    // Make sure that all modifications to game_state are complete before calling
    // `record_action`. This is critical for making sure replays are deterministic.
    fn record_action(&mut self, action: Action) {
        event!(Level::TRACE, action = ?action, game_state = ?self.game_state, "add_action");
        self.actions.push(action);
    }

    fn record_event(&mut self, event: GameEvent) {
        self.events.push(event);
    }

    fn player_name(&self, idx: usize) -> PlayerName {
        self.player_names[idx].clone()
    }

    /// Execute a player action externally  
    pub fn execute_player_action(&mut self, decision: Decision) {
        event!(Level::TRACE, ?decision, "execute_player_action");
        self.run_agent_action(decision);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_game() {
    //     let game_id = GameId::new("123");
    //     let small_blind = 5.0;
    //     let big_blind = 10.0;
    //     let stacks = vec![100.0; 3];

    //     let _game = GameSimulation::new(game_id, big_blind, small_blind, stacks);
    //     // Game created successfully
    // }

    // #[test]
    // fn test_external_control_flow() {
    //     let game_id = GameId::new("123");
    //     let small_blind = 5.0;
    //     let big_blind = 10.0;
    //     let stacks = vec![100.0, 100.0];

    //     let mut game = GameSimulation::new(game_id, big_blind, small_blind,
    // stacks);

    //     // First run should start the game
    //     let result = game.run();
    //     assert!(
    //         !result.no_action_needed(),
    //         "Game should require action to start"
    //     );

    //     // Game should start with dealing cards to players
    //     assert_eq!(result, GameActionRequired::DealPlayerCards { idx: 1 });
    //     game.deal_cards_to_player(1, cards(vec!["2s", "3s"]));

    //     // Next run should deal cards to the next player
    //     let result = game.run();
    //     assert_eq!(result, GameActionRequired::DealPlayerCards { idx: 0 });
    //     game.deal_cards_to_player(0, cards(vec!["4s", "5s"]));

    //     // PREFLOP.

    //     // Next run should start the betting round. First player 0.
    //     let result = game.run();
    //     assert_eq!(
    //         result,
    //         GameActionRequired::PlayerToAct {
    //             idx: 0,
    //             possible_actions: vec![
    //                 PossibleAction::Fold,
    //                 PossibleAction::Call,
    //                 PossibleAction::Bet {
    //                     min: 20.0,
    //                     max: 100.0
    //                 },
    //                 PossibleAction::AllIn,
    //             ]
    //         }
    //     );
    //     game.execute_player_action(AgentAction::Call);

    //     // Next run should ask player 1 to act
    //     let result = game.run();
    //     assert_eq!(
    //         result,
    //         GameActionRequired::PlayerToAct {
    //             idx: 1,
    //             possible_actions: vec![
    //                 PossibleAction::Call,
    //                 PossibleAction::Bet {
    //                     min: 20.0,
    //                     max: 100.0
    //                 },
    //                 PossibleAction::AllIn,
    //             ]
    //         }
    //     );
    //     game.execute_player_action(AgentAction::Call);

    //     // FLOP.

    //     // Bets: p0: 10, p1: 10

    //     // Next run should deal the flop
    //     let result = game.run();
    //     assert_eq!(
    //         result,
    //         GameActionRequired::DealCommunityCards { num_cards: 3 }
    //     );
    //     game.deal_community_cards_external(cards(vec!["6s", "7s", "8s"]));

    //     // Next run should start the flop betting round.
    //     // First player 1 bets 30.
    //     let result = game.run();
    //     assert_eq!(
    //         result,
    //         GameActionRequired::PlayerToAct {
    //             idx: 1,
    //             possible_actions: vec![
    //                 PossibleAction::Call,
    //                 PossibleAction::Bet {
    //                     min: 10.0,
    //                     max: 90.0
    //                 },
    //                 PossibleAction::AllIn,
    //             ]
    //         }
    //     );
    //     game.execute_player_action(AgentAction::Bet(30.0));

    //     // Then player 0 bets 80.
    //     let result = game.run();
    //     assert_eq!(
    //         result,
    //         GameActionRequired::PlayerToAct {
    //             idx: 0,
    //             possible_actions: vec![
    //                 PossibleAction::Fold,
    //                 PossibleAction::Call,
    //                 PossibleAction::Bet {
    //                     min: 60.0,
    //                     max: 90.0
    //                 },
    //                 PossibleAction::AllIn,
    //             ]
    //         }
    //     );
    //     game.execute_player_action(AgentAction::Bet(80.0));

    //     // Then player 1 calls.
    //     let result = game.run();
    //     assert_eq!(
    //         result,
    //         GameActionRequired::PlayerToAct {
    //             idx: 1,
    //             possible_actions: vec![
    //                 PossibleAction::Fold,
    //                 PossibleAction::Call,
    //                 PossibleAction::AllIn,
    //             ]
    //         }
    //     );
    //     game.execute_player_action(AgentAction::Call);

    //     // TURN.
    //     let result = game.run();
    //     assert_eq!(
    //         result,
    //         GameActionRequired::DealCommunityCards { num_cards: 1 }
    //     );
    //     game.deal_community_cards_external(cards(vec!["Ad"]));

    //     // Next run should start the turn betting round.
    //     // First player 1 bets 40.
    //     let result = game.run();
    //     assert_eq!(
    //         result,
    //         GameActionRequired::PlayerToAct {
    //             idx: 1,
    //             possible_actions: vec![
    //                 PossibleAction::Call,
    //                 PossibleAction::Bet {
    //                     min: 10.0,
    //                     max: 10.0
    //                 },
    //                 PossibleAction::AllIn,
    //             ]
    //         }
    //     );
    //     game.execute_player_action(AgentAction::Call);

    //     // Then player 0 calls.
    //     let result = game.run();
    //     assert_eq!(
    //         result,
    //         GameActionRequired::PlayerToAct {
    //             idx: 0,
    //             possible_actions: vec![
    //                 PossibleAction::Call,
    //                 PossibleAction::Bet {
    //                     min: 10.0,
    //                     max: 10.0
    //                 },
    //                 PossibleAction::AllIn,
    //             ]
    //         }
    //     );
    //     game.execute_player_action(AgentAction::Call);

    //     // RIVER.
    //     let result = game.run();
    //     assert_eq!(
    //         result,
    //         GameActionRequired::DealCommunityCards { num_cards: 1 }
    //     );
    //     game.deal_community_cards_external(cards(vec!["Kh"]));

    //     // Next run should start the river betting round.
    //     // Player 1 bets allin.
    //     let result = game.run();
    //     assert_eq!(
    //         result,
    //         GameActionRequired::PlayerToAct {
    //             idx: 1,
    //             possible_actions: vec![
    //                 PossibleAction::Call,
    //                 PossibleAction::Bet {
    //                     min: 10.0,
    //                     max: 10.0
    //                 },
    //                 PossibleAction::AllIn,
    //             ]
    //         }
    //     );
    //     game.execute_player_action(AgentAction::AllIn);

    //     // Then player 0 calls.
    //     let result = game.run();
    //     assert_eq!(
    //         result,
    //         GameActionRequired::PlayerToAct {
    //             idx: 0,
    //             possible_actions: vec![PossibleAction::Fold,
    // PossibleAction::Call,]         }
    //     );
    //     game.execute_player_action(AgentAction::Call);

    //     // SHOWDOWN.
    //     let result = game.run();
    //     assert_eq!(result, GameActionRequired::NoActionRequired);

    //     // Game should be over now
    //     assert!(game.game_state.is_complete());

    //     // Check finals stacks and who won
    //     assert_eq!(game.game_state.player_winnings[0], 200.0);
    //     assert_eq!(game.game_state.player_winnings[1], 0.0);
    // }

    #[allow(dead_code)]
    fn cards(cards: Vec<&str>) -> Vec<Card> {
        cards
            .into_iter()
            .map(|c| Card::try_from(c).unwrap())
            .collect()
    }
}

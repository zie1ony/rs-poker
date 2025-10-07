use rs_poker::core::Rank;
use rs_poker::{
    arena::{
        action::{self, AgentAction},
        game_state::Round,
    },
    core::Card,
};
use rs_poker_types::{
    game_event::{ForcedBetKind, GameEvent},
    player::PlayerName,
};

pub struct GameSummary {
    pub events: Vec<GameEvent>,
    pub for_player: Option<PlayerName>,
}

impl GameSummary {
    pub fn full(events: Vec<GameEvent>) -> Self {
        GameSummary {
            events,
            for_player: None,
        }
    }

    pub fn for_player(events: Vec<GameEvent>, player: PlayerName) -> Self {
        GameSummary {
            events,
            for_player: Some(player),
        }
    }

    pub fn summary(&self) -> String {
        let mut summary = String::new();
        for event in &self.events {
            match event {
                GameEvent::GameStarted(e) => {
                    let config = &e.settings;
                    let players_count = config.players.len();

                    summary.push_str(&format!("Game Started - ID: {:?}\n", config.game_id));
                    summary.push_str(&format!("Small blind: {}\n", config.small_blind));
                    summary.push_str(&format!("Big blind: {}\n", config.big_blind()));
                    summary.push_str(&format!("Players: {}\n", players_count));

                    for i in 0..players_count {
                        let player_name = config.players[i].name();
                        let stack = config.stacks[i];
                        let player_name = if let Some(name) = &self.for_player {
                            if name == &player_name {
                                format!("{} (You)", player_name)
                            } else {
                                player_name.to_string()
                            }
                        } else {
                            player_name.to_string()
                        };
                        summary.push_str(&format!("- {} has stack: {}\n", player_name, stack));
                    }

                    match &self.for_player {
                        Some(name) => {
                            if let Some((i, _)) = config
                                .players
                                .iter()
                                .enumerate()
                                .find(|(_, p)| p.name() == *name)
                            {
                                let hand = &config.hands.clone().unwrap();
                                let hand = &hand[i];
                                summary
                                    .push_str(&format!("\nYour Hand: {} {}\n", hand[0], hand[1]));
                            } else {
                                // Player wasn't in this game (e.g., eliminated/insufficient chips)
                                summary.push_str(&format!(
                                    "\nYou did not participate in this game.\n"
                                ));
                            }
                        }
                        None => {
                            summary.push_str("\nHole Cards:\n");
                            let hands = config.hands.clone().unwrap();
                            for (i, hand) in hands.iter().enumerate() {
                                let player_name = config.players[i].name();
                                summary.push_str(&format!(
                                    "- {}: {:?} {:?}\n",
                                    player_name, hand[0], hand[1]
                                ));
                            }
                            let community_cards = config.community_cards.unwrap();
                            summary.push_str(&format!(
                                "\nCommunity Cards: {:?} {:?} {:?} {:?} {:?}\n",
                                community_cards[0],
                                community_cards[1],
                                community_cards[2],
                                community_cards[3],
                                community_cards[4]
                            ));
                        }
                    }
                }
                GameEvent::RoundAdvance(round) => {
                    let round_name = match round {
                        Round::Preflop => "Preflop",
                        Round::DealFlop => "Flop",
                        Round::DealTurn => "Turn",
                        Round::DealRiver => "River",
                        _ => continue,
                        // Round::Starting => "DEBUG: Starting",
                        // Round::Ante => "DEBUG: Ante",
                        // Round::DealPreflop => "DEBUG: Deal Preflop",
                        // Round::Flop => "DEBUG: Flop",
                        // Round::Turn => "DEBUG: Turn",
                        // Round::River => "DEBUG: River",
                        // Round::Showdown => "DEBUG: Showdown",
                        // Round::Complete => "DEBUG: Complete",
                    };

                    summary.push_str(&format!("\n--- {} ---\n", round_name));
                }
                GameEvent::ForcedBet(bet_event) => {
                    let bet = match bet_event.bet_kind {
                        ForcedBetKind::SmallBlind => "small blind",
                        ForcedBetKind::BigBlind => "big blind",
                    };

                    summary.push_str(&format!(
                        "{} posts {} of {} {}\n",
                        bet_event.player_name,
                        bet,
                        bet_event.bet,
                        after_action_info(bet_event.stack_after, bet_event.pot_after)
                    ));
                }
                GameEvent::FailedPlayerAction(e) => {
                    let player_name = &e.player_name;

                    if self.for_player.is_none() {
                        summary.push_str(&format!(
                            "{} thinks: \"{}\"\n",
                            player_name, e.player_decision.reason
                        ));

                        let decision_action_str = action_to_str(&e.player_decision.action);
                        summary.push_str(&format!(
                            "{} tries to {}, but it's invalid action.\n",
                            player_name, decision_action_str
                        ));
                    }

                    let forced_action_str = action_to_str(&e.action);
                    summary.push_str(&format!(
                        "{} {} {}\n",
                        player_name,
                        forced_action_str,
                        after_action_info(e.stack_after, e.pot_after)
                    ));
                }
                GameEvent::PlayerAction(e) => {
                    let player_name = &e.player_name;

                    // If this summary is for a specific player, skip showing all thoughts.
                    if self.for_player.is_none() {
                        summary.push_str(&format!(
                            "{} thinks: \"{}\"\n",
                            player_name, e.player_decision.reason
                        ));
                    }

                    let action_str = action_to_str(&e.player_decision.action);
                    summary.push_str(&format!(
                        "{} {} {}\n",
                        player_name,
                        action_str,
                        after_action_info(e.stack_after, e.pot_after)
                    ));
                }
                GameEvent::ShowCommunityCards(show_event) => {
                    let card_str = if show_event.cards.len() == 1 {
                        "card"
                    } else {
                        "cards"
                    };
                    summary.push_str(&format!(
                        "{:?} {}: {}\n",
                        show_event.round,
                        card_str,
                        show_event
                            .cards
                            .iter()
                            .map(|c| c.to_string())
                            .collect::<Vec<_>>()
                            .join(" ")
                    ));
                }
                GameEvent::GameEnded(game_ended_event) => {
                    summary.push_str("\n--- Game Ended ---\n");
                    for award in &game_ended_event.awards {
                        let hand_info = match (&award.hand, &award.rank) {
                            (Some(hand), Some(rank)) => {
                                let mut cards: Vec<Card> = hand.iter().collect();
                                cards.sort();
                                let cards: Vec<String> =
                                    cards.iter().map(|c| c.to_string()).collect();
                                format!(" with {} ({})", cards.join(" "), rank_to_str(rank))
                            }
                            (Some(hand), None) => {
                                let mut cards: Vec<Card> = hand.iter().collect();
                                cards.sort();
                                let cards: Vec<String> =
                                    cards.iter().map(|c| c.to_string()).collect();
                                format!(" with {}", cards.join(" "))
                            }
                            _ => String::new(),
                        };

                        summary.push_str(&format!(
                            "{} wins {}{} (stack after: {})\n",
                            award.player_name, award.won_pot, hand_info, award.stack_after
                        ));
                    }
                }
            }
        }
        summary
    }
}

pub fn after_action_info(stack: f32, pot: f32) -> String {
    format!("(stack: {}, pot: {})", stack, pot)
}

pub fn action_to_str(action: &AgentAction) -> String {
    match action {
        action::AgentAction::Fold => "folds".to_string(),
        action::AgentAction::Call => "calls".to_string(),
        action::AgentAction::Bet(amount) => format!("bets {}", amount),
        action::AgentAction::AllIn => "goes all-in".to_string(),
    }
}

pub fn rank_to_str(rank: &Rank) -> String {
    match rank {
        Rank::HighCard(_) => "High Card".to_string(),
        Rank::OnePair(_) => "One Pair".to_string(),
        Rank::TwoPair(_) => "Two Pair".to_string(),
        Rank::ThreeOfAKind(_) => "Three of a Kind".to_string(),
        Rank::Straight(_) => "Straight".to_string(),
        Rank::Flush(_) => "Flush".to_string(),
        Rank::FullHouse(_) => "Full House".to_string(),
        Rank::FourOfAKind(_) => "Four of a Kind".to_string(),
        Rank::StraightFlush(_) => "Straight Flush".to_string(),
    }
}

use rs_poker::arena::{action::AgentAction, game_state::Round};
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
                    let players_count = e.players.len();

                    summary.push_str(&format!("Game Started - ID: {:?}\n", e.game_id));
                    summary.push_str(&format!("Small blind: {}\n", e.small_blind));
                    summary.push_str(&format!("Big blind: {}\n", e.big_blind));
                    summary.push_str(&format!("Players: {}\n", players_count));

                    for i in 0..players_count {
                        let player_name = e.players[i].name();
                        let stack = e.initial_stacks[i];
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
                            if let Some((i, _)) = e
                                .players
                                .iter()
                                .enumerate()
                                .find(|(_, p)| p.name() == *name)
                            {
                                let hand = e.hands[i];
                                summary
                                    .push_str(&format!("\nYour Hand: {} {}\n", hand[0], hand[1]));
                            } else {
                                panic!("Player {:?} not found in game players", name);
                            }
                        }
                        None => {
                            summary.push_str("\nHole Cards:\n");
                            for (i, hand) in e.hands.iter().enumerate() {
                                let player_name = e.players[i].name();
                                summary.push_str(&format!(
                                    "- {}: {} {}\n",
                                    player_name, hand[0], hand[1]
                                ));
                            }

                            summary.push_str(&format!(
                                "\nCommunity Cards: {} {} {} {} {}\n",
                                e.community_cards[0],
                                e.community_cards[1],
                                e.community_cards[2],
                                e.community_cards[3],
                                e.community_cards[4]
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
                GameEvent::FailedPlayerAction(_failed_player_action_event) => {
                    panic!("Should never happen");
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

                    let action_str = match e.player_decision.action {
                        AgentAction::Fold => String::from("folds"),
                        AgentAction::Call => String::from("calls"),
                        AgentAction::Bet(amount) => {
                            String::from(format!("increases to {}", amount))
                        }
                        AgentAction::AllIn => String::from("goes all-in"),
                    };
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
                        summary.push_str(&format!(
                            "{} wins {} (stack after: {})\n",
                            award.player_name, award.won_pot, award.stack_after
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

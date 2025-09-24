use std::collections::HashMap;

use rs_poker_types::{game::GameId, game_event::GameEvent, player::PlayerName, tournament_event::TournamentEvent};

use crate::game_summary::GameSummary;

pub struct TournamentSummary {
    pub tournament_events: Vec<TournamentEvent>,
    pub game_events: HashMap<GameId, Vec<GameEvent>>,
    pub for_player: Option<PlayerName>,
}

impl TournamentSummary {
    pub fn full(
        tournament_events: Vec<TournamentEvent>,
        game_events: HashMap<GameId, Vec<GameEvent>>,
    ) -> Self {
        TournamentSummary {
            tournament_events,
            game_events,
            for_player: None,
        }
    }

    pub fn for_player(
        tournament_events: Vec<TournamentEvent>,
        game_events: HashMap<GameId, Vec<GameEvent>>,
        player: PlayerName,
    ) -> Self {
        TournamentSummary {
            tournament_events,
            game_events,
            for_player: Some(player),
        }
    }

    pub fn summary(&self) -> String {
        let mut summary = String::new();
        let mut game_number = 0;
        
        // First pass - collect tournament stats
        let mut tournament_id = None;
        let mut tournament_settings = None;
        let mut winner = None;
        let mut total_games = 0;
        
        for tournament_event in &self.tournament_events {
            match tournament_event {
                TournamentEvent::TournamentCreated(event) => {
                    tournament_id = Some(&event.tournament_id);
                    tournament_settings = Some(&event.settings);
                }
                TournamentEvent::GameEnded(_) => {
                    total_games += 1;
                }
                TournamentEvent::TournamentFinished(event) => {
                    winner = Some(&event.winner);
                }
                _ => {}
            }
        }
        
        // Tournament stats section
        if let (Some(id), Some(settings)) = (tournament_id, tournament_settings) {
            summary.push_str("=== TOURNAMENT STATISTICS ===\n");
            summary.push_str(&format!("Tournament ID: {:?}\n", id));
            summary.push_str(&format!("Players: {} ({})\n", 
                settings.players.len(), 
                settings.players.iter()
                    .map(|p| {
                        let player_name = p.name();
                        if let Some(for_player) = &self.for_player {
                            if &player_name == for_player {
                                format!("{} (You)", player_name)
                            } else {
                                player_name.to_string()
                            }
                        } else {
                            player_name.to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
            summary.push_str(&format!("Starting Stack: {}\n", settings.starting_player_stack));
            summary.push_str(&format!("Starting Blinds: {}/{}\n", settings.starting_small_blind, settings.starting_small_blind * 2.0));
            
            if let Some(double_blinds_every) = settings.double_blinds_every_n_games {
                summary.push_str(&format!("Blind Increase Frequency: Every {} games\n", double_blinds_every));
            }
            
            summary.push_str(&format!("Total Games Played: {}\n", total_games));
            if let Some(w) = winner {
                let winner_display = if let Some(for_player) = &self.for_player {
                    if w == for_player {
                        format!("{} (You)", w)
                    } else {
                        w.to_string()
                    }
                } else {
                    w.to_string()
                };
                summary.push_str(&format!("Winner: {}\n", winner_display));
            }
            summary.push_str("\n");
        }
        
        // Games section
        for tournament_event in &self.tournament_events {
            match tournament_event {
                TournamentEvent::GameStarted(event) => {
                    summary.push_str(&format!("=== GAME {} ===\n", game_number));
                    game_number += 1;
                    
                    // Add game summary if we have game events for this game
                    if let Some(game_events) = self.game_events.get(&event.game_id) {
                        let game_summary = match &self.for_player {
                            Some(player) => GameSummary::for_player(game_events.clone(), player.clone()),
                            None => GameSummary::full(game_events.clone()),
                        };
                        summary.push_str(&game_summary.summary());
                        summary.push_str("\n");
                    }
                }
                
                TournamentEvent::GameEnded(_event) => {

                }
                
                _ => {}
            }
        }
        
        // Tournament finish section
        for tournament_event in &self.tournament_events {
            if let TournamentEvent::TournamentFinished(event) = tournament_event {
                summary.push_str("=== TOURNAMENT COMPLETED ===\n");
                let winner_display = if let Some(for_player) = &self.for_player {
                    if &event.winner == for_player {
                        format!("{} (You)", event.winner)
                    } else {
                        event.winner.to_string()
                    }
                } else {
                    event.winner.to_string()
                };
                summary.push_str(&format!("Winner: {}\n", winner_display));
                summary.push_str(&format!("Tournament ID: {:?}\n", event.tournament_id));
                summary.push_str(&format!("Games played: {}", game_number));
            }
        }
        
        summary
    }
}


use rs_poker_types::{
    game::{GameFinalResults, GameId, GameSettings},
    tournament::{TournamentId, TournamentInfo, TournamentSettings, TournamentStatus},
    tournament_event::{
        GameEndedEvent, GameStartedEvent, TournamentCreatedEvent, TournamentEvent,
        TournamentFinishedEvent,
    },
};

#[derive(Clone, Debug, PartialEq)]
pub struct TournamentInstance {
    pub tournament_id: TournamentId,
    pub events: Vec<TournamentEvent>,
    pub settings: TournamentSettings,
    pub status: TournamentStatus,
    pub next_game_number: usize,
    pub next_small_blind: f32,
    pub current_game_id: Option<GameId>,
    pub player_stacks: Vec<f32>,
    pub game_ids: Vec<GameId>,
}

impl TournamentInstance {
    pub fn new(settings: &TournamentSettings) -> Self {
        Self {
            tournament_id: settings.tournament_id.clone(),
            events: vec![TournamentCreatedEvent::new(&settings)],
            settings: settings.clone(),
            status: TournamentStatus::WaitingForNextGame,
            next_game_number: 0,
            next_small_blind: settings.starting_small_blind,
            current_game_id: None,
            player_stacks: vec![settings.starting_player_stack; settings.players.len()],
            game_ids: vec![],
        }
    }

    pub fn tournament_id(&self) -> TournamentId {
        self.tournament_id.clone()
    }

    pub fn game_ids(&self) -> &Vec<GameId> {
        &self.game_ids
    }

    pub fn events(&self) -> Vec<TournamentEvent> {
        self.events.clone()
    }

    pub fn status(&self) -> &TournamentStatus {
        &self.status
    }

    pub fn is_completed(&self) -> bool {
        matches!(self.status, TournamentStatus::Completed)
    }

    pub fn winner(&self) -> Option<&rs_poker_types::player::Player> {
        if matches!(self.status, TournamentStatus::Completed) {
            // Find the player with the highest stack (or the only one with money)
            if let Some((winner_index, _)) = self
                .player_stacks
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            {
                self.settings.players.get(winner_index)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn next_action(&mut self) -> Option<TournamentAction> {
        match self.status {
            TournamentStatus::WaitingForNextGame => {
                // Check if tournament should be completed by counting players who can afford
                // the next small blind
                let small_blind = self.next_small_blind;
                let players_with_sufficient_chips = self
                    .player_stacks
                    .iter()
                    .filter(|&&stack| stack > small_blind)
                    .count();
                if players_with_sufficient_chips <= 1 {
                    self.status = TournamentStatus::Completed;

                    // Record tournament finished event if there's a winner
                    if let Some(winner) = self.winner() {
                        let tournament_finished_event =
                            TournamentEvent::TournamentFinished(TournamentFinishedEvent {
                                timestamp: std::time::SystemTime::now(),
                                tournament_id: self.tournament_id.clone(),
                                winner: winner.name(),
                            });
                        self.events.push(tournament_finished_event);
                    }

                    None
                } else {
                    // Start next game
                    if let Ok(game_settings) = self.start_next_game() {
                        Some(TournamentAction::StartNextGame { game_settings })
                    } else {
                        None
                    }
                }
            }
            TournamentStatus::GameInProgress => {
                // Wait for the current game to finish
                if let Some(game_id) = &self.current_game_id {
                    Some(TournamentAction::FinishGame {
                        game_id: game_id.clone(),
                    })
                } else {
                    None
                }
            }
            TournamentStatus::Completed => None,
        }
    }

    pub fn start_next_game(&mut self) -> Result<GameSettings, TournamentError> {
        // Can only start a new game if the tournament is waiting for the next game.
        if !matches!(self.status, TournamentStatus::WaitingForNextGame) {
            return Err(TournamentError::CannotStartNewGame);
        }

        let game_number = self.next_game_number;
        self.next_game_number += 1;

        let mut small_blind = self.next_small_blind;
        // if double_blinds_every_n_games is set to 3 it should have double on game 3,
        // 6, 9, ...
        if let Some(n) = self.settings.double_blinds_every_n_games {
            if game_number > 0 && game_number % n == 0 {
                small_blind *= 2.0;
                self.next_small_blind = small_blind;
            }
        }

        let game_id = GameId::for_tournament(game_number);

        // Only include players with positive stacks (more than small blind).
        let positive_stacks_ids: Vec<usize> = self
            .player_stacks
            .iter()
            .enumerate()
            .filter_map(|(idx, &stack)| if stack > small_blind { Some(idx) } else { None })
            .collect();

        let positive_stacks: Vec<f32> = positive_stacks_ids
            .iter()
            .map(|&idx| self.player_stacks[idx])
            .collect();

        let positive_players: Vec<rs_poker_types::player::Player> = positive_stacks_ids
            .iter()
            .map(|&idx| self.settings.players[idx].clone())
            .collect();

        assert_eq!(
            positive_players.len(),
            positive_stacks.len(),
            "Player and stack counts must match"
        );

        let new_game = GameSettings {
            tournament_id: Some(self.tournament_id.clone()),
            torunament_game_number: Some(game_number),
            game_id: game_id.clone(),
            small_blind,
            players: positive_players,
            stacks: positive_stacks,
        };

        // Record game started event
        let game_started_event = TournamentEvent::GameStarted(GameStartedEvent {
            timestamp: std::time::SystemTime::now(),
            game_id: game_id.clone(),
            player_names: self.settings.players.iter().map(|p| p.name()).collect(),
            player_stacks: self.player_stacks.clone(),
        });
        self.events.push(game_started_event);

        self.status = TournamentStatus::GameInProgress;
        self.current_game_id = Some(game_id);
        self.game_ids.push(new_game.game_id.clone());
        Ok(new_game)
    }

    pub fn finish_game(
        &mut self,
        game_final_results: &GameFinalResults,
    ) -> Result<(), TournamentError> {
        // Can only finish a game if the tournament is in progress.
        if !matches!(self.status, TournamentStatus::GameInProgress) {
            return Err(TournamentError::CannotFinishGame);
        }

        // Check if the game ID matches the current game.
        if let Some(current_game_id) = &self.current_game_id {
            if *current_game_id != game_final_results.game_id {
                return Err(TournamentError::GameIdMismatch);
            }
        } else {
            return Err(TournamentError::CannotFinishGame);
        }

        // Validate that the game results contain valid players from the tournament
        let tournament_player_names: Vec<_> =
            self.settings.players.iter().map(|p| p.name()).collect();

        for player_name in &game_final_results.player_names {
            if !tournament_player_names.contains(player_name) {
                return Err(TournamentError::PlayersMismatch);
            }
        }

        // Update player stacks based on final results
        // Only update stacks for players who participated in the game
        for (i, player_name) in game_final_results.player_names.iter().enumerate() {
            if let Some(tournament_index) = tournament_player_names
                .iter()
                .position(|name| name == player_name)
            {
                if i < game_final_results.final_stacks.len() {
                    self.player_stacks[tournament_index] = game_final_results.final_stacks[i];
                }
            }
        }

        // Record game ended event with full tournament state
        let game_ended_event = TournamentEvent::GameEnded(GameEndedEvent {
            timestamp: std::time::SystemTime::now(),
            game_id: game_final_results.game_id.clone(),
            player_names: self.settings.players.iter().map(|p| p.name()).collect(),
            player_stacks: self.player_stacks.clone(),
        });
        self.events.push(game_ended_event);

        // Check if tournament should be completed (only one player can afford the next
        // small blind)
        let players_with_sufficient_chips = self
            .player_stacks
            .iter()
            .filter(|&&stack| stack > self.next_small_blind)
            .count();
        if players_with_sufficient_chips <= 1 {
            self.status = TournamentStatus::Completed;

            // Record tournament finished event
            if let Some(winner) = self.winner() {
                let tournament_finished_event =
                    TournamentEvent::TournamentFinished(TournamentFinishedEvent {
                        timestamp: std::time::SystemTime::now(),
                        tournament_id: self.tournament_id.clone(),
                        winner: winner.name(),
                    });
                self.events.push(tournament_finished_event);
            }
        } else {
            self.status = TournamentStatus::WaitingForNextGame;
        }

        self.current_game_id = None;

        Ok(())
    }

    pub fn info(&self) -> TournamentInfo {
        TournamentInfo {
            settings: self.settings.clone(),
            status: self.status.clone(),
            games_played: self.next_game_number,
            current_game_id: self.current_game_id.clone(),
            winner: self.winner().map(|p| p.name()),
        }
    }
}

impl From<Vec<TournamentEvent>> for TournamentInstance {
    fn from(events: Vec<TournamentEvent>) -> Self {
        if events.is_empty() {
            panic!("Cannot reconstruct tournament from empty events");
        }

        // First event must be TournamentCreated
        let settings = match &events[0] {
            TournamentEvent::TournamentCreated(event) => event.settings.clone(),
            _ => panic!("First event must be TournamentCreated"),
        };

        let mut instance = Self {
            tournament_id: settings.tournament_id.clone(),
            events: Vec::new(),
            settings: settings.clone(),
            status: TournamentStatus::WaitingForNextGame,
            next_game_number: 0,
            next_small_blind: settings.starting_small_blind,
            current_game_id: None,
            player_stacks: vec![settings.starting_player_stack; settings.players.len()],
            game_ids: vec![],
        };

        // Process each event to rebuild state
        for event in events {
            match &event {
                TournamentEvent::TournamentCreated(_) => {
                    // Already handled above, just add to events
                }
                TournamentEvent::GameStarted(game_started) => {
                    instance.status = TournamentStatus::GameInProgress;
                    instance.current_game_id = Some(game_started.game_id.clone());
                    instance.game_ids.push(game_started.game_id.clone());
                    instance.next_game_number += 1;

                    // Update blinds if needed (recalculate based on game number)
                    let game_number = instance.next_game_number - 1; // 0-indexed
                    if let Some(n) = instance.settings.double_blinds_every_n_games {
                        let mut small_blind = instance.settings.starting_small_blind;
                        let blind_doublings = game_number / n;
                        for _ in 0..blind_doublings {
                            small_blind *= 2.0;
                        }
                        instance.next_small_blind = small_blind;
                    }
                }
                TournamentEvent::GameEnded(game_ended) => {
                    instance.status = TournamentStatus::WaitingForNextGame;
                    instance.current_game_id = None;
                    instance.player_stacks = game_ended.player_stacks.clone();
                }
                TournamentEvent::TournamentFinished(_) => {
                    instance.status = TournamentStatus::Completed;
                    instance.current_game_id = None;
                }
            }
            instance.events.push(event);
        }

        instance
    }
}

pub enum TournamentAction {
    StartNextGame { game_settings: GameSettings },
    FinishGame { game_id: GameId },
}

#[derive(Debug, PartialEq, Clone)]
pub enum TournamentError {
    TournamentAlreadyCompleted,
    CannotStartNewGame,
    CannotFinishGame,
    GameIdMismatch,
    PlayersMismatch,
}

#[cfg(test)]
mod tests {
    use rs_poker_types::{
        player::{Player, PlayerName},
        tournament::TournamentEndCondition,
    };

    use super::*;

    #[test]
    fn test_tournament_complete() {
        let settings = TournamentSettings {
            tournament_id: TournamentId::random(),
            players: vec![
                Player::random("Alice"),
                Player::random("Bob"),
                Player::random("Charlie"),
            ],
            starting_player_stack: 100.0,
            starting_small_blind: 5.0,
            double_blinds_every_n_games: Some(3),
            end_condition: TournamentEndCondition::SingleWinner,
            see_historical_thoughts: false,
            public_chat: false,
        };

        let mut tournament = TournamentInstance::new(&settings);

        // Verify initial state
        assert_eq!(tournament.status(), &TournamentStatus::WaitingForNextGame);
        assert_eq!(tournament.player_stacks, vec![100.0, 100.0, 100.0]);
        assert!(tournament.winner().is_none());
        assert_eq!(tournament.events.len(), 1); // TournamentCreated event

        // GAME 0: Test error cases and successful completion
        let game0 = tournament.start_next_game().unwrap();
        assert_eq!(game0.tournament_id, Some(settings.tournament_id.clone()));
        assert_eq!(game0.torunament_game_number, Some(0));
        assert_eq!(game0.small_blind, 5.0);
        assert_eq!(game0.players, settings.players);
        assert_eq!(game0.stacks, vec![100.0, 100.0, 100.0]);
        assert_eq!(tournament.status(), &TournamentStatus::GameInProgress);
        assert_eq!(tournament.events.len(), 2); // TournamentCreated + GameStarted

        // Should fail if we try to start another game without finishing the first
        assert_eq!(
            tournament.start_next_game().unwrap_err(),
            TournamentError::CannotStartNewGame
        );

        // Should fail if we try to finish a game with a mismatched game ID
        let fake_game_id = GameId::random();
        let fake_results = GameFinalResults {
            game_id: fake_game_id,
            player_names: vec![
                PlayerName::new("Alice"),
                PlayerName::new("Bob"),
                PlayerName::new("Charlie"),
            ],
            final_stacks: vec![150.0, 50.0, 100.0],
        };
        assert_eq!(
            tournament.finish_game(&fake_results).unwrap_err(),
            TournamentError::GameIdMismatch
        );

        // Should fail if we try to finish a game with mismatched players
        let wrong_players_results = GameFinalResults {
            game_id: game0.game_id.clone(),
            player_names: vec![
                PlayerName::new("Alice"),
                PlayerName::new("Bob"),
                PlayerName::new("Dave"),
            ],
            final_stacks: vec![150.0, 50.0, 100.0],
        };
        assert_eq!(
            tournament.finish_game(&wrong_players_results).unwrap_err(),
            TournamentError::PlayersMismatch
        );

        // Should now succeed with fewer players (simulating eliminated players)
        // This behavior changed - we now allow partial results
        let fewer_players_results = GameFinalResults {
            game_id: game0.game_id.clone(),
            player_names: vec![PlayerName::new("Alice"), PlayerName::new("Bob")],
            final_stacks: vec![150.0, 50.0],
        };
        // This should now succeed instead of failing
        tournament.finish_game(&fewer_players_results).unwrap();

        // Check that player stacks were updated correctly
        assert_eq!(tournament.player_stacks, vec![150.0, 50.0, 100.0]); // Charlie's stack unchanged
        assert_eq!(tournament.status(), &TournamentStatus::WaitingForNextGame);
        assert_eq!(tournament.events.len(), 3); // TournamentCreated + GameStarted + GameEnded

        // GAME 1: Alice continues winning, but now starting from different stacks
        let game1 = tournament.start_next_game().unwrap();
        assert_eq!(game1.torunament_game_number, Some(1));
        assert_eq!(game1.small_blind, 5.0); // Blinds don't double until game 3
        assert_eq!(game1.stacks, vec![150.0, 50.0, 100.0]); // Uses updated stacks from game 0

        let game1_results = GameFinalResults {
            game_id: game1.game_id.clone(),
            player_names: vec![
                PlayerName::new("Alice"),
                PlayerName::new("Bob"),
                PlayerName::new("Charlie"),
            ],
            final_stacks: vec![180.0, 60.0, 60.0], // Redistributed chips
        };
        tournament.finish_game(&game1_results).unwrap();
        assert_eq!(tournament.player_stacks, vec![180.0, 60.0, 60.0]);
        assert_eq!(tournament.events.len(), 5); // +GameStarted +GameEnded

        // GAME 2: Alice dominates more
        let game2 = tournament.start_next_game().unwrap();
        assert_eq!(game2.torunament_game_number, Some(2));
        assert_eq!(game2.small_blind, 5.0); // Still no blind doubling

        let game2_results = GameFinalResults {
            game_id: game2.game_id.clone(),
            player_names: vec![
                PlayerName::new("Alice"),
                PlayerName::new("Bob"),
                PlayerName::new("Charlie"),
            ],
            final_stacks: vec![250.0, 25.0, 25.0], // Alice gains 30, others lose 15 each
        };
        tournament.finish_game(&game2_results).unwrap();
        assert_eq!(tournament.player_stacks, vec![250.0, 25.0, 25.0]);
        assert_eq!(tournament.events.len(), 7); // +GameStarted +GameEnded

        // GAME 3: Blinds double, Alice eliminates Bob
        let game3 = tournament.start_next_game().unwrap();
        assert_eq!(game3.torunament_game_number, Some(3));
        assert_eq!(game3.small_blind, 10.0); // Blinds should double on game 3

        let game3_results = GameFinalResults {
            game_id: game3.game_id.clone(),
            player_names: vec![
                PlayerName::new("Alice"),
                PlayerName::new("Bob"),
                PlayerName::new("Charlie"),
            ],
            final_stacks: vec![275.0, 0.0, 25.0], // Alice eliminates Bob
        };
        tournament.finish_game(&game3_results).unwrap();
        assert_eq!(tournament.player_stacks, vec![275.0, 0.0, 25.0]);
        assert_eq!(tournament.status(), &TournamentStatus::WaitingForNextGame); // Still 2 players with money
        assert_eq!(tournament.events.len(), 9); // +GameStarted +GameEnded

        // GAME 4: Alice eliminates Charlie and wins the tournament
        // Bob is excluded from this game because he has 0.0 chips (less than small
        // blind of 10.0)
        let game4 = tournament.start_next_game().unwrap();
        assert_eq!(game4.torunament_game_number, Some(4));
        assert_eq!(game4.small_blind, 10.0); // Blinds stay doubled
        assert_eq!(game4.stacks, vec![275.0, 25.0]); // Only Alice and Charlie have enough chips
        assert_eq!(game4.players.len(), 2); // Only Alice and Charlie

        let game4_results = GameFinalResults {
            game_id: game4.game_id.clone(),
            player_names: vec![PlayerName::new("Alice"), PlayerName::new("Charlie")],
            final_stacks: vec![300.0, 0.0], // Alice wins everything from Charlie
        };
        tournament.finish_game(&game4_results).unwrap();

        // Tournament should be completed
        assert_eq!(tournament.status(), &TournamentStatus::Completed);
        assert_eq!(tournament.player_stacks, vec![300.0, 0.0, 0.0]);

        // Alice should be the winner
        let winner = tournament.winner().unwrap();
        assert_eq!(winner.name(), PlayerName::new("Alice"));

        // No more actions should be available
        assert!(tournament.next_action().is_none());

        // Verify all expected events were recorded
        assert_eq!(tournament.events.len(), 12); // TournamentCreated + 5*(GameStarted+GameEnded) + TournamentFinished

        // Check event types in order
        match &tournament.events[0] {
            TournamentEvent::TournamentCreated(_) => {}
            _ => panic!("Expected TournamentCreated event"),
        }

        for i in 0..5 {
            let game_started_idx = 1 + i * 2;
            let game_ended_idx = 2 + i * 2;

            match &tournament.events[game_started_idx] {
                TournamentEvent::GameStarted(_) => {}
                _ => panic!("Expected GameStarted event at index {}", game_started_idx),
            }

            match &tournament.events[game_ended_idx] {
                TournamentEvent::GameEnded(_) => {}
                _ => panic!("Expected GameEnded event at index {}", game_ended_idx),
            }
        }

        match &tournament.events[11] {
            TournamentEvent::TournamentFinished(event) => {
                assert_eq!(event.winner, PlayerName::new("Alice"));
                assert_eq!(event.tournament_id, settings.tournament_id);
            }
            _ => panic!("Expected TournamentFinished event"),
        }

        // Build new tournament instance from events.
        let events = tournament.events.clone();
        let rebuilt_tournament = TournamentInstance::from(events);
        assert_eq!(rebuilt_tournament, tournament);
    }

    #[test]
    fn test_next_action() {
        let settings = TournamentSettings {
            tournament_id: TournamentId::random(),
            players: vec![Player::random("Alice"), Player::random("Bob")],
            starting_player_stack: 100.0,
            starting_small_blind: 5.0,
            double_blinds_every_n_games: None,
            end_condition: TournamentEndCondition::SingleWinner,
            see_historical_thoughts: false,
            public_chat: false,
        };

        let mut tournament = TournamentInstance::new(&settings);

        // Should suggest starting the first game
        if let Some(TournamentAction::StartNextGame { game_settings }) = tournament.next_action() {
            assert_eq!(game_settings.torunament_game_number, Some(0));
            assert_eq!(tournament.status(), &TournamentStatus::GameInProgress);
        } else {
            panic!("Expected StartNextGame action");
        }

        // Should suggest finishing the current game
        if let Some(TournamentAction::FinishGame { game_id }) = tournament.next_action() {
            assert_eq!(
                game_id,
                tournament.current_game_id.as_ref().unwrap().clone()
            );
        } else {
            panic!("Expected FinishGame action");
        }

        // Finish the game with Alice winning everything
        let finish_results = GameFinalResults {
            game_id: tournament.current_game_id.as_ref().unwrap().clone(),
            player_names: vec![PlayerName::new("Alice"), PlayerName::new("Bob")],
            final_stacks: vec![200.0, 0.0], // Alice wins all
        };
        tournament.finish_game(&finish_results).unwrap();

        // Tournament should be completed
        assert_eq!(tournament.status(), &TournamentStatus::Completed);
        assert!(tournament.next_action().is_none());
    }
}

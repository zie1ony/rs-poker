use rs_poker_types::{
    game::{GameFinalResults, GameId, GameSettings},
    tournament::{TournamentId, TournamentSettings, TournamentStatus},
    tournament_event::{TournamentCreatedEvent, TournamentEvent},
};

#[derive(Clone)]
pub struct TournamentInstance {
    pub tournament_id: TournamentId,
    pub events: Vec<TournamentEvent>,
    pub settings: TournamentSettings,
    pub status: TournamentStatus,
    pub next_game_number: usize,
    pub next_small_blind: f32,
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
        }
    }

    pub fn status(&self) -> &TournamentStatus {
        &self.status
    }

    pub fn next_action(&mut self) -> Option<TournamentAction> {
        match self.status {
            TournamentStatus::WaitingForNextGame => todo!(),
            TournamentStatus::GameInProgress => todo!(),
            TournamentStatus::Completed => todo!(),
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

        let new_game = GameSettings {
            tournament_id: Some(self.tournament_id.clone()),
            torunament_game_number: Some(game_number),
            game_id: GameId::random(),
            small_blind,
            players: self.settings.players.clone(),
            stacks: vec![self.settings.starting_player_stack; self.settings.players.len()],
        };

        self.status = TournamentStatus::GameInProgress;
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
        todo!()
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
    use rs_poker_types::player::{Player, PlayerName};

    use super::*;

    #[test]
    fn test_tournament_instance_creation() {
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
        };

        let mut t = TournamentInstance::new(&settings);

        // Check initial state.
        assert_eq!(t.status(), &TournamentStatus::WaitingForNextGame);

        // START GAME #0

        // Start game.
        let game0 = t.start_next_game().unwrap();
        assert_eq!(game0.tournament_id, Some(settings.tournament_id.clone()));
        assert_eq!(game0.torunament_game_number, Some(0));
        assert_eq!(game0.small_blind, 5.0);
        assert_eq!(game0.players, settings.players);
        assert_eq!(game0.stacks, vec![100.0, 100.0, 100.0]);

        // Check tournament state.
        assert_eq!(t.status(), &TournamentStatus::GameInProgress);

        // Should fail if we try to start another game without finishing the first.
        assert_eq!(
            t.start_next_game().unwrap_err(),
            TournamentError::CannotStartNewGame
        );

        // Should fail if we try to finish a game with a mismatched game ID.
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
            t.finish_game(&fake_results).unwrap_err(),
            TournamentError::GameIdMismatch
        );

        // Should fail if we try to finish a game with mismatched players.
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
            t.finish_game(&wrong_players_results).unwrap_err(),
            TournamentError::PlayersMismatch
        );

        // Should fail if we try to finish a game with mismatched number of players.
        let fewer_players_results = GameFinalResults {
            game_id: game0.game_id.clone(),
            player_names: vec![PlayerName::new("Alice"), PlayerName::new("Bob")],
            final_stacks: vec![150.0, 50.0],
        };
        assert_eq!(
            t.finish_game(&fewer_players_results).unwrap_err(),
            TournamentError::PlayersMismatch
        );
    }
}

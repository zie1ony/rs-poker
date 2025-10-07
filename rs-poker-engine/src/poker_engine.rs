use std::collections::HashMap;
use thiserror::Error;

use rs_poker_types::{game::{Decision, GameFullView, GameId, GameInfo, GamePlayerView, GameSettings}, player::PlayerName, tournament::TournamentId};

use crate::{game_instance::GameInstance, tournament_instance::TournamentInstance};

pub trait PokerEngineStorage: Send + Sync + 'static {
    fn clone_box(&self) -> Box<dyn PokerEngineStorage>;
    fn save_game(&mut self, game: &GameInstance);
    fn save_tournament(&mut self, tournament: &TournamentInstance);
    fn load_state(&self) -> (Vec<GameInstance>, Vec<TournamentInstance>);
}

#[derive(Error, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PokerEngineError {
    #[error("Game {0} already exists.")]
    GameNotFound(GameId),

    #[error("Game {0} already exists.")]
    GameAlreadyExists(GameId),

    #[error("Invalid game configuration.")]
    GameConfigurationError,

    #[error("Player {1} is not the current player in game {0}.")]
    WrongPlayer(GameId, PlayerName),

    #[error("Game {0} is not part of a tournament.")]
    GameNotPartOfTournament(GameId),

    #[error("Game {0} is already complete.")]
    GameAlreadyComplete(GameId),

    #[error("Tournament {0} not found.")]
    TournamentNotFound(TournamentId),

    #[error("Tournament {0} already exists.")]
    TournamentAlreadyExists(TournamentId),
}

pub type PokerEngineResult<T> = Result<T, PokerEngineError>;

#[derive(Default)]
pub struct PokerEngine {
    pub games: HashMap<GameId, GameInstance>,
    pub tournaments: HashMap<TournamentId, TournamentInstance>,
    pub storage: Option<Box<dyn PokerEngineStorage>>,
}

impl Clone for PokerEngine {
    fn clone(&self) -> Self {
        let storage = match &self.storage {
            Some(s) => Some(s.clone_box()),
            None => None,
        };
        Self {
            games: self.games.clone(),
            tournaments: self.tournaments.clone(),
            storage,
        }
    }
}

impl PokerEngine {

    // Constructors
    
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_storage(storage: Box<dyn PokerEngineStorage>) -> Self {
        let (games, tournaments) = storage.load_state();
        Self {
            games: games.into_iter().map(|g| (g.game_id(), g)).collect(),
            tournaments: tournaments
                .into_iter()
                .map(|t| (t.tournament_id(), t))
                .collect(),
            storage: Some(storage),
        }
    }

    // Game management
    
    pub fn game_new(&mut self, game_settings: GameSettings) -> PokerEngineResult<GameInfo> {
        // Validate if the game or tournament already exists.
        if let Some(game_id) = &game_settings.game_id {
            self.asset_game_does_not_exist(game_id)?;
        }
        if let Some(tournament_id) = &game_settings.tournament_id {
            self.assert_tournament_exists(tournament_id)?;
        }

        // Create a new game instance.
        let mut game = GameInstance::new(game_settings);
        game.run();

        self.set_game(game.clone());

        Ok(game.game_info())
    }

    pub fn game_info(&self, game_id: &GameId) -> PokerEngineResult<GameInfo> {
        let game = self.game(&game_id)?;
        Ok(GameInfo {
            game_id: game.game_id.clone(),
            players: game.players.clone(),
            status: game.game_status(),
            current_player_name: game.current_player_name(),
        })
    }

    pub fn game_list(&self, only_active: bool) -> Vec<GameInfo> {
        self.games
            .values()
            .filter(|g| !only_active || !g.is_complete())
            .map(|g| GameInfo {
                game_id: g.game_id.clone(),
                players: g.players.clone(),
                status: g.game_status(),
                current_player_name: g.current_player_name(),
            })
            .collect()
    }

    pub fn game_full_view(&self, game_id: &GameId) -> PokerEngineResult<GameFullView> {
        let game = self.game(game_id)?;
        Ok(game.as_game_full_view())
    }

    pub fn game_player_view(
        &self,
        game_id: &GameId,
        player_name: &PlayerName,
    ) -> PokerEngineResult<GamePlayerView> {
        let game = self.game(game_id)?;
        Ok(game.as_game_player_view(player_name))
    }

    pub fn game_make_action(
        &mut self,
        game_id: GameId,
        player_name: PlayerName,
        decision: Decision,
    ) -> PokerEngineResult<GameInfo> {
        let mut game = self.game(&game_id)?.clone();
        let game_info = game.game_info();

        // Check if the game is already complete.
        if game.is_complete() {
            return Err(PokerEngineError::GameAlreadyComplete(game_id.clone()));
        }

        // Check if the player is the current player.
        match game_info.current_player_name {
            Some(name) if name == player_name => (),
            _ => {
                return Err(PokerEngineError::WrongPlayer(
                    game_id.clone(),
                    player_name.clone(),
                ))
            }
        }
        
        // Apply the action.
        game.excute_player_action(decision);

        // Advance the game state.
        game.run();

        let game_info = game.game_info();

        self.set_game(game);

        // To

        Ok(game_info)
    }

    // Helpers

    fn game(&self, game_id: &GameId) -> PokerEngineResult<&GameInstance> {
        match self.games.get(game_id) {
            Some(game) => Ok(game),
            None => Err(PokerEngineError::GameNotFound(game_id.clone())),
        }
    }

    fn set_game(&mut self, game: GameInstance) {
        if let Some(storage) = &mut self.storage {
            storage.save_game(&game);
        }
        self.games.insert(game.game_id(), game);
    }

    fn asset_game_does_not_exist(&self, game_id: &GameId) -> PokerEngineResult<()> {
        if self.games.contains_key(game_id) {
            Err(PokerEngineError::GameAlreadyExists(game_id.clone()))
        } else {
            Ok(())
        }
    }

    fn assert_tournament_exists(&self, tournament_id: &TournamentId) -> PokerEngineResult<()> {
        if self.tournaments.contains_key(tournament_id) {
            Ok(())
        } else {
            Err(PokerEngineError::TournamentNotFound(tournament_id.clone()))
        }
    }
}

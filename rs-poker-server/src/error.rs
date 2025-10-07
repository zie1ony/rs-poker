use rs_poker_engine::poker_engine::PokerEngineError;
use rs_poker_types::{game::GameId, tournament::TournamentId};
use thiserror::Error;

#[derive(Error, Debug, serde::Serialize, serde::Deserialize)]
pub enum ServerError {
    #[error("Game {0:?} not found")]
    GameNotFound(GameId),

    #[error("Game {0:?} already exists")]
    GameAlreadyExists(GameId),

    #[error("Tournament {0:?} not found")]
    TournamentNotFound(TournamentId),

    #[error("Tournament {0:?} already exists")]
    TournamentAlreadyExists(TournamentId),

    #[error("Internal server error: {0}")]
    PokerEngineError(#[from] PokerEngineError),
}

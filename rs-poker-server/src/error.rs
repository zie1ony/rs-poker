use rs_poker_engine::poker_engine::PokerEngineError;
use rs_poker_types::tournament::TournamentId;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ServerError {
    #[error("Tournament {0:?} not found")]
    TournamentNotFound(TournamentId),

    #[error("Tournament {0:?} already exists")]
    TournamentAlreadyExists(TournamentId),

    #[error("Internal server error: {0}")]
    PokerEngineError(#[from] PokerEngineError),
}

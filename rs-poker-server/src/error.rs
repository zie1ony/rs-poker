use rs_poker_types::game::GameId;
use thiserror::Error;

#[derive(Error, Debug, serde::Serialize, serde::Deserialize)]
pub enum ServerError {
    #[error("Game {0:?} not found")]
    GameNotFound(GameId),

    #[error("Game {0:?} already exists")]
    GameAlreadyExists(GameId),
}

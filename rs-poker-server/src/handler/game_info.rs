use axum::{extract::{Query, State}, Json};
use rs_poker_types::game::{GameId, GameInfo};

use crate::{define_handler, error::ServerError, handler::HandlerResponse, poker_server::ServerState};

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
pub struct GameInfoRequest {
    pub game_id: GameId,
}

async fn game_info_handler(
    State(state): State<ServerState>,
    Query(params): Query<GameInfoRequest>,
) -> HandlerResponse<GameInfo> {
    let server = state.server.lock().unwrap();

    // Find the game instance.
    match server.games.get(&params.game_id) {
        Some(game) => Json(Ok(GameInfo {
            game_id: game.game_id.clone(),
            players: game.players.clone(),
            status: game.game_status(),
            current_player_name: game.current_player_name(),
        })),
        None => Json(Err(ServerError::GameNotFound(params.game_id.clone()))),
    }
}

define_handler!(
    GameInfoHandler {
        Request = GameInfoRequest;
        Response = GameInfo;
        Method = GET;
        Path = "/game_info";
        FN = game_info_handler;
    }
);

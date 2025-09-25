use axum::{extract::{Query, State}, Json};
use rs_poker_types::{game::{GameId, GamePlayerView}, player::PlayerName};

use crate::{define_handler, error::ServerError, handler::HandlerResponse, poker_server::ServerState};

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
pub struct GamePlayerViewRequest {
    pub game_id: GameId,
    pub player_name: PlayerName,
}

async fn game_player_view_handler(
    State(state): State<ServerState>,
    Query(params): Query<GamePlayerViewRequest>,
) -> HandlerResponse<GamePlayerView> {
    let server = state.server.lock().unwrap();

    // Find the game instance.
    match server.games.get(&params.game_id) {
        Some(game) => Json(Ok(game.as_game_player_view(&params.player_name))),
        None => Json(Err(ServerError::GameNotFound(params.game_id.clone()))),
    }
}

define_handler!(
    GamePlayerViewHandler {
        Request = GamePlayerViewRequest;
        Response = GamePlayerView;
        Method = GET;
        Path = "/game_player_view";
        FN = game_player_view_handler;
    }
);

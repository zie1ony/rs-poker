use axum::{
    extract::{Query, State},
    Json,
};
use rs_poker_types::game::{GameFullView, GameId};

use crate::{
    define_handler, error::ServerError, handler::HandlerResponse, poker_server::ServerState,
};

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
pub struct GameFullViewRequest {
    pub game_id: GameId,
    pub debug: bool,
}

async fn game_full_view_handler(
    State(state): State<ServerState>,
    Query(params): Query<GameFullViewRequest>,
) -> HandlerResponse<GameFullView> {
    let server = state.server.lock().unwrap();

    // Find the game instance.
    match server.games.get(&params.game_id) {
        Some(game) => {
            let mut view = game.as_game_full_view();
            if params.debug {
                view.summary.push_str("\n\n [Debug Info]\n");
                view.summary.push_str(game.actions_str().as_str());
            }
            Json(Ok(view))
        }
        None => Json(Err(ServerError::GameNotFound(params.game_id.clone()))),
    }
}

define_handler!(
    GameFullViewHandler {
        Request = GameFullViewRequest;
        Response = GameFullView;
        Method = GET;
        Path = "/game_full_view";
        FN = game_full_view_handler;
    }
);

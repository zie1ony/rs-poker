use axum::{
    extract::{Query, State},
    Json,
};
use rs_poker_types::game::GameStatus;

use crate::{define_handler, handler::HandlerResponse, poker_server::ServerState};

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
pub struct ListGamesRequest {
    pub active_only: bool,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
pub struct ListGamesResponse {
    pub game_ids: Vec<(String, GameStatus)>,
}

async fn list_games_handler(
    State(state): State<ServerState>,
    Query(params): Query<ListGamesRequest>,
) -> HandlerResponse<ListGamesResponse> {
    let server = state.server.lock().unwrap();

    let game_ids: Vec<(String, rs_poker_types::game::GameStatus)> = server
        .games
        .iter()
        .filter_map(|(game_id, game)| {
            let status = game.game_status();
            if params.active_only && status == rs_poker_types::game::GameStatus::Finished {
                None
            } else {
                Some((game_id.to_string(), status))
            }
        })
        .collect();

    Json(Ok(ListGamesResponse { game_ids }))
}

define_handler!(
    ListGamesHandler {
        Request = ListGamesRequest;
        Response = ListGamesResponse;
        Method = GET;
        Path = "/list_games";
        FN = list_games_handler;
    }
);

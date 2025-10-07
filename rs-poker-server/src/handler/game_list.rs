use axum::{
    extract::{Query, State},
};
use rs_poker_types::game::{GameInfo};

use crate::{define_handler, handler::{response, HandlerResponse}, poker_server::ServerState};

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
pub struct ListGamesRequest {
    pub active_only: bool,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
pub struct ListGamesResponse {
    pub list: Vec<GameInfo>
}

async fn list_games_handler(
    State(state): State<ServerState>,
    Query(params): Query<ListGamesRequest>,
) -> HandlerResponse<ListGamesResponse> {
    let engine = state.engine.lock().unwrap();
    let list = engine.game_list(params.active_only);
    response(Ok(ListGamesResponse { list }))
}

define_handler!(
    ListGamesHandler {
        Request = ListGamesRequest;
        Response = ListGamesResponse;
        Method = GET;
        Path = "/game/list";
        FN = list_games_handler;
    }
);

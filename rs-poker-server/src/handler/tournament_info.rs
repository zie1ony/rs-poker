use axum::{
    extract::{Query, State},
    Json,
};
use rs_poker_types::tournament::{TournamentId, TournamentInfo};

use crate::{
    define_handler, error::ServerError, handler::HandlerResponse, poker_server::ServerState,
};

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
pub struct TournamentInfoRequest {
    pub tournament_id: TournamentId,
}

async fn tournament_info_handler(
    State(state): State<ServerState>,
    Query(params): Query<TournamentInfoRequest>,
) -> HandlerResponse<TournamentInfo> {
    let server = state.engine.lock().unwrap();
    let tournament_id = params.tournament_id;
    match server.tournaments.get(&tournament_id) {
        Some(tournament) => {
            let info = tournament.info();
            Json(Ok(info))
        }
        None => Json(Err(ServerError::TournamentNotFound(tournament_id))),
    }
}

define_handler!(
    TournamentInfoHandler {
        Request = TournamentInfoRequest;
        Response = TournamentInfo;
        Method = GET;
        Path = "/tournament_info";
        FN = tournament_info_handler;
    }
);

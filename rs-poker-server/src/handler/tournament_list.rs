use axum::{
    extract::{Query, State},
    Json,
};
use rs_poker_types::tournament::{TournamentId, TournamentStatus};

use crate::{define_handler, handler::HandlerResponse, poker_server::ServerState};

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
pub struct ListTournamentsRequest {
    pub active_only: bool,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
pub struct ListTournamentsResponse {
    pub tournament_ids: Vec<(TournamentId, TournamentStatus)>,
}

async fn list_tournaments_handler(
    State(state): State<ServerState>,
    Query(params): Query<ListTournamentsRequest>,
) -> HandlerResponse<ListTournamentsResponse> {
    let server = state.server.lock().unwrap();

    let tournament_ids: Vec<(TournamentId, TournamentStatus)> = server
        .tournaments
        .iter()
        .filter_map(|(tournament_id, tournament)| {
            let status = tournament.status().clone();
            if params.active_only && status == TournamentStatus::Completed {
                None
            } else {
                Some((tournament_id.clone(), status))
            }
        })
        .collect();

    Json(Ok(ListTournamentsResponse { tournament_ids }))
}

define_handler!(
    ListTournamentsHandler {
        Request = ListTournamentsRequest;
        Response = ListTournamentsResponse;
        Method = GET;
        Path = "/list_tournaments";
        FN = list_tournaments_handler;
    }
);

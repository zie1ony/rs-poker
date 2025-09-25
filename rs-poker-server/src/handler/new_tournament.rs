use axum::{extract::State, Json};
use rs_poker_engine::tournament_instance::TournamentInstance;
use rs_poker_types::tournament::{TournamentId, TournamentSettings};

use crate::{define_handler, error::ServerError, handler::HandlerResponse, poker_server::ServerState};

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
pub struct NewTournamentRequest {
    pub settings: TournamentSettings,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
pub struct TournamentCreatedResponse {
    pub tournament_id: TournamentId,
}

async fn new_tournament_handler(
    State(state): State<ServerState>,
    Json(payload): Json<NewTournamentRequest>,
) -> HandlerResponse<TournamentCreatedResponse> {
    let mut server = state.server.lock().unwrap();
    let settings = payload.settings;

    // Fail if the tournament ID already exists.
    if server.tournaments.contains_key(&settings.tournament_id) {
        return Json(Err(ServerError::TournamentAlreadyExists(
            settings.tournament_id,
        )));
    }

    // Create a new tournament instance.
    let tournament = TournamentInstance::new(&settings);

    server
        .tournaments
        .insert(settings.tournament_id.clone(), tournament);

    Json(Ok(TournamentCreatedResponse {
        tournament_id: settings.tournament_id.clone(),
    }))
}

define_handler!(
    NewTournamentHandler {
        Request = NewTournamentRequest;
        Response = TournamentCreatedResponse;
        Method = POST;
        Path = "/new_tournament";
        FN = new_tournament_handler;
    }
);

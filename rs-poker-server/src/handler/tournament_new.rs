use axum::{extract::State, Json};
use rs_poker_engine::tournament_instance::TournamentInstance;
use rs_poker_types::tournament::{TournamentId, TournamentSettings};

use crate::{
    define_handler, error::ServerError, handler::HandlerResponse, poker_server::ServerState,
};

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
pub struct TournamentCreatedResponse {
    pub tournament_id: TournamentId,
}

async fn new_tournament_handler(
    State(state): State<ServerState>,
    Json(settings): Json<TournamentSettings>,
) -> HandlerResponse<TournamentCreatedResponse> {
    let mut server = state.engine.lock().unwrap();
    let tournament_id = settings.tournament_id.clone();

    // Fail if the tournament ID already exists.
    if server.tournaments.contains_key(&tournament_id) {
        return Json(Err(ServerError::TournamentAlreadyExists(tournament_id)));
    }

    // Create a new tournament instance.
    let tournament = TournamentInstance::new(&settings);

    // // Store it.
    // server.update_tournament(&tournament);

    // // Start playing.
    // server.progress_tournament(&tournament_id);

    Json(Ok(TournamentCreatedResponse {
        tournament_id: settings.tournament_id.clone(),
    }))
}

define_handler!(
    NewTournamentHandler {
        Request = TournamentSettings;
        Response = TournamentCreatedResponse;
        Method = POST;
        Path = "/new_tournament";
        FN = new_tournament_handler;
    }
);

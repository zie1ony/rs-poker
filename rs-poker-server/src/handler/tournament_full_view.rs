use axum::{
    extract::{Query, State},
    Json,
};
use rs_poker_engine::tournament_summary::TournamentSummary;
use rs_poker_types::tournament::TournamentId;

use crate::{
    define_handler, error::ServerError, handler::HandlerResponse, poker_server::ServerState,
};

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
pub struct TournamentFullViewRequest {
    pub tournament_id: TournamentId,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
pub struct TournamentFullViewResponse {
    pub summary: String,
}

async fn tournament_full_view_handler(
    State(state): State<ServerState>,
    Query(params): Query<TournamentFullViewRequest>,
) -> HandlerResponse<TournamentFullViewResponse> {
    let server = state.engine.lock().unwrap();
    let tournament_id = params.tournament_id;
    match server.tournaments.get(&tournament_id) {
        Some(tournament) => {
            // Get the tournament events
            let tournament_events = tournament.events.clone();

            // Collect game events from all games associated with this tournament
            let mut game_events = std::collections::HashMap::new();
            for game_id in tournament.game_ids() {
                if let Some(game) = server.games.get(game_id) {
                    game_events.insert(game_id.clone(), game.simulation.events.clone());
                }
            }

            // Create full tournament summary
            let tournament_summary = TournamentSummary::full(tournament_events, game_events);
            let summary = tournament_summary.summary();

            Json(Ok(TournamentFullViewResponse { summary }))
        }
        None => Json(Err(ServerError::TournamentNotFound(tournament_id))),
    }
}

define_handler!(
    TournamentFullViewHandler {
        Request = TournamentFullViewRequest;
        Response = TournamentFullViewResponse;
        Method = GET;
        Path = "/tournament_full_view";
        FN = tournament_full_view_handler;
    }
);

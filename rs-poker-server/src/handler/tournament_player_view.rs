use axum::{
    extract::{Query, State},
    Json,
};
use rs_poker_engine::tournament_summary::TournamentSummary;
use rs_poker_types::{player::PlayerName, tournament::TournamentId};

use crate::{
    define_handler, error::ServerError, handler::HandlerResponse, poker_server::ServerState,
};

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
pub struct TournamentPlayerViewRequest {
    pub tournament_id: TournamentId,
    pub player_name: PlayerName,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
pub struct TournamentPlayerViewResponse {
    pub summary: String,
}

async fn tournament_player_view_handler(
    State(state): State<ServerState>,
    Query(params): Query<TournamentPlayerViewRequest>,
) -> HandlerResponse<TournamentPlayerViewResponse> {
    let server = state.server.lock().unwrap();
    let tournament_id = params.tournament_id;
    match server.tournaments.get(&tournament_id) {
        Some(tournament) => {
            // Get the tournament events
            let tournament_events = tournament.events.clone();
            let player_name = params.player_name;

            // Collect game events from all games associated with this tournament
            let mut game_events = std::collections::HashMap::new();
            for game_id in tournament.game_ids() {
                if let Some(game) = server.games.get(game_id) {
                    game_events.insert(game_id.clone(), game.simulation.events.clone());
                }
            }

            // Create player-specific tournament summary
            let tournament_summary =
                TournamentSummary::for_player(tournament_events, game_events, player_name);
            let summary = tournament_summary.summary();

            Json(Ok(TournamentPlayerViewResponse { summary }))
        }
        None => Json(Err(ServerError::TournamentNotFound(tournament_id))),
    }
}

define_handler!(
    TournamentPlayerViewHandler {
        Request = TournamentPlayerViewRequest;
        Response = TournamentPlayerViewResponse;
        Method = GET;
        Path = "/tournament_player_view";
        FN = tournament_player_view_handler;
    }
);

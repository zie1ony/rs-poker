use axum::{
    extract::{Query, State},
    Json,
};
use rs_poker_engine::{game_summary::GameSummary, tournament_summary::TournamentSummary};
use rs_poker_types::{
    game::{GameId, GamePlayerView},
    player::PlayerName,
};

use crate::{
    define_handler, error::ServerError, handler::HandlerResponse, poker_server::ServerState,
};

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
pub struct GamePlayerViewRequest {
    pub game_id: GameId,
    pub player_name: PlayerName,
    pub include_tournament_history: bool,
}

async fn game_player_view_handler(
    State(state): State<ServerState>,
    Query(params): Query<GamePlayerViewRequest>,
) -> HandlerResponse<GamePlayerView> {
    let server = state.server.lock().unwrap();

    // Find the game instance.
    match server.games.get(&params.game_id) {
        Some(game) => {
            let mut game_view = game.as_game_player_view(&params.player_name);
            if params.include_tournament_history {
                if let Some(tournament_id) = &game.tournament_id {
                    if let Some(tournament) = server.tournaments.get(&tournament_id) {
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
                        game_view.summary = tournament_summary.summary();
                    }
                }
            }

            Json(Ok(game_view))
        },
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

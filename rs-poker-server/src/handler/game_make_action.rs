use axum::{extract::State, Json};
use rs_poker_types::game::{Decision, GameId, GameInfo};

use crate::{
    define_handler, error::ServerError, handler::HandlerResponse, poker_server::ServerState,
};

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct MakeActionRequest {
    pub game_id: GameId,
    pub decision: Decision,
}

async fn make_action_handler(
    State(state): State<ServerState>,
    Json(payload): Json<MakeActionRequest>,
) -> HandlerResponse<GameInfo> {
    let mut server = state.server.lock().unwrap();

    // Find the game instance.
    match server.games.get_mut(&payload.game_id) {
        Some(game) => {
            // Apply the action.
            game.excute_player_action(payload.decision);
            // Advance the game state.
            game.run();

            let is_complete = game.is_complete();
            let game_info = GameInfo {
                game_id: game.game_id.clone(),
                players: game.players.clone(),
                status: game.game_status(),
                current_player_name: game.current_player_name(),
            };

            // If the game is complete and is part of a tournament, progress the tournament.
            if is_complete {
                if let Some(tournament_id) = game.tournament_id.clone() {
                    server.progress_tournament(&tournament_id);
                }
            }

            Json(Ok(game_info))
        }
        None => Json(Err(ServerError::GameNotFound(payload.game_id.clone()))),
    }
}

define_handler!(
    MakeActionHandler {
        Request = MakeActionRequest;
        Response = GameInfo;
        Method = POST;
        Path = "/make_action";
        FN = make_action_handler;
    }
);

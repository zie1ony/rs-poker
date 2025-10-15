use axum::{extract::State, Json};
use rs_poker_types::game::{GameInfo, GameSettings};

use crate::{
    define_handler,
    handler::{response, HandlerResponse},
    poker_client::{ClientResult, PokerClient},
    poker_server::ServerState,
};

async fn new_game_handler(
    State(state): State<ServerState>,
    Json(payload): Json<GameSettings>,
) -> HandlerResponse<GameInfo> {
    let game_info = {
        let mut engine = state.engine.lock().unwrap();
        engine.game_new(payload.clone())
    };

    // Broadcast new game info to all subscribers if game was created successfully
    if let Ok(ref info) = game_info {
        if let Some(ref game_id) = payload.game_id {
            state
                .game_subscribers
                .broadcast_game_info(game_id, info)
                .await;
        }
    }

    response(game_info)
}

define_handler!(
    NewGameHandler {
        Request = GameSettings;
        Response = GameInfo;
        Method = POST;
        Path = "/game/new";
        FN = new_game_handler;
    }
);

impl PokerClient {
    pub async fn new_game(&self, settings: &GameSettings) -> ClientResult<GameInfo> {
        self.query::<NewGameHandler>(settings.clone()).await
    }
}

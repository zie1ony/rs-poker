use axum::{extract::State, Json};
use rs_poker_types::game::{GameId, GameInfo, GameSettings};

use crate::{
    define_handler,
    error::ServerError,
    handler::{response, HandlerResponse},
    poker_client::{ClientResult, PokerClient},
    poker_server::ServerState,
};

async fn new_game_handler(
    State(state): State<ServerState>,
    Json(payload): Json<GameSettings>,
) -> HandlerResponse<GameInfo> {
    let mut engine = state.engine.lock().unwrap();
    response(engine.game_new(payload))
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

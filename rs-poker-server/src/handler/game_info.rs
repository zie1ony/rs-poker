use axum::{
    extract::{Query, State},
};
use rs_poker_types::game::{GameId, GameInfo};

use crate::{
    define_handler,
    handler::{response, HandlerResponse},
    poker_client::{ClientResult, PokerClient},
    poker_server::ServerState,
};

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
pub struct GameInfoRequest {
    pub game_id: GameId,
}

async fn game_info_handler(
    State(state): State<ServerState>,
    Query(params): Query<GameInfoRequest>,
) -> HandlerResponse<GameInfo> {
    let engine = state.engine.lock().unwrap();
    response(engine.game_info(&params.game_id))
}

define_handler!(
    GameInfoHandler {
        Request = GameInfoRequest;
        Response = GameInfo;
        Method = GET;
        Path = "/game/info";
        FN = game_info_handler;
    }
);

impl PokerClient {
    pub async fn game_info(&self, game_id: &GameId) -> ClientResult<GameInfo> {
        self.query::<GameInfoHandler>(GameInfoRequest {
            game_id: game_id.clone(),
        })
        .await
    }
}

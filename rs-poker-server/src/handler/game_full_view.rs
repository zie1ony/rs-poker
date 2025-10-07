use axum::{
    extract::{Query, State},
};
use rs_poker_types::game::{GameFullView, GameId};

use crate::{
    define_handler, handler::{response, HandlerResponse}, poker_client::{ClientResult, PokerClient}, poker_server::ServerState
};

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
pub struct GameFullViewRequest {
    pub game_id: GameId,
}

async fn game_full_view_handler(
    State(state): State<ServerState>,
    Query(params): Query<GameFullViewRequest>,
) -> HandlerResponse<GameFullView> {
    let engine = state.engine.lock().unwrap();
    response(engine.game_full_view(&params.game_id))
}

define_handler!(
    GameFullViewHandler {
        Request = GameFullViewRequest;
        Response = GameFullView;
        Method = GET;
        Path = "/game/full_view";
        FN = game_full_view_handler;
    }
);

impl PokerClient {
    pub async fn game_full_view(&self, game_id: &GameId) -> ClientResult<GameFullView> {
        self.query::<GameFullViewHandler>(GameFullViewRequest { game_id: game_id.clone() }).await
    }
}

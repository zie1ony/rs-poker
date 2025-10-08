use axum::extract::{Query, State};
use rs_poker_types::{
    game::{GameId, GamePlayerView},
    player::PlayerName,
};

use crate::{
    define_handler,
    handler::{response, HandlerResponse},
    poker_client::{ClientResult, PokerClient},
    poker_server::ServerState,
};

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
pub struct GamePlayerViewRequest {
    pub game_id: GameId,
    pub player_name: PlayerName,
}

async fn game_player_view_handler(
    State(state): State<ServerState>,
    Query(params): Query<GamePlayerViewRequest>,
) -> HandlerResponse<GamePlayerView> {
    let engine = state.engine.lock().unwrap();
    response(engine.game_player_view(&params.game_id, &params.player_name))
}

define_handler!(
    GamePlayerViewHandler {
        Request = GamePlayerViewRequest;
        Response = GamePlayerView;
        Method = GET;
        Path = "/game/player_view";
        FN = game_player_view_handler;
    }
);

impl PokerClient {
    pub async fn game_player_view(
        &self,
        params: GamePlayerViewRequest,
    ) -> ClientResult<GamePlayerView> {
        self.query::<GamePlayerViewHandler>(params).await
    }
}

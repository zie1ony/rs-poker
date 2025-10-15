use axum::{extract::State, Json};
use rs_poker_types::{
    game::{Decision, GameId, GameInfo},
    player::PlayerName,
};

use crate::{
    define_handler,
    handler::{response, HandlerResponse},
    poker_client::{ClientResult, PokerClient},
    poker_server::ServerState,
};

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct MakeActionRequest {
    pub game_id: GameId,
    pub player_name: PlayerName,
    pub decision: Decision,
}

async fn make_action_handler(
    State(state): State<ServerState>,
    Json(payload): Json<MakeActionRequest>,
) -> HandlerResponse<GameInfo> {
    let game_info = {
        let mut engine = state.engine.lock().unwrap();
        engine.game_make_action(
            payload.game_id.clone(),
            payload.player_name,
            payload.decision,
        )
    };

    // Broadcast game update to all subscribers if action was successful
    if let Ok(ref info) = game_info {
        state
            .game_subscribers
            .broadcast_game_info(&payload.game_id, info)
            .await;
    }

    response(game_info)
}

define_handler!(
    MakeActionHandler {
        Request = MakeActionRequest;
        Response = GameInfo;
        Method = POST;
        Path = "/game/make_action";
        FN = make_action_handler;
    }
);

impl PokerClient {
    pub async fn make_action(&self, decision: MakeActionRequest) -> ClientResult<GameInfo> {
        self.query::<MakeActionHandler>(decision).await
    }
}

use axum::{extract::State, Json};
use rs_poker::core::Card;
use rs_poker_engine::game_instance::GameInstance;
use rs_poker_types::{game::GameId, player::Player};

use crate::{
    define_handler, error::ServerError, handler::HandlerResponse, poker_server::ServerState,
};

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
pub struct NewGameRequest {
    pub game_id: GameId,
    pub players: Vec<Player>,
    pub small_blind: f32,
    pub initial_stacks: Vec<f32>,
    pub predefined_hands: Option<Vec<(Card, Card)>>,
    pub predefined_board: Option<Vec<Card>>,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
pub struct GameCreatedResponse {
    pub game_id: GameId,
}

async fn new_game_handler(
    State(state): State<ServerState>,
    Json(payload): Json<NewGameRequest>,
) -> HandlerResponse<GameCreatedResponse> {
    let mut server = state.server.lock().unwrap();

    // Fail if the game ID already exists.
    if server.games.contains_key(&payload.game_id) {
        return Json(Err(ServerError::GameAlreadyExists(payload.game_id)));
    }

    // Create a new game instance.
    let mut game = GameInstance::new_with_random_cards(
        payload.game_id.clone(),
        None,
        payload.players.clone(),
        payload.initial_stacks.clone(),
        payload.small_blind * 2.0,
        payload.small_blind,
    );
    game.run();

    server.games.insert(payload.game_id.clone(), game);

    Json(Ok(GameCreatedResponse {
        game_id: payload.game_id.clone(),
    }))
}

define_handler!(
    NewGameHandler {
        Request = NewGameRequest;
        Response = GameCreatedResponse;
        Method = POST;
        Path = "/new_game";
        FN = new_game_handler;
    }
);

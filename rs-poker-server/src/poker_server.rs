use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    api_types::{
        GameCreatedResponse, GameFullViewRequest, GameInfoRequest, GamePlayerViewRequest,
        HealthCheckRequest, HealthCheckResponse, ListGamesRequest, ListGamesResponse,
        MakeActionRequest, NewGameRequest, NewTournamentRequest, ServerResponse,
        TournamentCreatedResponse,
    },
    error::ServerError,
};
use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};
use rs_poker_engine::{game_instance::GameInstance, tournament_instance::TournamentInstance};
use rs_poker_types::{
    game::{GameFullView, GameId, GameInfo, GamePlayerView},
    tournament::TournamentId,
};

#[derive(Clone, Default)]
pub struct PokerServer {
    pub games: HashMap<GameId, GameInstance>,
    pub tournaments: HashMap<TournamentId, TournamentInstance>,
}

#[derive(Clone)]
pub struct ServerState {
    pub server: Arc<Mutex<PokerServer>>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            server: Arc::new(Mutex::new(PokerServer::default())),
        }
    }
}

/// Having a function that produces our app makes it easy to call it from tests
/// without having to create an HTTP server.
pub fn app() -> Router {
    Router::new()
        .route("/health_check", get(health_check_handler))
        // Game.
        .route("/new_game", post(new_game_handler))
        .route("/list_games", get(list_games_handler))
        .route("/game_full_view", get(game_full_view_handler))
        .route("/game_player_view", get(game_player_view_handler))
        .route("/game_info", get(game_info_handler))
        .route("/make_action", post(make_action_handler))
        // Tournament.
        .route("/new_tournament", post(new_tournament_handler))
        .with_state(ServerState::new())
}

async fn health_check_handler(
    Query(params): Query<HealthCheckRequest>,
) -> ServerResponse<HealthCheckResponse> {
    Json(Ok(HealthCheckResponse {
        id: params.id,
        status: "ok".to_string(),
    }))
}

// --- game handlers ---

async fn new_game_handler(
    State(state): State<ServerState>,
    Json(payload): Json<NewGameRequest>,
) -> ServerResponse<GameCreatedResponse> {
    let mut server = state.server.lock().unwrap();

    // Fail if the game ID already exists.
    if server.games.contains_key(&payload.game_id) {
        return Json(Err(ServerError::GameAlreadyExists(payload.game_id)));
    }

    // Create a new game instance.
    let mut game = GameInstance::new_with_random_cards(
        payload.game_id.clone(),
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

async fn list_games_handler(
    State(state): State<ServerState>,
    Query(params): Query<ListGamesRequest>,
) -> ServerResponse<ListGamesResponse> {
    let server = state.server.lock().unwrap();

    let game_ids: Vec<(String, rs_poker_types::game::GameStatus)> = server
        .games
        .iter()
        .filter_map(|(game_id, game)| {
            let status = game.game_status();
            if params.active_only && status == rs_poker_types::game::GameStatus::Finished {
                None
            } else {
                Some((game_id.to_string(), status))
            }
        })
        .collect();

    Json(Ok(ListGamesResponse { game_ids }))
}

async fn game_full_view_handler(
    State(state): State<ServerState>,
    Query(params): Query<GameFullViewRequest>,
) -> ServerResponse<GameFullView> {
    let server = state.server.lock().unwrap();

    // Find the game instance.
    match server.games.get(&params.game_id) {
        Some(game) => {
            let mut view = game.as_game_full_view();
            if params.debug {
                view.summary.push_str("\n\n [Debug Info]\n");
                view.summary.push_str(game.actions_str().as_str());
            }
            Json(Ok(view))
        }
        None => Json(Err(ServerError::GameNotFound(params.game_id.clone()))),
    }
}

async fn game_player_view_handler(
    State(state): State<ServerState>,
    Query(params): Query<GamePlayerViewRequest>,
) -> ServerResponse<GamePlayerView> {
    let server = state.server.lock().unwrap();

    // Find the game instance.
    match server.games.get(&params.game_id) {
        Some(game) => Json(Ok(game.as_game_player_view(&params.player_name))),
        None => Json(Err(ServerError::GameNotFound(params.game_id.clone()))),
    }
}

async fn game_info_handler(
    State(state): State<ServerState>,
    Query(params): Query<GameInfoRequest>,
) -> ServerResponse<GameInfo> {
    let server = state.server.lock().unwrap();

    // Find the game instance.
    match server.games.get(&params.game_id) {
        Some(game) => Json(Ok(GameInfo {
            game_id: game.game_id.clone(),
            players: game.players.clone(),
            status: game.game_status(),
            current_player_name: game.current_player_name(),
        })),
        None => Json(Err(ServerError::GameNotFound(params.game_id.clone()))),
    }
}

async fn make_action_handler(
    State(state): State<ServerState>,
    Json(payload): Json<MakeActionRequest>,
) -> ServerResponse<GameInfo> {
    let mut server = state.server.lock().unwrap();

    // Find the game instance.
    match server.games.get_mut(&payload.game_id) {
        Some(game) => {
            // Apply the action.
            game.excute_player_action(payload.decision);
            // Advance the game state.
            game.run();

            Json(Ok(GameInfo {
                game_id: game.game_id.clone(),
                players: game.players.clone(),
                status: game.game_status(),
                current_player_name: game.current_player_name(),
            }))
        }
        None => Json(Err(ServerError::GameNotFound(payload.game_id.clone()))),
    }
}

// --- tournament handlers ---

async fn new_tournament_handler(
    State(state): State<ServerState>,
    Json(payload): Json<NewTournamentRequest>,
) -> ServerResponse<TournamentCreatedResponse> {
    let mut server = state.server.lock().unwrap();
    let settings = payload.settings;

    // Fail if the tournament ID already exists.
    if server.tournaments.contains_key(&settings.tournament_id) {
        return Json(Err(ServerError::TournamentAlreadyExists(
            settings.tournament_id,
        )));
    }

    // Create a new tournament instance.
    let tournament = TournamentInstance::new(&settings);

    server
        .tournaments
        .insert(settings.tournament_id.clone(), tournament);

    Json(Ok(TournamentCreatedResponse {
        tournament_id: settings.tournament_id.clone(),
    }))
}

#[cfg(test)]
mod tests {

    // #[tokio::test]
    // async fn as_json_with_client() {
    //     let app = app();
    //     let client = PokerClient::new_test(app);

    //     let payload = serde_json::json!({"test": "data"});
    //     let result = client.as_json(payload.clone()).await.unwrap();

    //     let expected = payload;
    //     assert_eq!(result, expected);
    // }

    // #[tokio::test]
    // async fn health_check_with_client() {
    //     let app = app();
    //     let client = PokerClient::new_test(app);

    //     let request = HealthCheckRequest {
    //         id: "test-123".to_string(),
    //     };
    //     let result = client.health_check(request).await.unwrap();

    //     assert_eq!(result.id, "test-123");
    //     assert_eq!(result.status, "ok");
    // }

    // #[tokio::test]
    // async fn new_game_creation() {
    //     let app = app();
    //     let client = PokerClient::new_test(app);

    //     // Define players.
    //     let player_ai = Player::AI {
    //         name: PlayerName::new("Adi"),
    //         model: String::from("gpt-4o-mini"),
    //         strategy: String::from("Play tight aggressive"),
    //     };
    //     let player_human = Player::Human {
    //         name: PlayerName::new("Bid"),
    //     };
    //     let player_auto_1 = Player::Automat {
    //         name: PlayerName::new("Cici"),
    //         automat_type: AutomatType::AllIn,
    //     };
    //     let player_auto_2 = Player::Automat {
    //         name: PlayerName::new("Dedi"),
    //         automat_type: AutomatType::Calling,
    //     };

    //     // Create new game.
    //     let game_id = String::from("game1");
    //     let request = NewGameRequest {
    //         game_id: game_id.clone(),
    //         players: vec![player_ai, player_human, player_auto_1,
    // player_auto_2],         small_blind: 5.0,
    //         initial_stacks: vec![100.0, 100.0, 100.0, 100.0],
    //         predefined_hands: Some(vec![
    //             (card("Ah"), card("Ad")),
    //             (card("Kh"), card("Kd")),
    //             (card("Qh"), card("Qd")),
    //             (card("Jh"), card("Jd")),
    //         ]),
    //         predefined_board: Some(vec![
    //             card("Ac"),
    //             card("As"),
    //             card("2d"),
    //             card("2h"),
    //             card("3c"),
    //         ]),
    //     };
    //     let response = client.new_game(request).await.unwrap();
    //     let expected = GameStatusResponse {
    //         game_id: game_id.clone(),
    //         status: GameStatus::InProgress,
    //         next_player: Some(String::from("Adi")),
    //         actions: vec![],
    //     };
    //     assert_eq!(response, expected);

    //     // At this point it is expected:
    //     // - that game stared, and first player
    //     // - Adi was forced to play small blind,
    //     // - Bid was forced to play big blind,
    //     // - Cici played automatically (AllIn),
    //     // - Dedi played automatically (Calling),
    //     // - Next player is Adi.

    //     // Load the game state for Adi.
    //     let next_action_request = NextActionInfoRequest {
    //         game_id: game_id.clone(),
    //     };
    //     let next_action_info = client
    //         .next_action_info(next_action_request.clone())
    //         .await
    //         .unwrap();
    //     let expected_next_action_info = GameViewForPlayer {
    //         game_id: game_id.clone(),
    //         current_player: PlayerName::new("Adi"),
    //         your_hands: (card("Ah"), card("Ad")),
    //         community_cards: vec![],
    //         round: Round::Preflop,
    //         pot: 5.0 + 10.0 + 100.0 + 100.0,
    //         round_actions: vec![
    //             PlayerAction::bet("Adi", 5.0),
    //             PlayerAction::bet("Bid", 10.0),
    //             PlayerAction::all_in("Cici"),
    //             PlayerAction::call("Dedi"),
    //         ],
    //         possible_actions: vec![
    //             PossibleAction::Fold,
    //             PossibleAction::Call,
    //             PossibleAction::Bet {
    //                 min: 100.0,
    //                 max: 100.0,
    //             },
    //             PossibleAction::AllIn,
    //         ],
    //     };
    //     assert_eq!(next_action_info, expected_next_action_info);

    //     // Adi decides to go AllIn.
    //     let adi_decision = PlayerDecision {
    //         player_name: PlayerName::new("Adi"),
    //         action: AgentAction::AllIn,
    //         reason: String::from("Feeling lucky!"),
    //     };
    //     let response = client
    //         .make_decision(MakeActionRequest {
    //             game_id: game_id.clone(),
    //             decision: adi_decision,
    //         })
    //         .await
    //         .unwrap();
    //     let expected = GameStatusResponse {
    //         game_id: game_id.clone(),
    //         status: GameStatus::InProgress,
    //         next_player: Some(String::from("Bid")),
    //     };
    //     assert_eq!(response, expected);

    //     // Load the game state for Bid.
    //     let next_action_info =
    // client.next_action_info(next_action_request).await.unwrap();
    //     let expected_next_action_info = GameViewForPlayer {
    //         game_id: game_id.clone(),
    //         current_player: PlayerName::new("Bid"),
    //         your_hands: (card("Kh"), card("Kd")),
    //         community_cards: vec![],
    //         round: Round::Preflop,
    //         pot: 5.0 + 10.0 + 100.0 + 100.0 + 95.0,
    //         round_actions: vec![
    //             PlayerAction::bet("Adi", 5.0),
    //             PlayerAction::bet("Bid", 10.0),
    //             PlayerAction::all_in("Cici"),
    //             PlayerAction::call("Dedi"),
    //             PlayerAction::all_in("Adi"),
    //         ],
    //         possible_actions: vec![
    //             PossibleAction::Fold,
    //             PossibleAction::Call,
    //             PossibleAction::Bet {
    //                 min: 90.0,
    //                 max: 90.0,
    //             },
    //             PossibleAction::AllIn,
    //         ],
    //     };
    //     assert_eq!(next_action_info, expected_next_action_info);

    //     // Bid decides to Fold.
    //     let bid_decision = PlayerDecision {
    //         player_name: PlayerName::new("Bid"),
    //         action: AgentAction::Fold,
    //         reason: String::from("I can't beat that."),
    //     };
    //     let response = client
    //         .make_decision(MakeActionRequest {
    //             game_id: game_id.clone(),
    //             decision: bid_decision,
    //         })
    //         .await
    //         .unwrap();

    //     // It is expected that game is now finished, as all other players are
    // AllIn.     // Game was won by Adi.
    //     let expected = GameStatusResponse {
    //         game_id: game_id.clone(),
    //         status: GameStatus::Finished,
    //         next_player: None,
    //     };
    //     assert_eq!(response, expected);
    // }

    // fn card(s: &str) -> Card {
    //     Card::try_from(s).unwrap()
    // }
}

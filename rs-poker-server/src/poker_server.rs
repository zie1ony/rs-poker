use std::sync::{Arc, Mutex};

use crate::{
    handler::{
        game_full_view::GameFullViewHandler,
        game_info::GameInfoHandler,
        game_info_stream::{GameInfoStreamHandler, GameInfoStreamSubscribers},
        game_list::ListGamesHandler,
        game_make_action::MakeActionHandler,
        game_new::NewGameHandler,
        game_player_view::GamePlayerViewHandler,
        health_check::HealthCheckHandler,
        Handler,
    },
    persistence::Persistance,
};
use axum::Router;
use rs_poker_engine::poker_engine::PokerEngine;

macro_rules! router {
    ($use_storage:expr, $($handler:ident),* $(,)?) => {
        Router::new()
            $(
                .route($handler::path(), $handler::router())
            )*
            .with_state(ServerState::new($use_storage))
    };
}

#[derive(Clone)]
pub struct ServerState {
    pub engine: Arc<Mutex<PokerEngine>>,
    pub game_subscribers: GameInfoStreamSubscribers,
}

impl ServerState {
    pub fn new(use_storage: bool) -> Self {
        let engine = if use_storage {
            let storage = Box::new(Persistance::new());
            PokerEngine::new_with_storage(storage)
        } else {
            PokerEngine::new()
        };
        Self {
            engine: Arc::new(Mutex::new(engine)),
            game_subscribers: GameInfoStreamSubscribers::new(),
        }
    }
}

pub fn app_with_storage() -> Router {
    app(true)
}

pub fn app_no_storage() -> Router {
    app(false)
}

pub fn app(use_storage: bool) -> Router {
    router! {
        use_storage,

        // Game.
        HealthCheckHandler,
        NewGameHandler,
        ListGamesHandler,
        GameFullViewHandler,
        GamePlayerViewHandler,
        GameInfoHandler,
        GameInfoStreamHandler,
        MakeActionHandler,
    }
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

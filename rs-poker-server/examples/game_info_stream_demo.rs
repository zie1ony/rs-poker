use rs_poker::arena::action::AgentAction;
use rs_poker_types::{
    game::{Decision, GameId, GameInfo, GameSettings, GameStatus},
    player::{Player, PlayerName},
};
use std::{net::SocketAddr, vec};
use tokio::{
    task::JoinHandle,
    time::{sleep, Duration},
};

use rs_poker_engine::poker_engine::PokerEngineError;
use rs_poker_server::{
    error::ServerError,
    handler::{game_info_stream::GameInfoStream, game_make_action::MakeActionRequest},
    poker_client::{PokerClient, PokerClientError},
    poker_server::app_no_storage,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Game Info Stream...");

    let server = start_server().await;
    run_scenario().await?;
    server.abort();
    println!("Server stopped.");

    Ok(())
}

async fn start_server() -> JoinHandle<()> {
    // Create a test client that uses the axum app directly
    let app = app_no_storage();

    // Start a real HTTP server for testing the HTTP client
    println!("\n--- Starting HTTP Server ---");
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // Start server in background
    let server_handle = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        println!("Server running on http://127.0.0.1:3000");
        axum::serve(listener, app).await.unwrap();
    });

    // Give the server time to start
    sleep(Duration::from_millis(100)).await;

    server_handle
}

async fn run_scenario() -> Result<(), Box<dyn std::error::Error>> {
    let client = PokerClient::new_http("http://127.0.0.1:3000");
    let game_id = GameId::new("demo-game-123");

    let players = vec![Player::human("Alice"), Player::human("Bob")];

    let alice_name = PlayerName::new("Alice");
    let bob_name = PlayerName::new("Bob");

    let stacks = vec![100.0; players.len()];

    let game_settings = GameSettings {
        tournament_id: None,
        tournament_game_number: None,
        game_id: Some(game_id.clone()),
        small_blind: 5.0,
        players: players.clone(),
        stacks,
        hands: None,
        community_cards: None,
        dealer_index: 0,
    };

    let create_resp = client.new_game(&game_settings).await?;
    println!("Game created with ID: {}", create_resp.game_id);

    // let mut stream1_messages = vec![];
    let stream1 = client.game_info_stream(&game_id).await?;
    let stream1_result = listen(stream1);

    // Alice all-in
    client
        .make_action(MakeActionRequest {
            game_id: game_id.clone(),
            player_name: alice_name.clone(),
            decision: Decision {
                reason: "All-in for testing".to_string(),
                action: AgentAction::AllIn,
            },
        })
        .await?;

    let stream2 = client.game_info_stream(&game_id).await?;
    let stream2_result = listen(stream2);

    // Bob Folds.
    client
        .make_action(MakeActionRequest {
            game_id: game_id.clone(),
            player_name: bob_name.clone(),
            decision: Decision {
                reason: "Folding for testing".to_string(),
                action: AgentAction::Fold,
            },
        })
        .await?;

    let stream1_expected = vec![
        GameInfo {
            game_id: game_id.clone(),
            players: players.clone(),
            status: GameStatus::InProgress,
            current_player_name: Some(alice_name),
        },
        GameInfo {
            game_id: game_id.clone(),
            players: players.clone(),
            status: GameStatus::InProgress,
            current_player_name: Some(bob_name),
        },
        GameInfo {
            game_id: game_id.clone(),
            players: players.clone(),
            status: GameStatus::Finished,
            current_player_name: None,
        },
    ];
    let stream1_messages = stream1_result.await;
    assert_eq!(stream1_messages, stream1_expected);

    let stream2_expected = stream1_expected[1..].to_vec();
    let stream2_messages = stream2_result.await;
    assert_eq!(stream2_messages, stream2_expected);

    // Try to connect to finished game
    let stream3 = client.game_info_stream(&game_id).await?;
    let stream3_messages = listen(stream3).await;
    let stream3_expected = stream1_expected[2..].to_vec();
    assert_eq!(stream3_messages, stream3_expected);

    // Try to connect to non-existing game
    let non_existing_game_id = GameId::new("non-existing-game");
    let stream4 = client.game_info_stream(&non_existing_game_id).await;
    assert!(stream4.is_err());
    if let Err(error) = stream4 {
        // The error comes from the engine, so it's wrapped in PokerEngineError
        use rs_poker_engine::poker_engine::PokerEngineError;
        let expected_error = PokerClientError::ServerError(ServerError::PokerEngineError(
            PokerEngineError::GameNotFound(non_existing_game_id.clone()),
        ));
        assert_eq!(error, expected_error);
        println!("Received expected error for non-existing game: {:?}", error);
    }

    Ok(())
}

async fn listen(mut stream: GameInfoStream) -> Vec<GameInfo> {
    let handle = tokio::spawn(async move {
        let mut messages = vec![];
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(msg) => {
                    messages.push(msg.clone());
                    println!("Received message: {:?}", msg);
                }
                Err(e) => {
                    eprintln!("WebSocket error: {}", e);
                    break;
                }
            }
        }
        messages
    });
    handle.await.unwrap()
}

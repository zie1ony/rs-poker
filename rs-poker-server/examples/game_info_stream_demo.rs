use std::{net::SocketAddr};
use futures_util::StreamExt;
use rs_poker_types::{game::{GameId, GameSettings}, player::Player};
use tokio::{task::JoinHandle, time::{sleep, Duration}};

use rs_poker_server::{poker_client::PokerClient, poker_server::app_no_storage};

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

    let players = vec![
        Player::human("Alice"),
        Player::human("Bob"),
    ];

    let stacks = vec![100.0; players.len()];
    
    let game_settings = GameSettings {
        tournament_id: None,
        tournament_game_number: None,
        game_id: Some(game_id.clone()),
        small_blind: 5.0,
        players,
        stacks,
        hands: None,
        community_cards: None,
        dealer_index: 0,
    };

    let create_resp = client.new_game(&game_settings).await?;
    println!("Game created with ID: {}", create_resp.game_id);

    // let mut stream1_messages = vec![];
    let mut stream1 = client.game_info_stream(&game_id).await?;

    while let Some(msg) = stream1.next().await {
        match msg {
            Ok(msg) => {
                println!("Received message: {:?}", msg);
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
        }
    }

    println!("Game info stream ended.");
    Ok(())
}
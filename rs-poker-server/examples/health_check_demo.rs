use rs_poker_server::{
    handler::health_check::HealthCheckRequest, poker_client::PokerClient, poker_server::app,
};
use std::net::SocketAddr;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing PokerClient...");

    // Create a test client that uses the axum app directly
    let app = app();

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

    // Test HTTP client
    println!("\n--- Testing with HTTP Client ---");
    let http_client = PokerClient::new_http("http://127.0.0.1:3000");

    println!("Testing HTTP health check...");
    let health_request = HealthCheckRequest {
        id: "http-test-456".to_string(),
    };
    let response = http_client.health_check(health_request).await.unwrap();

    match response.id {
        id if id == "http-test-456" => println!(
            "✅ HTTP health check response: id={}, status={}",
            id, response.status
        ),
        _ => println!("❌ HTTP health check error: unexpected id {}", response.id),
    }

    // Clean shutdown
    server_handle.abort();
    println!("\n✅ All tests completed!");

    Ok(())
}

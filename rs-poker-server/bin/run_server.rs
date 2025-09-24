use rs_poker_server::poker_server::app;
use tokio::signal;

#[tokio::main]
pub async fn main() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001")
        .await
        .unwrap();
    println!("Listening on http://{}", listener.local_addr().unwrap());

    // Create the server
    let server = axum::serve(listener, app());

    // Handle graceful shutdown on Ctrl+C
    tokio::select! {
        result = server => {
            if let Err(err) = result {
                eprintln!("Server error: {}", err);
            }
        }
        _ = signal::ctrl_c() => {
            println!("\nReceived Ctrl+C, shutting down gracefully...");
        }
    }
}

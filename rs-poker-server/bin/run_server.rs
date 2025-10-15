use rs_poker_server::poker_server::app_no_storage;
use tokio::signal;

#[tokio::main]
pub async fn main() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001")
        .await
        .unwrap();
    println!("Listening on http://{}", listener.local_addr().unwrap());

    // Create and run the server with graceful shutdown
    axum::serve(listener, app_no_storage())
        .with_graceful_shutdown(async {
            signal::ctrl_c()
                .await
                .expect("Failed to listen for ctrl_c signal");
            println!("\nReceived Ctrl+C, shutting down gracefully...");
        })
        .await
        .unwrap();
}

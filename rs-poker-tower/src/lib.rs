use rs_poker_server::poker_client::PokerClient;
use tokio::sync::{mpsc, oneshot};

use crate::{
    tower::Tower,
    worker::{Worker, WorkerId, WorkerMessage},
};

pub mod ai_player;
pub mod tower;
pub mod worker;

pub fn client() -> PokerClient {
    PokerClient::new_http("http://localhost:3001")
}

pub async fn run() {
    // Configuration.
    let number_of_workers = 2;
    let max_tasks = 4;

    // Create communication channels for workers to send messages to the tower.
    // mpsc works great here as there are multiple workers sending messages to a
    // single tower.
    let (worker_msg_producer, worker_msg_consumer) = mpsc::channel::<WorkerMessage>(1000);

    // For each worker, create a channel for the tower to send messages to the
    // worker. Here, mpsc is used as as single producer, single consumer, but it
    // is fine.
    let mut tower_msg_producers = Vec::new();
    let mut tower_msg_consumers = Vec::new();

    for _ in 0..number_of_workers {
        let (tx, rx) = mpsc::channel::<tower::TowerMessage>(100);
        tower_msg_producers.push(tx);
        tower_msg_consumers.push(rx);
    }

    // Create the tower.
    let mut tower = Tower::new(
        max_tasks,
        client(),
        worker_msg_consumer,
        tower_msg_producers,
    );

    // Create and spawn workers
    let mut worker_handles = Vec::new();
    for worker_id in 0..number_of_workers {
        let worker_msg_sender = worker_msg_producer.clone();
        let tower_msg_receiver = tower_msg_consumers.remove(0);
        let handle = tokio::spawn(async move {
            let mut worker =
                Worker::new(WorkerId(worker_id), worker_msg_sender, tower_msg_receiver, client());
            worker.run().await;
        });
        worker_handles.push(handle);
    }

    // Create a shutdown channel
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    let mut tower_handle = tokio::spawn(async move {
        // Run tower with cancellation support
        tokio::select! {
            _ = tower.run() => {
                println!("Tower finished naturally");
            }
            _ = shutdown_rx => {
                println!("Tower received shutdown signal, shutting down workers...");
                tower.shutdown("External shutdown signal").await;
            }
        }
    });

    // Wait for either tower to finish or ctrl+c signal to exit
    tokio::select! {
        _ = &mut tower_handle => {
            println!("Tower finished, waiting for workers to stop...");
        }
        _ = tokio::signal::ctrl_c() => {
            println!("Received Ctrl+C signal");
            // Signal tower to shutdown gracefully
            let _ = shutdown_tx.send(());
            // Wait for tower to finish shutdown
            let _ = tower_handle.await;
        }
    }

    // Wait for all workers to finish
    for handle in worker_handles {
        if let Err(e) = handle.await {
            eprintln!("Worker task failed: {:?}", e);
        }
    }

    println!("Gracefully shutting down tower.");
}

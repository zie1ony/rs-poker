use rs_poker_server::poker_client::{self, PokerClient};
use tokio::sync::mpsc;

use rs_poker_types::{game::GameId, tournament::TournamentId};

use crate::tower::TowerMessage;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WorkerId(pub usize);

pub struct Worker {
    id: WorkerId,
    worker_msg_sender: mpsc::Sender<WorkerMessage>,
    tower_msg_receiver: mpsc::Receiver<TowerMessage>,
    poker_client: PokerClient
}

impl Worker {
    pub fn new(
        id: WorkerId,
        worker_msg_sender: mpsc::Sender<WorkerMessage>,
        tower_msg_receiver: mpsc::Receiver<TowerMessage>,
        poker_client: PokerClient
    ) -> Self {
        Self {
            id,
            worker_msg_sender,
            tower_msg_receiver,
            poker_client
        }
    }

    pub async fn run(&mut self) {
        println!("[w] Worker {:?} started.", self.id);
        
        // Notify tower that worker is waiting for work
        if !self.send_ready_message().await {
            return;
        }

        loop {
            if let Some(tower_msg) = self.tower_msg_receiver.recv().await {
                match tower_msg {
                    TowerMessage::StartTournament { tournament_id } => {
                        if !self.handle_tournament(tournament_id).await {
                            break;
                        }
                    }
                    TowerMessage::Shutdown => {
                        println!("[w] Worker {:?} received shutdown signal.", self.id);
                        break;
                    }
                }
            } else {
                println!("[w] Worker {:?} exiting.", self.id);
                break;
            }
        }
    }

    async fn handle_tournament(&mut self, tournament_id: TournamentId) -> bool {
        // Notify tournament started
        let msg = WorkerMessage::TorunamentStarted {
            worker_id: self.id,
            tournament_id: tournament_id.clone(),
        };
        if !self.send_message(msg).await {
            return false;
        }

        // Execute the tournament.
        self.execute_tournament(&tournament_id).await;

        // Notify tournament finished
        let msg = WorkerMessage::TournamentFinished {
            worker_id: self.id,
            tournament_id,
        };
        if !self.send_message(msg).await {
            return false;
        }

        // Signal ready for next task
        self.send_ready_message().await
    }

    async fn send_ready_message(&self) -> bool {
        let msg = WorkerMessage::WorkerReadyForNewTask { worker_id: self.id };
        self.send_message(msg).await
    }

    async fn send_message(&self, msg: WorkerMessage) -> bool {
        match self.worker_msg_sender.send(msg).await {
            Ok(_) => true,
            Err(_) => {
                println!("[w] Worker {:?} exiting - tower stopped.", self.id);
                false
            }
        }
    }

    async fn execute_tournament(&self, tournament_id: &TournamentId) {
        // Fetch tournament info.
        let info = self.poker_client.tournament_info(tournament_id).await;
        println!("Tournament info: {:#?}", info);

        if let Some()
    }

    async fn execute_game(&self, game_id: &GameId) {
        let info = self.poker_client.game_info(game_id).await;

    }
}

pub enum WorkerMessage {
    WorkerReadyForNewTask {
        worker_id: WorkerId,
    },
    TorunamentStarted {
        worker_id: WorkerId,
        tournament_id: TournamentId,
    },
    TournamentFinished {
        worker_id: WorkerId,
        tournament_id: TournamentId,
    },
}

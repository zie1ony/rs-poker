use std::collections::{HashMap, HashSet};

use rs_poker_server::{
    handler::tournament_list::ListTournamentsRequest, poker_client::PokerClient,
};
use rs_poker_types::tournament::TournamentId;
use tokio::sync::mpsc;

use crate::worker::{WorkerId, WorkerMessage};

pub struct WorkersManager {
    status: HashMap<WorkerId, WorkerStatus>,
    tasks_completed: HashMap<WorkerId, usize>,
    channels: HashMap<WorkerId, mpsc::Sender<TowerMessage>>,
}

impl WorkersManager {
    pub fn new(channels: HashMap<WorkerId, mpsc::Sender<TowerMessage>>) -> Self {
        let mut status = HashMap::new();
        let mut tasks_completed = HashMap::new();

        // Initialize all workers as idle
        for worker_id in channels.keys() {
            status.insert(*worker_id, WorkerStatus::Idle);
            tasks_completed.insert(*worker_id, 0);
        }

        Self {
            status,
            tasks_completed,
            channels,
        }
    }

    pub fn workers_count(&self) -> usize {
        self.status.len()
    }

    pub fn get_idle_worker(&self) -> Option<WorkerId> {
        self.status
            .iter()
            .find(|(_, status)| matches!(status, WorkerStatus::Idle))
            .map(|(worker_id, _)| *worker_id)
    }

    pub fn set_worker_status(&mut self, worker_id: WorkerId, status: WorkerStatus) {
        self.status.insert(worker_id, status);
    }

    pub fn increment_task_count(&mut self, worker_id: WorkerId) {
        *self.tasks_completed.entry(worker_id).or_insert(0) += 1;
    }

    pub fn total_tasks_completed(&self) -> usize {
        self.tasks_completed.values().sum()
    }

    pub async fn send_to_worker(
        &self,
        worker_id: WorkerId,
        message: TowerMessage,
    ) -> Result<(), mpsc::error::SendError<TowerMessage>> {
        if let Some(sender) = self.channels.get(&worker_id) {
            sender.send(message).await
        } else {
            Err(mpsc::error::SendError(message))
        }
    }

    pub async fn broadcast_shutdown(&self) {
        for (_, sender) in &self.channels {
            let _ = sender.send(TowerMessage::Shutdown).await;
        }
    }
}

pub enum WorkerStatus {
    Idle,
    Working(TournamentId),
}

pub struct Tower {
    // This is the total number of tasks that after which the tower stops and exits.
    max_tasks: usize,

    // Tracking information about workers.
    workers_info: WorkersManager,

    // Channel used to send messages from workers to the tower.
    workers_msg_receiver: mpsc::Receiver<WorkerMessage>,

    // Client to interact with the poker server.
    poker_client: PokerClient,

    // Tournament counter for creating tournament IDs
    tournament_counter: usize,

    // Track tournaments that are currently being processed by workers
    tournaments_in_progress: HashSet<TournamentId>,
}

impl Tower {
    pub fn new(
        max_tasks: usize,
        poker_client: PokerClient,
        workers_msg_receiver: mpsc::Receiver<WorkerMessage>,
        tower_msg_producers: Vec<mpsc::Sender<TowerMessage>>,
    ) -> Self {
        // Create a mapping from WorkerId to their message channels
        let mut channels = HashMap::new();
        for (i, sender) in tower_msg_producers.into_iter().enumerate() {
            channels.insert(WorkerId(i), sender);
        }

        Self {
            max_tasks,
            workers_info: WorkersManager::new(channels),
            workers_msg_receiver,
            poker_client,
            tournament_counter: 0,
            tournaments_in_progress: HashSet::new(),
        }
    }

    /// Runs the tower's main event loop.
    ///
    /// The tower will:
    /// 1. Listen for worker messages
    /// 2. Assign tournaments to idle workers
    /// 3. Track tournament completion
    /// 4. After `max_tasks` tournaments are started, send shutdown to all
    ///    workers
    /// 5. Exit when all workers have been notified to shutdown
    pub async fn run(&mut self) {
        self.print_startup_message();

        // Main event loop - continue until we've started max_tasks tournaments
        while self.should_continue_running() {
            if let Some(worker_message) = self.workers_msg_receiver.recv().await {
                self.handle_worker_message(worker_message).await;
            } else {
                // Channel closed, exit loop
                break;
            }
        }

        self.shutdown(&format!("Max {} number of tasks reached.", self.max_tasks))
            .await;
    }

    fn print_startup_message(&self) {
        println!(
            "[t] Tower started with {} workers",
            self.workers_info.workers_count()
        );
    }

    fn should_continue_running(&self) -> bool {
        self.workers_info.total_tasks_completed() < self.max_tasks
    }

    async fn handle_worker_message(&mut self, worker_message: WorkerMessage) {
        match worker_message {
            WorkerMessage::WorkerReadyForNewTask { worker_id } => {
                self.handle_worker_ready(worker_id).await;
            }
            WorkerMessage::TorunamentStarted {
                worker_id,
                tournament_id,
            } => {
                self.handle_tournament_started(worker_id, tournament_id);
            }
            WorkerMessage::TournamentFinished {
                worker_id,
                tournament_id,
            } => {
                self.handle_tournament_finished(worker_id, tournament_id);
            }
        }
    }

    async fn handle_worker_ready(&mut self, worker_id: WorkerId) {
        println!("[t] Worker {:?} is ready", worker_id);
        self.workers_info
            .set_worker_status(worker_id, WorkerStatus::Idle);

        // If we haven't reached max tasks, assign a new tournament
        if self.can_assign_new_tournament() {
            self.assign_tournament_to_worker(worker_id).await;
        }
    }

    fn handle_tournament_started(&mut self, worker_id: WorkerId, tournament_id: TournamentId) {
        println!(
            "[t] Worker {:?} started tournament {:?}",
            worker_id, tournament_id
        );
        self.workers_info
            .set_worker_status(worker_id, WorkerStatus::Working(tournament_id));
    }

    fn handle_tournament_finished(&mut self, worker_id: WorkerId, tournament_id: TournamentId) {
        println!(
            "[t] Worker {:?} finished tournament {:?}",
            worker_id, tournament_id
        );

        // Remove tournament from in-progress tracking
        self.tournaments_in_progress.remove(&tournament_id);

        self.workers_info.increment_task_count(worker_id);
        self.workers_info
            .set_worker_status(worker_id, WorkerStatus::Idle);
    }

    fn can_assign_new_tournament(&self) -> bool {
        self.tournament_counter < self.max_tasks
    }

    async fn assign_tournament_to_worker(&mut self, worker_id: WorkerId) {
        if let Some(tournament_id) = self.get_next_tournament_id().await {
            self.tournament_counter += 1;

            // Mark tournament as being processed
            self.tournaments_in_progress.insert(tournament_id.clone());

            println!(
                "[t] Assigning tournament {:?} to worker {:?}",
                tournament_id, worker_id
            );

            if let Err(e) = self
                .send_tournament_to_worker(worker_id, &tournament_id)
                .await
            {
                eprintln!(
                    "[!] Failed to send tournament to worker {:?}: {:?}",
                    worker_id, e
                );
                // Remove from in-progress set if sending failed
                self.tournaments_in_progress.remove(&tournament_id);
            } else {
                self.workers_info
                    .set_worker_status(worker_id, WorkerStatus::Working(tournament_id));
            }
        } else {
            println!("[t] No available tournaments for worker {:?}", worker_id);
        }
    }

    async fn load_active_tournaments(&self) -> Vec<TournamentId> {
        let request = ListTournamentsRequest { active_only: true };
        let results = self.poker_client.list_tournaments(request).await.unwrap();
        let ids = results
            .tournament_ids
            .into_iter()
            .map(|(id, _)| id)
            .collect();
        ids
    }

    async fn get_next_tournament_id(&self) -> Option<TournamentId> {
        let active_tournaments = self.load_active_tournaments().await;

        // Find a tournament that is not currently being processed by any worker
        for tournament_id in active_tournaments {
            if !self.tournaments_in_progress.contains(&tournament_id) {
                return Some(tournament_id);
            }
        }

        // No available tournaments found
        None
    }

    async fn send_tournament_to_worker(
        &self,
        worker_id: WorkerId,
        tournament_id: &TournamentId,
    ) -> Result<(), mpsc::error::SendError<TowerMessage>> {
        self.workers_info
            .send_to_worker(
                worker_id,
                TowerMessage::StartTournament {
                    tournament_id: tournament_id.clone(),
                },
            )
            .await
    }

    pub async fn shutdown(&mut self, reason: &str) {
        println!(
            "[t] Shutting down tower. Reason: {}. Shutting down workers...",
            reason
        );

        // Send shutdown message to all workers
        self.workers_info.broadcast_shutdown().await;

        println!("[t] Tower exiting.");
    }
}

pub enum TowerMessage {
    Shutdown,
    StartTournament { tournament_id: TournamentId },
}

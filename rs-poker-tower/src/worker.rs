use rs_poker_server::{
    handler::{game_full_view::GameFullViewRequest, game_make_action::MakeActionRequest},
    poker_client::PokerClient,
};
use tokio::sync::mpsc;

use rs_poker_types::{game::GameId, tournament::TournamentId};

use crate::{logger::TournamentLogger, tower::TowerMessage};

const SYSTEM_PROMPT: &str = r#"
You are an expert Texas Hold'em poker player.
You take part in the Texas Hold'em poker tournament with max 10 games.
If the tournament reaches 10 games, the player with the highest total chips wins.
Winner takes it all.
You will be given the full tournament log so far, including all previous games and the current game state.
For your convenience, you will be given possible actions you can take.
Before you decide, think.
At the end make a decision what to do next, and only respond with one of the available actions.
Follow given strategy.

BETTING RULES:
- Use Bet(amount) to specify the TOTAL amount you want to bet in the current round.
- Minimum raise must equal the previous bet/raise amount (e.g., if big blind is X, minimum raise is X more, so Bet(2*X) total)
- If you're in small blind (Y) and want to raise, Bet(Z) means you're adding Z-Y more chips (Z total - Y already posted)
- Invalid bet amounts will result in an automatic fold, so always ensure your bet meets minimum requirements
- Use Call to match the current bet, AllIn to bet all remaining chips, or Fold to quit the hand
"#;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WorkerId(pub usize);

pub struct Worker {
    id: WorkerId,
    worker_msg_sender: mpsc::Sender<WorkerMessage>,
    tower_msg_receiver: mpsc::Receiver<TowerMessage>,
    poker_client: PokerClient,
}

impl Worker {
    pub fn new(
        id: WorkerId,
        worker_msg_sender: mpsc::Sender<WorkerMessage>,
        tower_msg_receiver: mpsc::Receiver<TowerMessage>,
        poker_client: PokerClient,
    ) -> Self {
        Self {
            id,
            worker_msg_sender,
            tower_msg_receiver,
            poker_client,
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
        let logger = TournamentLogger::new(&tournament_id);
        println!(
            "[w] Worker {:?} executing tournament {:?}",
            self.id, tournament_id
        );

        // Fetch tournament info.
        let info = self.poker_client.tournament_info(tournament_id).await;
        if let Err(e) = info {
            println!(
                "[w] Worker {:?} failed to get tournament info: {:?}",
                self.id, e
            );
            return;
        }
        let info = info.unwrap();
        println!("[w] Tournament info: {:#?}", info);

        // Keep running games until tournament is complete
        loop {
            // Get current tournament status
            let tournament_info = match self.poker_client.tournament_info(tournament_id).await {
                Ok(info) => info,
                Err(e) => {
                    println!(
                        "[w] Worker {:?} failed to get tournament info: {:?}",
                        self.id, e
                    );
                    break;
                }
            };

            match tournament_info.status {
                rs_poker_types::tournament::TournamentStatus::WaitingForNextGame => {
                    // Tournament is waiting - it should automatically start the next game
                    println!("[w] Tournament waiting for next game, checking again...");
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
                rs_poker_types::tournament::TournamentStatus::GameInProgress => {
                    if let Some(game_id) = tournament_info.current_game_id {
                        println!("[w] Tournament has game in progress: {:?}", game_id);
                        self.execute_game(&game_id, &logger).await;
                    } else {
                        println!("[w] Tournament claims game in progress but no game ID found");
                        break;
                    }
                }
                rs_poker_types::tournament::TournamentStatus::Completed => {
                    println!("[w] Tournament completed!");
                    break;
                }
            }
        }

        // Log tournament finished
        let info = self
            .poker_client
            .tournament_full_view(&tournament_id)
            .await
            .unwrap();
        logger.log_tournament_finished(&info.summary);
    }

    async fn execute_game(&self, game_id: &GameId, logger: &TournamentLogger) {
        println!("[w] Worker {:?} executing game {:?}", self.id, game_id);

        // Keep processing until game is complete
        loop {
            let game_info = match self.poker_client.game_info(game_id).await {
                Ok(info) => info,
                Err(e) => {
                    println!("[w] Worker {:?} failed to get game info: {:?}", self.id, e);
                    break;
                }
            };

            match game_info.status {
                rs_poker_types::game::GameStatus::InProgress => {
                    if let Some(current_player) = game_info.current_player() {
                        println!("[w] Current player: {:?}", current_player.name());

                        // Only handle AI players
                        if let rs_poker_types::player::Player::AI {
                            name,
                            model,
                            strategy,
                        } = current_player
                        {
                            // Get the game view for this player
                            let game_view = match self.poker_client.game_player_view(
                                rs_poker_server::handler::game_player_view::GamePlayerViewRequest {
                                    game_id: game_id.clone(),
                                    player_name: name.clone(),
                                }
                            ).await {
                                Ok(view) => view,
                                Err(e) => {
                                    println!("[w] Worker {:?} failed to get player view: {:?}", self.id, e);
                                    break;
                                }
                            };

                            let system_prompt = SYSTEM_PROMPT.to_string();

                            // Make AI decision
                            let (user_prompt, decision) = crate::ai_player::decide(
                                model.clone(),
                                system_prompt.clone(),
                                strategy.clone(),
                                game_view.summary.clone(),
                                format!("{:?}", game_view.possible_actions),
                            )
                            .await;

                            let decision_str = format!("{:#?}", decision);

                            logger.log_game_action(
                                game_id,
                                &system_prompt,
                                &user_prompt,
                                &decision_str,
                            );

                            // Submit the decision
                            match self
                                .poker_client
                                .make_decision(MakeActionRequest {
                                    game_id: game_id.clone(),
                                    decision,
                                })
                                .await
                            {
                                Ok(_) => {
                                    println!("[w] AI player {:?} made decision", name);
                                }
                                Err(e) => {
                                    println!(
                                        "[w] Worker {:?} failed to submit decision: {:?}",
                                        self.id, e
                                    );
                                    break;
                                }
                            }
                        } else {
                            // For human players or other types, we can't proceed
                            println!(
                                "[w] Current player is not AI, cannot proceed: {:?}",
                                current_player
                            );
                            break;
                        }
                    } else {
                        println!("[w] Game in progress but no current player found");
                        break;
                    }
                }
                rs_poker_types::game::GameStatus::Finished => {
                    let full_view = self
                        .poker_client
                        .game_full_view(GameFullViewRequest {
                            game_id: game_id.clone(),
                            debug: false,
                        })
                        .await
                        .unwrap();
                    logger.log_game_finished(game_id, &full_view.summary);
                    println!("[w] Game {:?} finished", game_id);
                    break;
                }
            }

            // Small delay to avoid overwhelming the server
            // tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
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

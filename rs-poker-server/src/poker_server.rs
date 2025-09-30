use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    handler::{
        game_full_view::GameFullViewHandler, game_info::GameInfoHandler,
        game_list::ListGamesHandler, game_make_action::MakeActionHandler, game_new::NewGameHandler,
        game_player_view::GamePlayerViewHandler, health_check::HealthCheckHandler,
        tournament_full_view::TournamentFullViewHandler, tournament_info::TournamentInfoHandler,
        tournament_list::ListTournamentsHandler, tournament_new::NewTournamentHandler,
        tournament_player_view::TournamentPlayerViewHandler, Handler,
    },
    persistence,
};
use axum::Router;
use rs_poker_engine::{
    game_instance::GameInstance,
    tournament_instance::{TournamentAction, TournamentInstance},
};
use rs_poker_types::{game::GameId, tournament::TournamentId};

macro_rules! router {
    ($($handler:ident),* $(,)?) => {
        Router::new()
            $(
                .route($handler::path(), $handler::router())
            )*
            .with_state(ServerState::new())
    };
}

#[derive(Clone, Default)]
pub struct PokerServer {
    pub games: HashMap<GameId, GameInstance>,
    pub tournaments: HashMap<TournamentId, TournamentInstance>,
}

impl PokerServer {
    pub fn new() -> Self {
        let games: HashMap<GameId, GameInstance> = persistence::load_games()
            .unwrap()
            .into_iter()
            .map(|g| (g.game_id(), g))
            .collect();
        let tournaments: HashMap<TournamentId, TournamentInstance> =
            persistence::load_tournaments()
                .unwrap()
                .into_iter()
                .map(|t| (t.tournament_id(), t))
                .collect();

        println!(
            "Loaded {} games and {} tournaments from storage.",
            games.len(),
            tournaments.len()
        );

        Self { games, tournaments }
    }

    pub fn game(&self, game_id: &GameId) -> Option<GameInstance> {
        self.games.get(game_id).cloned()
    }

    pub fn update_game(&mut self, game: &GameInstance) {
        self.games.insert(game.game_id(), game.clone());
        persistence::store_game(game).unwrap();
    }

    pub fn tournament(&self, tournament_id: &TournamentId) -> Option<TournamentInstance> {
        self.tournaments.get(tournament_id).cloned()
    }

    pub fn update_tournament(&mut self, tournament: &TournamentInstance) {
        self.tournaments
            .insert(tournament.tournament_id(), tournament.clone());
        persistence::store_tournament(tournament).unwrap();
    }

    pub fn progress_tournament(&mut self, tournament_id: &TournamentId) {
        if let Some(mut tournament) = self.tournament(tournament_id) {
            while let Some(action) = tournament.next_action() {
                match action {
                    TournamentAction::StartNextGame { game_settings } => {
                        // Create and store a new game instance.
                        let mut game =
                            GameInstance::new_from_config_with_random_cards(&game_settings);
                        game.run();

                        if !game.is_complete() {
                            // Store the game.
                            self.update_game(&game);

                            // Update tournament and break if the game could not be completed.
                            // This means it is waiting for player input.
                            self.update_tournament(&tournament);
                            break;
                        }

                        // If the game is complete, load the final results.
                        let game_result = game.game_final_results().unwrap();

                        // Also store the game.
                        self.update_game(&game);

                        // Finish the game in the tournament.
                        tournament.finish_game(&game_result).unwrap();
                        // Continue to the next action.
                    }
                    TournamentAction::FinishGame { game_id } => {
                        // This state means the game has already been started,
                        // and needs to be pushed to completion.
                        if let Some(mut game) = self.game(&game_id) {
                            if !game.is_complete() {
                                game.run();
                                if game.is_complete() {
                                    let game_result = game.game_final_results().unwrap();
                                    tournament.finish_game(&game_result).unwrap();
                                    self.update_game(&game);
                                } else {
                                    self.update_game(&game);
                                    self.update_tournament(&tournament);
                                    // Game is still not complete, break to wait for player input.
                                    break;
                                }
                            } else {
                                // Game is already complete, just finish it in the tournament.
                                let game_result = game.game_final_results().unwrap();
                                tournament.finish_game(&game_result).unwrap();
                            }
                        } else {
                            // Game not found, this is an error in the tournament state.
                            self.update_tournament(&tournament);
                            break;
                        }
                    }
                }
            }
            // Update tournament after all actions are processed
            self.update_tournament(&tournament);
        }
    }
}

#[derive(Clone)]
pub struct ServerState {
    pub server: Arc<Mutex<PokerServer>>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            server: Arc::new(Mutex::new(PokerServer::new())),
        }
    }
}

pub fn app() -> Router {
    router! {
        // Game.
        HealthCheckHandler,
        NewGameHandler,
        ListGamesHandler,
        GameFullViewHandler,
        GamePlayerViewHandler,
        GameInfoHandler,
        MakeActionHandler,

        // Tournament.
        NewTournamentHandler,
        ListTournamentsHandler,
        TournamentInfoHandler,
        TournamentFullViewHandler,
        TournamentPlayerViewHandler,
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

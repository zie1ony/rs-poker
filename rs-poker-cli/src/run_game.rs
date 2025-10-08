use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, ClearType},
};
use rs_poker::arena::action::AgentAction;
use rs_poker_server::{
    handler::{game_make_action::MakeActionRequest, game_player_view::GamePlayerViewRequest},
    poker_client::PokerClient,
    poker_server,
};
use rs_poker_types::{
    game::{Decision, GameId, GamePlayerView, GameSettings, GameStatus, PossibleAction},
    player::Player,
};
use std::io::{self, Write};

use crate::{
    ai_player,
    frame::{self, Frame},
};

pub fn client(mock_server: bool) -> PokerClient {
    if mock_server {
        PokerClient::new_test(poker_server::app())
    } else {
        PokerClient::new_http("http://localhost:3001")
    }
}

pub async fn run_example_game(mock_server: bool) {
    let client = client(mock_server);
    let game_id = initialize_game(&client).await;

    // Enable raw mode for immediate key detection

    loop {
        let mut frame = Frame::new(&game_id);
        let game_info = client.game_info(&game_id).await.unwrap();
        frame.with_game_info(game_info.clone());

        match game_info.status {
            GameStatus::InProgress => {
                handle_game_in_progress(&client, &game_id, &mut frame, &game_info).await;
            }
            GameStatus::Finished => {
                if handle_game_finished(&client, &game_id, &mut frame).await {
                    break;
                }
            }
        }
    }

    // Restore terminal state
    terminal::disable_raw_mode().unwrap();
}

async fn initialize_game(client: &PokerClient) -> GameId {
    let players = vec![
        Player::ai("Alice", "gpt-4o-mini", "aggressive, but withdraw sometimes"),
        Player::ai("Bob", "gpt-4o-mini", "defensive, but bluffy"),
        Player::human("Diana"),
    ];

    let players_count = players.len();

    // Start new game.
    let result = client
        .new_game(&GameSettings {
            tournament_id: None,
            tournament_game_number: None,
            game_id: None,
            small_blind: 5.0,
            players,
            stacks: vec![100.0; players_count],
            hands: None,
            community_cards: None,
            dealer_index: 0,
        })
        .await
        .unwrap();

    result.game_id
}

async fn handle_game_in_progress(
    client: &PokerClient,
    game_id: &GameId,
    frame: &mut Frame,
    game_info: &rs_poker_types::game::GameInfo,
) {
    // Get the current player.
    let current_player = game_info.current_player().unwrap();

    // Load the game view for the current player.
    let game: GamePlayerView = client
        .game_player_view(GamePlayerViewRequest {
            game_id: game_id.clone(),
            player_name: current_player.name().clone(),
        })
        .await
        .unwrap();

    // Update frame with game summary
    frame.with_game_summary(game.summary.clone());

    match current_player {
        Player::Automat { .. } => panic!("Should not happen!"),
        Player::Human { name: _ } => {
            handle_human_player_action(client, game_id, frame, &game).await;
        }
        Player::AI {
            name: _,
            model,
            strategy,
        } => {
            handle_ai_player_action(client, game_id, frame, &game, model, strategy).await;
        }
    }
}

async fn handle_human_player_action(
    client: &PokerClient,
    game_id: &GameId,
    frame: &mut Frame,
    game: &GamePlayerView,
) {
    let mut selected_index = 0;
    loop {
        display_action_menu(frame, &game.possible_actions, selected_index);
        frame::clean_frame();
        println!("{}", frame.render());

        // Wait for key input
        if let Ok(Event::Key(key_event)) = event::read() {
            match key_event.code {
                KeyCode::Up => {
                    if selected_index > 0 {
                        selected_index -= 1;
                    }
                }
                KeyCode::Down => {
                    if selected_index < game.possible_actions.len() - 1 {
                        selected_index += 1;
                    }
                }
                KeyCode::Enter => {
                    let selected_action = &game.possible_actions[selected_index];
                    let action = handle_action_selection(selected_action, frame).await;

                    if let Some(action) = action {
                        let decision = Decision {
                            action,
                            reason: "Player decision".to_string(),
                        };
                        let _result = client
                            .make_action(MakeActionRequest {
                                game_id: game_id.clone(),
                                player_name: game.player.clone(),
                                decision,
                            })
                            .await
                            .unwrap();
                        break; // Break out of action selection loop
                    }
                    // If action is None (user pressed Esc), continue the loop
                }
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                    // Exit the entire program
                    terminal::disable_raw_mode().unwrap();
                    std::process::exit(0);
                }
                _ => {}
            }
        }
    }
}

async fn handle_ai_player_action(
    client: &PokerClient,
    game_id: &GameId,
    frame: &mut Frame,
    game: &GamePlayerView,
    model: &str,
    strategy: &str,
) {
    frame::clean_frame();
    println!("{}", frame.render());

    let game_view = game.summary.clone();
    let possible_actions = game.possible_actions.clone();
    let decision = ai_player::decide(
        model.to_string(),
        strategy.to_string(),
        game_view,
        format!("{:?}", possible_actions),
    )
    .await;
    let _result = client
        .make_action(MakeActionRequest {
            game_id: game_id.clone(),
            player_name: game.player.clone(),
            decision,
        })
        .await
        .unwrap();
    frame::clean_frame();
    println!("{}", frame.render());
}

fn display_action_menu(
    frame: &mut Frame,
    possible_actions: &[PossibleAction],
    selected_index: usize,
) {
    let mut actions_text = String::new();
    actions_text.push_str("Available actions (use ↑/↓ to navigate, Enter to select, q to quit):\n");
    for (i, action) in possible_actions.iter().enumerate() {
        let prefix = if i == selected_index { "→ " } else { "  " };
        let action_text = match action {
            PossibleAction::Fold => "Fold".to_string(),
            PossibleAction::Call => "Call".to_string(),
            PossibleAction::Bet { min, max } => {
                format!("Bet (${:.2} - ${:.2})", min, max)
            }
            PossibleAction::AllIn => "All In".to_string(),
        };
        actions_text.push_str(&format!("{}{}\n", prefix, action_text));
    }
    frame.with_possible_actions(actions_text);
}

async fn handle_action_selection(
    selected_action: &PossibleAction,
    frame: &mut Frame,
) -> Option<AgentAction> {
    match selected_action {
        PossibleAction::Fold => Some(AgentAction::Fold),
        PossibleAction::Call => Some(AgentAction::Call),
        PossibleAction::AllIn => Some(AgentAction::AllIn),
        PossibleAction::Bet { min, max } => handle_bet_input(frame, *min, *max).await,
    }
}

async fn handle_bet_input(frame: &mut Frame, min: f32, max: f32) -> Option<AgentAction> {
    let mut input = String::new();
    let mut error_message = None;

    loop {
        let mut input_display = format!(
            "Enter bet amount (${:.2} - ${:.2}):\n> {}\n\n",
            min, max, input
        );

        if let Some(ref error) = error_message {
            input_display.push_str(&format!("Error: {}\n", error));
        }

        input_display.push_str("Press Enter to confirm, Backspace to delete, Esc to cancel");

        frame.with_possible_actions(input_display);
        frame::clean_frame();
        println!("{}", frame.render());

        if let Ok(Event::Key(key_event)) = event::read() {
            match key_event.code {
                KeyCode::Char(c) if c.is_ascii_digit() || c == '.' => {
                    input.push(c);
                    error_message = None;
                }
                KeyCode::Backspace => {
                    input.pop();
                    error_message = None;
                }
                KeyCode::Enter => {
                    if input.is_empty() {
                        error_message = Some("Please enter an amount".to_string());
                        continue;
                    }

                    match input.parse::<f32>() {
                        Ok(amount) => {
                            if amount >= min && amount <= max {
                                return Some(AgentAction::Bet(amount));
                            } else {
                                error_message = Some(format!(
                                    "Amount must be between ${:.2} and ${:.2}",
                                    min, max
                                ));
                            }
                        }
                        Err(_) => {
                            error_message = Some("Invalid number format".to_string());
                        }
                    }
                }
                KeyCode::Esc => {
                    return None; // User cancelled
                }
                _ => {}
            }
        }
    }
}

async fn handle_game_finished(client: &PokerClient, game_id: &GameId, frame: &mut Frame) -> bool {
    let game_full_view_resp = client.game_full_view(&game_id).await;
    let game = match game_full_view_resp {
        Ok(view) => view,
        Err(e) => {
            println!("Error fetching game view: {:?}", e);
            return true;
        }
    };
    frame.with_game_summary(game.summary);
    frame::clean_frame();
    println!("{}", frame.render());

    // Wait for any key to exit when game is finished
    if let Ok(Event::Key(_)) = event::read() {
        return true;
    }
    false
}

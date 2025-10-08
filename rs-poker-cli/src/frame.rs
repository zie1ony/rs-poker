use crossterm::{
    cursor, execute,
    terminal::{self, ClearType},
};
use rs_poker_types::game::{GameId, GameInfo};
use std::{
    io::{self, Write},
    time::{SystemTime, UNIX_EPOCH},
};

pub struct Frame {
    pub timestamp: u64,
    pub game_id: GameId,
    pub game_info: Option<GameInfo>,
    pub game_summary: Option<String>,
    pub possible_actions: Option<String>,
}

impl Frame {
    pub fn new(game_id: &GameId) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            timestamp,
            game_id: game_id.clone(),
            game_info: None,
            game_summary: None,
            possible_actions: None,
        }
    }

    pub fn with_game_info(&mut self, game_info: GameInfo) {
        self.game_info = Some(game_info);
    }

    pub fn with_game_summary(&mut self, game_summary: String) {
        self.game_summary = Some(game_summary);
    }

    pub fn with_possible_actions(&mut self, possible_actions: String) {
        self.possible_actions = Some(possible_actions);
    }

    // Render functions

    pub fn render(&self) -> String {
        let header = "Icefall v0.0.6\n";
        let divider = "==============================\n";
        let mut result = String::new();

        // Add header only once
        result.push_str(header);
        result.push_str(&divider);

        // Add HUD
        let hud = self.render_hud();
        result.push_str(&hud);

        if let Some(summary) = &self.game_summary {
            result.push_str(&divider);
            result.push_str(summary);
        }

        if let Some(possible_actions) = self.render_possible_actions() {
            result.push_str(&divider);
            result.push_str(&possible_actions);
        }

        // Fix line endings.
        result.replace("\n", "\r\n")
    }

    pub fn render_hud(&self) -> String {
        let mut result = String::new();
        result.push_str(&format!("Timestamp: {}\n", self.timestamp));
        result.push_str(&format!("Game: {:?}\n", self.game_id));

        if let Some(game_info) = &self.game_info {
            result.push_str(&format!("Players: {}\n", game_info.players.len()));
            result.push_str(&format!("Status: {:?}\n", game_info.status));
            if let Some(current_player_name) = &game_info.current_player_name {
                result.push_str(&format!("Current Player: {}\n", current_player_name));
            } else {
                result.push_str("Current Player: None\n");
            }
        }
        result
    }

    fn render_possible_actions(&self) -> Option<String> {
        self.possible_actions.clone()
    }
}

pub fn start_session() {
    terminal::enable_raw_mode().unwrap();
    execute!(io::stdout(), terminal::EnterAlternateScreen).unwrap();
}

pub fn end_session() {
    execute!(io::stdout(), terminal::LeaveAlternateScreen).unwrap();
    terminal::disable_raw_mode().unwrap();
}

pub fn clean_frame() {
    // Clear the entire screen and move cursor to top-left corner
    execute!(
        io::stdout(),
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0)
    )
    .unwrap();
    io::stdout().flush().unwrap();
}

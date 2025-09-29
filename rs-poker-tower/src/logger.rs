use rs_poker_types::{game::GameId, tournament::TournamentId};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::collections::HashMap;
use std::sync::Mutex;

pub const TOWER_LOGS_DIR: &str = "logs-tower";

pub struct TournamentLogger {
    tournament_id: TournamentId,
    action_counters: Mutex<HashMap<String, AtomicU64>>,
}

impl TournamentLogger {
    pub fn new(tournament_id: &TournamentId) -> Self {
        Self {
            tournament_id: tournament_id.clone(),
            action_counters: Mutex::new(HashMap::new()),
        }
    }
    
    fn get_timestamp() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let secs = duration.as_secs();
        
        // Simple timestamp format: seconds since epoch
        format!("{}", secs)
    }
    
    fn write_to_file(&self, filepath: &PathBuf, content: &str) -> std::io::Result<()> {
        // Ensure the logs directory exists
        if let Some(parent) = filepath.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(filepath)?;
        
        writeln!(file, "{}", content)?;
        Ok(())
    }
    
    // Log to `logs-tower/<tournament_id>_<game_id>_<action_number>.log`
    // examples:
    // logs-tower/tournament123_game456_action_1.log
    // logs-tower/tournament123_game456_action_2.log
    // logs-tower/tournament123_game456_action_3.log
    pub fn log_game_action(&self, game_id: &GameId, system_prompt: &str, user_prompt: &str, response: &str) {
        let game_key = format!("{}_{}", self.tournament_id.as_str(), game_id.as_str());
        
        // Get or create action counter for this game
        let action_number = {
            let mut counters = self.action_counters.lock().unwrap();
            let counter = counters.entry(game_key.clone()).or_insert_with(|| AtomicU64::new(1));
            counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        };
        
        let timestamp = Self::get_timestamp();
        let filename = format!("{}_{}_action_{}.log", self.tournament_id.as_str(), game_id.as_str(), action_number);
        let filepath = PathBuf::from(TOWER_LOGS_DIR).join(&filename);
        
        let log_content = format!(
            "Timestamp: {}\nTournament ID: {}\nGame ID: {}\nAction Number: {}\n\n=== SYSTEM PROMPT ===\n{}\n\n=== USER PROMPT ===\n{}\n\n=== RESPONSE ===\n{}\n",
            timestamp,
            self.tournament_id.as_str(),
            game_id.as_str(),
            action_number,
            system_prompt,
            user_prompt,
            response
        );
        
        if let Err(e) = self.write_to_file(&filepath, &log_content) {
            eprintln!("Failed to write action log to {}: {}", filepath.display(), e);
        }
    }

    // Log to `logs-tower/<tournament_id>_<game_id>.log`
    // example:
    // logs-tower/tournament123_game456.log
    pub fn log_game_finished(&self, game_id: &GameId, summary: &str) {
        let timestamp = Self::get_timestamp();
        let filename = format!("{}_{}.log", self.tournament_id.as_str(), game_id.as_str());
        let filepath = PathBuf::from(TOWER_LOGS_DIR).join(&filename);
        
        let log_content = format!(
            "Timestamp: {}\nTournament ID: {}\nGame ID: {}\nStatus: GAME FINISHED\n\n=== GAME SUMMARY ===\n{}\n",
            timestamp,
            self.tournament_id.as_str(),
            game_id.as_str(),
            summary
        );
        
        if let Err(e) = self.write_to_file(&filepath, &log_content) {
            eprintln!("Failed to write game finished log to {}: {}", filepath.display(), e);
        }
    }

    // Log to `logs-tower/<tournament_id>.log`
    // example:
    // logs-tower/tournament123.log
    pub fn log_tournament_finished(&self, summary: &str) {
        let timestamp = Self::get_timestamp();
        let filename = format!("{}.log", self.tournament_id.as_str());
        let filepath = PathBuf::from(TOWER_LOGS_DIR).join(&filename);
        
        let log_content = format!(
            "Timestamp: {}\nTournament ID: {}\nStatus: TOURNAMENT FINISHED\n\n=== TOURNAMENT SUMMARY ===\n{}\n",
            timestamp,
            self.tournament_id.as_str(),
            summary
        );
        
        if let Err(e) = self.write_to_file(&filepath, &log_content) {
            eprintln!("Failed to write tournament finished log to {}: {}", filepath.display(), e);
        }
    }
}

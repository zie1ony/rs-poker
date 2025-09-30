use std::{fs, path::PathBuf};

use rs_poker_engine::{game_instance::GameInstance, tournament_instance::TournamentInstance};
use rs_poker_types::{game_event::GameEvent, tournament_event::TournamentEvent};
use serde::Serialize;
use thiserror::Error;

pub const STORAGE_DIR: &str = "data";
pub const STORAGE_GAMES_DIR: &str = "data/games";
pub const STORAGE_TOURNAMENTS_DIR: &str = "data/tournaments";

#[derive(Error, Debug)]
pub enum PersistenceError {
    #[error("Failed to create directory: {0}")]
    DirectoryCreation(#[from] std::io::Error),

    #[error("Failed to serialize data: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Failed to write file: {path}")]
    FileWrite {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

pub type PersistenceResult<T> = Result<T, PersistenceError>;

/// Store a game instance to persistent storage
pub fn store_game(game: &GameInstance) -> PersistenceResult<()> {
    let events = game.events();
    store_to_json(&events, STORAGE_GAMES_DIR, game.game_id.as_str())
}

/// Store a tournament instance to persistent storage
pub fn store_tournament(tournament: &TournamentInstance) -> PersistenceResult<()> {
    let events = tournament.events();
    store_to_json(
        &events,
        STORAGE_TOURNAMENTS_DIR,
        tournament.tournament_id.as_str(),
    )
}

pub fn load_games() -> PersistenceResult<Vec<GameInstance>> {
    let mut games = Vec::new();
    let dir = PathBuf::from(STORAGE_GAMES_DIR);

    if dir.exists() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let data = fs::read_to_string(&path)?;
                let events: Vec<GameEvent> = serde_json::from_str(&data)?;
                let game = GameInstance::from(events);
                games.push(game);
            }
        }
    }

    Ok(games)
}

pub fn load_tournaments() -> PersistenceResult<Vec<TournamentInstance>> {
    let mut tournaments = Vec::new();
    let dir = PathBuf::from(STORAGE_TOURNAMENTS_DIR);

    if dir.exists() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let data = fs::read_to_string(&path)?;
                let events: Vec<TournamentEvent> = serde_json::from_str(&data)?;
                let tournament = TournamentInstance::from(events);
                tournaments.push(tournament);
            }
        }
    }

    Ok(tournaments)
}

/// Generic function to store serializable data to a JSON file
fn store_to_json<T: Serialize>(data: &T, directory: &str, filename: &str) -> PersistenceResult<()> {
    // Ensure the storage directory exists
    fs::create_dir_all(directory)?;

    // Create the file path
    let filepath = PathBuf::from(directory).join(format!("{}.json", filename));

    // Serialize the data
    let json = serde_json::to_string_pretty(data)?;

    // Write to file
    fs::write(&filepath, json).map_err(|source| PersistenceError::FileWrite {
        path: filepath,
        source,
    })?;

    Ok(())
}

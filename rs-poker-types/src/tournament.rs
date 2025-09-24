use crate::{player::Player, random_id};

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct TournamentId(pub String);

impl TournamentId {
    pub fn new(id: &str) -> Self {
        Self(id.to_string())
    }

    pub fn random() -> Self {
        Self(random_id("tournament"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
pub struct TournamentSettings {
    pub tournament_id: TournamentId,
    pub players: Vec<Player>,
    pub starting_player_stack: f32,
    pub starting_small_blind: f32,
    pub double_blinds_every_n_games: Option<usize>,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
pub enum TournamentStatus {
    WaitingForNextGame,
    GameInProgress,
    Completed,
}

use std::fmt::Display;

use crate::{
    game::GameId,
    player::{Player, PlayerName},
    random_id,
};

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

impl Display for TournamentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
pub struct TournamentSettings {
    pub tournament_id: TournamentId,
    pub players: Vec<Player>,
    pub starting_player_stack: f32,
    pub starting_small_blind: f32,
    pub double_blinds_every_n_games: Option<usize>,
    pub end_condition: TournamentEndCondition,
    pub see_historical_thoughts: bool,
    pub public_chat: bool,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
pub enum TournamentStatus {
    WaitingForNextGame,
    GameInProgress,
    Completed,
}

impl TournamentStatus {
    pub fn is_completed(&self) -> bool {
        matches!(self, TournamentStatus::Completed)
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
pub struct TournamentInfo {
    pub settings: TournamentSettings,
    pub status: TournamentStatus,
    pub games_played: usize,
    pub current_game_id: Option<GameId>,
    pub winner: Option<PlayerName>,
}

impl TournamentInfo {
    pub fn tournament_id(&self) -> TournamentId {
        self.settings.tournament_id.clone()
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
pub enum TournamentEndCondition {
    SingleWinner,
    MaxGames(usize),
}

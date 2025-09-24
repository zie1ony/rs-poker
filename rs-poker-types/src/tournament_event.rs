use std::time::SystemTime;

use crate::{
    game::GameId,
    player::PlayerName,
    tournament::{TournamentId, TournamentSettings},
};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum TournamentEvent {
    TournamentCreated(TournamentCreatedEvent),
    GameStarted(GameStartedEvent),
    GameEnded(GameEndedEvent),
    TournamentFinished(TournamentFinishedEvent),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct TournamentCreatedEvent {
    pub timestamp: SystemTime,
    pub tournament_id: TournamentId,
    pub settings: TournamentSettings,
}

impl TournamentCreatedEvent {
    pub fn new(settings: &TournamentSettings) -> TournamentEvent {
        TournamentEvent::TournamentCreated(Self {
            timestamp: SystemTime::now(),
            tournament_id: settings.tournament_id.clone(),
            settings: settings.clone(),
        })
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct GameStartedEvent {
    pub timestamp: SystemTime,
    pub game_id: GameId,
    pub player_names: Vec<PlayerName>,
    pub player_stacks: Vec<f32>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct GameEndedEvent {
    pub timestamp: SystemTime,
    pub game_id: GameId,
    pub player_names: Vec<PlayerName>,
    pub player_stacks: Vec<f32>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct TournamentFinishedEvent {
    pub timestamp: SystemTime,
    pub tournament_id: TournamentId,
    pub winner: PlayerName,
}

use rs_poker::{
    arena::{action::AgentAction, game_state::Round},
    core::{Card, Hand, Rank},
};

use crate::{
    game::{Decision, GameId, GameSettings},
    player::PlayerName,
};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum GameEvent {
    GameStarted(GameStartedEvent),
    RoundAdvance(Round),
    ForcedBet(ForcedBetEvent),
    FailedPlayerAction(FailedPlayerActionEvent),
    PlayerAction(PlayerActionEvent),
    ShowCommunityCards(ShowCommunityCardsEvent),
    GameEnded(GameEndedEvent),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct GameStartedEvent {
    pub game_id: GameId,
    pub settings: GameSettings,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct ForcedBetEvent {
    pub bet_kind: ForcedBetKind,
    pub player_idx: usize,
    pub player_name: PlayerName,
    pub bet: f32,
    pub stack_after: f32,
    pub pot_after: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum ForcedBetKind {
    SmallBlind,
    BigBlind,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct FailedPlayerActionEvent {
    pub player_idx: usize,
    pub player_name: PlayerName,
    pub player_decision: Decision,
    pub action: AgentAction,
    pub stack_after: f32,
    pub pot_after: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct PlayerActionEvent {
    pub player_idx: usize,
    pub player_name: PlayerName,
    pub player_decision: Decision,
    pub stack_after: f32,
    pub pot_after: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct ShowCommunityCardsEvent {
    pub round: Round,
    pub cards: Vec<Card>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct GameEndedEvent {
    pub final_round: Round,
    pub awards: Vec<Award>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct Award {
    pub player_idx: usize,
    pub player_name: PlayerName,
    pub won_pot: f32,
    pub stack_after: f32,
    pub rank: Option<Rank>,
    pub hand: Option<Hand>,
}

use rs_poker::{
    arena::{action::AgentAction, game_state::Round},
    core::Card,
};

use crate::{
    game::{Decision, GameId},
    player::{Player, PlayerName},
};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum GameEvent {
    GameStarted(GameStartedEvent),
    RoundAdvance(Round),
    ForcedBet(ForcedBetEvent),
    FailedPlayerAction(FailedPlayerActionEvent),
    PlayerAction(PlayerActionEvent),
    ShowCommunityCards(ShowCommunityCardsEvent),
    GameEnded(GameEndedEvent),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct GameStartedEvent {
    pub game_id: GameId,
    pub players: Vec<Player>,
    pub initial_stacks: Vec<f32>,
    pub small_blind: f32,
    pub big_blind: f32,
    pub hands: Vec<[Card; 2]>,
    pub community_cards: [Card; 5],
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ForcedBetEvent {
    pub bet_kind: ForcedBetKind,
    pub player_idx: usize,
    pub player_name: PlayerName,
    pub bet: f32,
    pub stack_after: f32,
    pub pot_after: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum ForcedBetKind {
    SmallBlind,
    BigBlind,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct FailedPlayerActionEvent {
    pub player_idx: usize,
    pub player_name: PlayerName,
    pub player_decision: Decision,
    pub action: AgentAction,
    pub stack_after: f32,
    pub pot_after: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct PlayerActionEvent {
    pub player_idx: usize,
    pub player_name: PlayerName,
    pub player_decision: Decision,
    pub stack_after: f32,
    pub pot_after: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ShowCommunityCardsEvent {
    pub round: Round,
    pub cards: Vec<Card>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct GameEndedEvent {
    pub final_round: Round,
    pub awards: Vec<Award>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Award {
    pub player_idx: usize,
    pub player_name: PlayerName,
    pub won_pot: f32,
    pub stack_after: f32,
}

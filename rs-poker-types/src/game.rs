use rs_poker::arena::action::AgentAction;
use rs_poker_llm_client::LLMResponse;

use crate::{
    player::{Player, PlayerName},
    random_id,
    tournament::TournamentId,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct GameId(String);

impl GameId {
    pub fn new(id: &str) -> Self {
        Self(id.to_string())
    }

    pub fn random() -> Self {
        Self(random_id("game"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
pub enum GameStatus {
    InProgress,
    Finished,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
pub enum PossibleAction {
    Fold,
    Call,
    Bet { min: f32, max: f32 },
    AllIn,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, schemars::JsonSchema)]
pub struct Decision {
    pub action: AgentAction,
    pub reason: String,
}

impl LLMResponse for Decision {
    const DESCRIPTION: &'static str = "A poker decision with an action and a reason.";
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
pub struct GameFullView {
    pub game_id: GameId,
    pub status: GameStatus,
    pub summary: String,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
pub struct GamePlayerView {
    pub game_id: GameId,
    pub player: PlayerName,
    pub is_active_player: bool,
    pub summary: String,
    pub possible_actions: Vec<PossibleAction>,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
pub struct GameInfo {
    pub game_id: GameId,
    pub players: Vec<Player>,
    pub status: GameStatus,
    pub current_player_name: Option<PlayerName>,
}

impl GameInfo {
    pub fn current_player(&self) -> Option<&Player> {
        if let Some(name) = &self.current_player_name {
            self.players.iter().find(|p| p.name() == *name)
        } else {
            None
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
pub struct GameSettings {
    pub tournament_id: Option<TournamentId>,
    pub torunament_game_number: Option<usize>,
    pub game_id: GameId,
    pub small_blind: f32,
    pub players: Vec<Player>,
    pub stacks: Vec<f32>,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
pub struct GameFinalResults {
    pub game_id: GameId,
    pub player_names: Vec<PlayerName>,
    pub final_stacks: Vec<f32>,
}

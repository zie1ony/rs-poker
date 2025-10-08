use std::fmt::Display;

use rs_poker::{arena::action::AgentAction, core::Card};
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

    pub fn for_tournament(game_number: usize) -> Self {
        let prefix = format!("game_{}", game_number);
        Self(random_id(&prefix))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl Display for GameId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, schemars::JsonSchema)]
pub struct Decision {
    pub reason: String,
    pub action: AgentAction,
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

    pub fn is_finished(&self) -> bool {
        self.status == GameStatus::Finished
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
pub struct GameSettings {
    pub tournament_id: Option<TournamentId>,
    pub tournament_game_number: Option<usize>,
    pub game_id: Option<GameId>,
    pub small_blind: f32,
    pub players: Vec<Player>,
    pub stacks: Vec<f32>,
    pub hands: Option<Vec<[Card; 2]>>,
    pub community_cards: Option<[Card; 5]>,
    pub dealer_index: usize,
}

impl GameSettings {
    pub fn validate(&self) -> Result<(), String> {
        if self.players.len() < 2 {
            return Err("At least two players are required.".to_string());
        }
        if self.players.len() > 10 {
            return Err("A maximum of ten players is allowed.".to_string());
        }
        if self.players.len() != self.stacks.len() {
            return Err("The number of players must match the number of stacks.".to_string());
        }
        if self.small_blind <= 0.0 {
            return Err("Small blind must be greater than zero.".to_string());
        }
        // Dealer index must be valid
        if self.dealer_index >= self.players.len() {
            return Err("Dealer index is out of bounds.".to_string());
        }
        // Check unique player names
        let mut names = std::collections::HashSet::new();
        for player in &self.players {
            if !names.insert(player.name()) {
                return Err(format!("Duplicate player name found: {}", player.name()));
            }
        }
        // Check both hands and community cards if provided or neither
        match (&self.hands, &self.community_cards) {
            (Some(_), None) | (None, Some(_)) => {
                return Err(
                    "Both predefined hands and community cards must be provided together."
                        .to_string(),
                );
            }
            // Validate all cards are unique if predefined hands are provided
            (Some(hands), Some(community)) => {
                let mut used_cards = std::collections::HashSet::new();
                for hand in hands {
                    for &card in hand {
                        if !used_cards.insert(card) {
                            return Err(format!("Duplicate card found in hands: {:?}", card));
                        }
                    }
                }
                for &card in community {
                    if !used_cards.insert(card) {
                        return Err(format!(
                            "Duplicate card found in community cards: {:?}",
                            card
                        ));
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn big_blind(&self) -> f32 {
        self.small_blind * 2.0
    }

    pub fn player_names(&self) -> Vec<PlayerName> {
        self.players.iter().map(|p| p.name().clone()).collect()
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
pub struct GameFinalResults {
    pub game_id: GameId,
    pub player_names: Vec<PlayerName>,
    pub final_stacks: Vec<f32>,
}

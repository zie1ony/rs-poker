use std::fmt::Display;

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[serde(transparent)]
pub struct PlayerName(String);

impl PlayerName {
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }
}

impl Display for PlayerName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
pub enum AutomatType {
    Random,
    AllIn,
    Calling,
    Filding,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
pub enum Player {
    Automat {
        name: PlayerName,
        automat_type: AutomatType,
    },
    Human {
        name: PlayerName,
    },
    AI {
        name: PlayerName,
        model: String,
        strategy: String,
    },
}

impl Player {
    pub fn name(&self) -> PlayerName {
        match self {
            Player::Automat { name, .. } => name,
            Player::Human { name } => name,
            Player::AI { name, .. } => name,
        }
        .clone()
    }

    pub fn human(name: &str) -> Self {
        Player::Human {
            name: PlayerName::new(name),
        }
    }

    pub fn ai(name: &str, model: &str, strategy: &str) -> Self {
        Player::AI {
            name: PlayerName::new(name),
            model: model.to_string(),
            strategy: strategy.to_string(),
        }
    }

    pub fn random(name: &str) -> Self {
        Player::Automat {
            name: PlayerName::new(name),
            automat_type: AutomatType::Random,
        }
    }

    pub fn all_in(name: &str) -> Self {
        Player::Automat {
            name: PlayerName::new(name),
            automat_type: AutomatType::AllIn,
        }
    }

    pub fn is_human(&self) -> bool {
        matches!(self, Player::Human { .. })
    }

    pub fn is_ai(&self) -> bool {
        matches!(self, Player::AI { .. })
    }
}

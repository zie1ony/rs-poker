use super::action::Action;
use super::game_state::GameState;

pub trait Agent {
    fn act(&self, game_state: &GameState) -> Action;
}

pub struct FoldingAgent {}

impl Agent for FoldingAgent {
    fn act(&self, _: &GameState) -> Action {
        Action::Fold
    }
}

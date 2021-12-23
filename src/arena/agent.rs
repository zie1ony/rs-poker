use super::action::Action;
use super::game_state::GameState;

pub trait Agent {
    fn act(&self, game_state: &GameState) -> Action;
}

pub struct FoldingAgent {}

impl Agent for FoldingAgent {
    fn act(&self, game_state: &GameState) -> Action {
        if game_state.player_active.count_ones(..) == game_state.num_players {
            Action::Fold
        } else {
            Action::Check
        }
    }
}

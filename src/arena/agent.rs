use super::action::Action;
use super::game_state::GameState;

pub trait Agent {
    fn act(&self, game_state: &GameState) -> Action;
}

pub struct FoldingAgent {}

impl Agent for FoldingAgent {
    fn act(&self, game_state: &GameState) -> Action {
        match (
            game_state.current_round_data(),
            game_state.num_active_players(),
        ) {
            (Some(round), 1) => Action::Bet(round.bet),
            _ => Action::Fold,
        }
    }
}

pub struct CallingAgent {}

impl Agent for CallingAgent {
    fn act(&self, game_state: &GameState) -> Action {
        if let Some(round) = game_state.current_round_data() {
            Action::Bet(round.bet)
        } else {
            Action::Fold
        }
    }
}

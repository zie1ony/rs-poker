use crate::arena::{action::AgentAction, game_state::GameState};

use super::{Agent, AgentGenerator};

/// A simple agent that folds unless there is only one active player left.
#[derive(Default, Debug, Clone, Copy)]
pub struct FoldingAgent;

impl Agent for FoldingAgent {
    fn act(self: &mut FoldingAgent, _id: u128, game_state: &GameState) -> AgentAction {
        let count = game_state.current_round_num_active_players() + game_state.num_all_in_players();
        if count == 1 {
            AgentAction::Bet(game_state.current_round_bet())
        } else {
            AgentAction::Fold
        }
    }
}

/// Default Generator for `FoldingAgent`.
#[derive(Debug, Clone, Copy, Default)]
pub struct FoldingAgentGenerator;

impl AgentGenerator for FoldingAgentGenerator {
    fn generate(&self, _game_state: &GameState) -> Box<dyn Agent> {
        Box::new(FoldingAgent)
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use rand::{SeedableRng, rngs::StdRng};

    use crate::arena::{HoldemSimulationBuilder, game_state::Round};

    use super::*;

    #[test_log::test]
    fn test_folding_agents() {
        let stacks = vec![100.0; 2];
        let mut rng = StdRng::seed_from_u64(420);

        let game_state = GameState::new_starting(stacks, 10.0, 5.0, 0.0, 0);
        let mut sim = HoldemSimulationBuilder::default()
            .game_state(game_state)
            .agents(vec![Box::new(FoldingAgent {}), Box::new(FoldingAgent {})])
            .build()
            .unwrap();

        sim.run(&mut rng);

        assert_eq!(sim.game_state.num_active_players(), 1);
        assert_eq!(sim.game_state.round, Round::Complete);

        assert_relative_eq!(15.0_f32, sim.game_state.player_bet.iter().sum());

        assert_relative_eq!(15.0_f32, sim.game_state.player_winnings.iter().sum());
        assert_relative_eq!(15.0_f32, sim.game_state.player_winnings[1]);
    }
}

use crate::core::{Deck, FlatDeck};

use super::agent::Agent;

use super::game_state::GameState;

pub struct HoldemSimulation {
    agents: Vec<Box<dyn Agent>>,
    game_state: GameState,
    deck: FlatDeck,
}

impl HoldemSimulation {
    pub fn new(game_state: GameState) -> Self {
        let mut d = Deck::default();

        for hand in game_state.hands.iter() {
            for card in hand.iter() {
                d.remove(card);
            }
        }

        Self {
            game_state,
            deck: d.into(),
            agents: vec![],
        }
    }

    pub fn step(&mut self) {
        match self.game_state.round {
            super::game_state::Round::Starting => self.start(),
            _ => todo!(),
        }
    }

    fn start(&mut self) {
        self.game_state.deal_preflop(&mut self.deck);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_step_agent() {
        let game_state = GameState::new(vec![100, 40], 10, 5);
        let mut sim = HoldemSimulation::new(game_state);

        // We are starting out.
        sim.step();
        // Now run preflop
    }
}

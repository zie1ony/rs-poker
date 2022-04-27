use core::fmt;

use crate::arena::game_state::Round;
use crate::core::{Card, Deck, FlatDeck};

use super::action::Action;
use super::agent::Agent;

use super::game_state::GameState;

pub struct HoldemSimulation {
    agents: Vec<Box<dyn Agent>>,
    game_state: GameState,
    deck: FlatDeck,
}

impl HoldemSimulation {
    pub fn new(game_state: GameState) -> Self {
        HoldemSimulation::new_with_agents(game_state, vec![])
    }

    pub fn new_with_agents(game_state: GameState, agents: Vec<Box<dyn Agent>>) -> Self {
        let mut d = Deck::default();

        for hand in game_state.hands.iter() {
            for card in hand.iter() {
                d.remove(card);
            }
        }

        let mut flat_deck: FlatDeck = d.into();
        flat_deck.shuffle();

        Self {
            game_state,
            agents,
            deck: flat_deck,
        }
    }

    pub fn more_rounds(&self) -> bool {
        !matches!(self.game_state.round, Round::Showdown)
    }

    pub fn step(&mut self) {
        match self.game_state.round {
            Round::Starting => self.start(),
            Round::Preflop => self.preflop(),
            Round::Flop => self.flop(),
            Round::Turn => self.turn(),
            Round::River => self.river(),
            _ => (),
        }
    }

    fn start(&mut self) {
        for h in &mut self.game_state.hands {
            h.push(self.deck.deal().unwrap());
            h.push(self.deck.deal().unwrap());
        }
        self.game_state.advance_round().unwrap();
    }

    fn preflop(&mut self) {
        self.run_betting_round();
        self.game_state.advance_round().unwrap()
    }

    fn flop(&mut self) {
        self.deal_comunity(3);
        self.run_betting_round();
        self.game_state.advance_round().unwrap()
    }

    fn turn(&mut self) {
        self.deal_comunity(1);
        self.run_betting_round();
        self.game_state.advance_round().unwrap()
    }

    fn river(&mut self) {
        self.deal_comunity(1);
        self.run_betting_round();
        self.game_state.advance_round().unwrap()
    }

    fn deal_comunity(&mut self, num_cards: usize) {
        let mut community_cards: Vec<Card> =
            (0..num_cards).map(|_| self.deck.deal().unwrap()).collect();
        // Add all the cards to the hands as well.
        for h in &mut self.game_state.hands {
            for c in &community_cards {
                // push a copy
                h.push(*c);
            }
        }
        // Drain the community_cards vec into the game_state board.
        self.game_state.board.append(&mut community_cards);
    }

    fn run_betting_round(&mut self) {
        while self.game_state.num_active_players_in_round() > 0 {
            let round = self.game_state.current_round_data().unwrap();
            let idx = round.to_act_idx;
            let action = self.agents[idx].act(&self.game_state);
            self.run_action(action)
        }
    }

    fn run_action(&mut self, action: Action) {
        match action {
            Action::Bet(bet_ammount) => {
                let result = self.game_state.do_bet(bet_ammount, false);
                if result.is_err() {
                    self.game_state.fold().unwrap();
                }
            }
            Action::Fold => self.game_state.fold().unwrap(),
        }
    }
}

impl fmt::Debug for HoldemSimulation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HoldemSimulation")
            .field("game_state", &self.game_state)
            .field("deck", &self.deck)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::arena::agent::CallingAgent;

    use super::*;

    #[test]
    fn test_single_step_agent() {
        let stacks = vec![100; 9];
        let game_state = GameState::new(stacks, 10, 5, 0);
        let mut sim = HoldemSimulation::new(game_state);

        assert_eq!(100, sim.game_state.stacks[1]);
        // We are starting out.
        sim.step();
        // assert that blinds are there
        assert_eq!(95, sim.game_state.stacks[1]);
        assert_eq!(90, sim.game_state.stacks[2]);
    }

    #[test]
    fn test_call_agents() {
        let stacks = vec![100; 4];
        let game_state = GameState::new(stacks, 10, 5, 0);
        let mut sim = HoldemSimulation::new_with_agents(
            game_state,
            vec![
                Box::new(CallingAgent {}),
                Box::new(CallingAgent {}),
                Box::new(CallingAgent {}),
                Box::new(CallingAgent {}),
            ],
        );

        while sim.more_rounds() {
            sim.step();
        }

        assert_eq!(sim.game_state.num_active_players(), 4);
    }
}

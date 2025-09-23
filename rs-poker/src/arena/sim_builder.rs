use rand::Rng;

use crate::core::{CardBitSet, Deck};

use super::{
    Agent, GameState, HoldemSimulation, agent::FoldingAgent, errors::HoldemSimulationError,
    historian::Historian,
};

// Some builder methods to help with turning a builder struct into a ready
// simulation
fn build_deck(game_state: &GameState) -> Deck {
    let mut d = CardBitSet::default();

    for hand in game_state.hands.iter() {
        let bitset: CardBitSet = (*hand).into();

        d &= !bitset; // remove the cards in the hand from the deck
    }
    for card in game_state.board.iter() {
        d.remove(*card); // remove the cards on the board from the deck
    }

    d.into() // convert the bitset into a deck
}

fn build_agents(num_agents: usize) -> Vec<Box<dyn Agent>> {
    (0..num_agents)
        .map(|_| -> Box<dyn Agent> { Box::<FoldingAgent>::default() })
        .collect()
}

/// # HoldemSimulationBuilder
///
/// `RngHoldemSimulationBuilder` is a builder to allow for complex
/// configurations of a holdem simulation played via agents. A game state is
/// required, other fields are optional.
///
/// `HolemSimulationBuilder` is a type alias
/// for `RngHoldemSimulationBuilder<ThreadRng>` which is the default builder.
///
/// ## Setters
///
/// Each setter will set the optional value to the passed in value. Then return
/// the mutated builder.
///
/// While agents are not required the default is a full ring of folding agents.
/// So likely not that interesting a simulation.
///
/// ## Examples
///
/// ```
/// use rs_poker::arena::{GameState, HoldemSimulationBuilder};
///
/// let game_state = GameState::new_starting(vec![100.0; 5], 2.0, 1.0, 0.0, 3);
/// let sim = HoldemSimulationBuilder::default()
///     .game_state(game_state)
///     .build()
///     .unwrap();
/// ```
/// However sometimes you want to use a known but random simulation. In that
/// case you can pass in the rng like this:
///
/// ```
/// use rand::{SeedableRng, rngs::StdRng};
/// use rs_poker::arena::{GameState, HoldemSimulationBuilder};
///
/// let game_state = GameState::new_starting(vec![100.0; 5], 2.0, 1.0, 0.0, 3);
/// let sim = HoldemSimulationBuilder::default()
///     .game_state(game_state)
///     .build()
///     .unwrap();
/// ```
pub struct HoldemSimulationBuilder {
    agents: Option<Vec<Box<dyn Agent>>>,
    historians: Vec<Box<dyn Historian>>,
    game_state: Option<GameState>,
    deck: Option<Deck>,
    panic_on_historian_error: bool,
}

/// # Examples
/// ```
/// use rand::{SeedableRng, rngs::StdRng};
/// use rs_poker::arena::{Agent, agent::FoldingAgent};
/// use rs_poker::arena::{GameState, HoldemSimulationBuilder};
///
/// let game_state = GameState::new_starting(vec![100.0; 5], 2.0, 1.0, 0.0, 3);
/// let agents: Vec<Box<dyn Agent>> = (0..5)
///     .map(|_| Box::<FoldingAgent>::default() as Box<dyn Agent>)
///     .collect();
/// let sim = HoldemSimulationBuilder::default()
///     .game_state(game_state)
///     .agents(agents)
///     .build();
/// ```
impl HoldemSimulationBuilder {
    /// Set the agents for the simulation created by this builder.
    pub fn agents(mut self, agents: Vec<Box<dyn Agent>>) -> Self {
        self.agents = Some(agents);
        self
    }

    /// Set the game state for ths simulation created by this bu
    pub fn game_state(mut self, game_state: GameState) -> Self {
        self.game_state = Some(game_state);
        self
    }

    /// Set the deck. If not set a deck will be
    /// created from the game state and shuffled.
    pub fn deck(mut self, deck: Deck) -> Self {
        self.deck = Some(deck);
        self
    }

    /// Set the historians for the simulation created by this builder.
    pub fn historians(mut self, historians: Vec<Box<dyn Historian>>) -> Self {
        self.historians = historians;
        self
    }

    /// Should the simulation panic if a historian errors.
    /// Default is false and allows the simulation to continue if a historian
    /// errors. It will be removed from the simulation and recorded in the logs.
    pub fn panic_on_historian_error(mut self, panic_on_historian_error: bool) -> Self {
        self.panic_on_historian_error = panic_on_historian_error;
        self
    }

    /// Given the fields already specified build any that are not specified and
    /// create a new HoldemSimulation.
    ///
    /// @returns HoldemSimulationError if no game_state was given.
    pub fn build(self) -> Result<HoldemSimulation, HoldemSimulationError> {
        let game_state = self
            .game_state
            .ok_or(HoldemSimulationError::NeedGameState)?;

        let agents = self
            .agents
            .unwrap_or_else(|| build_agents(game_state.hands.len()));

        let agent_historians = agents.iter().filter_map(|a| a.historian());

        // Add the agent historians to the simulation
        // historians.
        let historians: Vec<_> = self
            .historians
            .into_iter()
            .chain(agent_historians)
            .collect();

        let deck = self.deck.unwrap_or_else(|| build_deck(&game_state));

        // Create a new simulation id.
        // This will be used to track
        // this exact run of a simulation.
        let mut rand = rand::rng();
        let id = rand.random::<u128>();

        Ok(HoldemSimulation {
            agents,
            game_state,
            deck,
            id,
            historians,
            panic_on_historian_error: self.panic_on_historian_error,
        })
    }
}

impl Default for HoldemSimulationBuilder {
    fn default() -> Self {
        Self {
            agents: None,
            historians: vec![],
            game_state: None,
            deck: None,
            panic_on_historian_error: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::{SeedableRng, rngs::StdRng};

    use crate::{arena::game_state::Round, core::Card};

    use super::*;

    #[test_log::test]
    fn test_single_step_agent() {
        let mut rng = StdRng::seed_from_u64(420);
        let stacks = vec![100.0; 9];
        let game_state = GameState::new_starting(stacks, 10.0, 5.0, 1.0, 0);
        let mut sim = HoldemSimulationBuilder::default()
            .game_state(game_state)
            .build()
            .unwrap();

        assert_eq!(100.0, sim.game_state.stacks[1]);
        assert_eq!(100.0, sim.game_state.stacks[2]);
        // We are starting out.
        sim.run_round(&mut rng);
        assert_eq!(100.0, sim.game_state.stacks[1]);
        assert_eq!(100.0, sim.game_state.stacks[2]);

        // Post the ante and check the results.
        sim.run_round(&mut rng);
        for i in 0..9 {
            assert_eq!(99.0, sim.game_state.stacks[i]);
        }

        // Deal Pre-Flop
        sim.run_round(&mut rng);

        // Post the blinds and check the results.
        sim.run_round(&mut rng);
        assert_eq!(6.0, sim.game_state.player_bet[1]);
        assert_eq!(11.0, sim.game_state.player_bet[2]);
    }

    // #[test_log::test]
    // fn test_flatdeck_order() {
    //     let stacks = vec![100.0; 2];
    //     let game_state = GameState::new_starting(stacks, 10.0, 5.0, 0.0, 0);

    //     let rng_one = StdRng::seed_from_u64(420);
    //     let sim_one = RngHoldemSimulationBuilder::default()
    //         .rng(rng_one)
    //         .game_state(game_state.clone())
    //         .build()
    //         .unwrap();

    //     let rng_two = StdRng::seed_from_u64(420);
    //     let sim_two = RngHoldemSimulationBuilder::default()
    //         .rng(rng_two)
    //         .game_state(game_state)
    //         .build()
    //         .unwrap();

    //     assert_eq!(sim_two.deck[..], sim_one.deck[..]);
    // }

    #[test_log::test]
    fn test_simulation_complex_showdown() {
        let stacks = vec![102.0, 7.0, 12.0, 102.0, 202.0];
        let mut game_state = GameState::new_starting(stacks, 10.0, 5.0, 2.0, 0);
        let mut deck = CardBitSet::default();
        let mut rng = rand::rng();

        // Start
        game_state.advance_round();

        // Ante
        game_state.do_bet(2.0, true).unwrap(); // ante@idx 1
        game_state.do_bet(2.0, true).unwrap(); // ante@idx 2
        game_state.do_bet(2.0, true).unwrap(); // ante@idx 3
        game_state.do_bet(2.0, true).unwrap(); // ante@idx 4
        game_state.do_bet(2.0, true).unwrap(); // ante@idx 0
        game_state.advance_round();

        // Deal Preflop
        deal_hand_card(0, "Ks", &mut deck, &mut game_state);
        deal_hand_card(0, "Kh", &mut deck, &mut game_state);

        deal_hand_card(1, "As", &mut deck, &mut game_state);
        deal_hand_card(1, "Ac", &mut deck, &mut game_state);

        deal_hand_card(2, "Ad", &mut deck, &mut game_state);
        deal_hand_card(2, "Ah", &mut deck, &mut game_state);

        deal_hand_card(3, "6d", &mut deck, &mut game_state);
        deal_hand_card(3, "4d", &mut deck, &mut game_state);

        deal_hand_card(4, "9d", &mut deck, &mut game_state);
        deal_hand_card(4, "9s", &mut deck, &mut game_state);
        game_state.advance_round();

        // Preflop
        game_state.do_bet(5.0, true).unwrap(); // blinds@idx 1
        game_state.do_bet(10.0, true).unwrap(); // blinds@idx 2
        game_state.fold(); // idx 3
        game_state.do_bet(10.0, false).unwrap(); // idx 4
        game_state.do_bet(10.0, false).unwrap(); // idx 0
        game_state.advance_round();

        // Deal Flop
        deal_community_card("6c", &mut deck, &mut game_state);
        deal_community_card("2d", &mut deck, &mut game_state);
        deal_community_card("3d", &mut deck, &mut game_state);
        game_state.advance_round();

        // Flop
        assert_eq!(game_state.num_active_players(), 2);
        game_state.do_bet(90.0, false).unwrap(); // idx 4
        game_state.do_bet(90.0, false).unwrap(); // idx 0
        game_state.advance_round();
        assert_eq!(game_state.num_active_players(), 1);

        // Deal Turn
        deal_community_card("8h", &mut deck, &mut game_state);
        game_state.advance_round();

        // Turn
        game_state.do_bet(0.0, false).unwrap(); // idx 4
        game_state.advance_round();
        assert_eq!(game_state.num_active_players(), 1);

        // Deal River
        deal_community_card("8s", &mut deck, &mut game_state);
        game_state.advance_round();

        // River
        game_state.do_bet(100.0, false).unwrap(); // idx 4
        game_state.advance_round();
        assert_eq!(game_state.num_active_players(), 0);

        let mut sim = HoldemSimulationBuilder::default()
            .game_state(game_state)
            .build()
            .unwrap();
        sim.run(&mut rng);

        assert_eq!(Round::Complete, sim.game_state.round);

        assert_eq!(180.0, sim.game_state.player_winnings[0]);
        assert_eq!(15.0, sim.game_state.player_winnings[1]);
        assert_eq!(30.0, sim.game_state.player_winnings[2]);
        assert_eq!(0.0, sim.game_state.player_winnings[3]);
        assert_eq!(100.0, sim.game_state.player_winnings[4]);

        assert_eq!(180.0, sim.game_state.stacks[0]);
        assert_eq!(15.0, sim.game_state.stacks[1]);
        assert_eq!(30.0, sim.game_state.stacks[2]);
        assert_eq!(100.0, sim.game_state.stacks[3]);
        assert_eq!(100.0, sim.game_state.stacks[4]);
    }

    fn deal_hand_card(
        idx: usize,
        card_str: &str,
        deck: &mut CardBitSet,
        game_state: &mut GameState,
    ) {
        let card = Card::try_from(card_str).unwrap();
        assert!(deck.contains(card));
        deck.remove(card);
        game_state.hands[idx].insert(card);
    }

    fn deal_community_card(card_str: &str, deck: &mut CardBitSet, game_state: &mut GameState) {
        let card = Card::try_from(card_str).unwrap();
        assert!(deck.contains(card));
        deck.remove(card);
        for h in &mut game_state.hands {
            h.insert(card);
        }

        game_state.board.push(card);
    }
}

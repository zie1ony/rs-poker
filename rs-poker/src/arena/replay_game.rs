use crate::arena::{
    Agent, GameState, HoldemSimulation, HoldemSimulationBuilder,
    action::Action,
    action::{
        AwardPayload, DealStartingHandPayload, ForcedBetPayload, GameStartPayload,
        PlayedActionPayload, PlayerSitPayload,
    },
};
use crate::core::{Card, Deck};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReplayError {
    #[error("Invalid action sequence")]
    InvalidActionSequence,
    #[error("Action index out of bounds")]
    ActionIndexOutOfBounds,
    #[error("Game state inconsistency")]
    GameStateInconsistency,
    #[error("Missing initial game state")]
    MissingInitialGameState,
}

/// A system for replaying poker games from recorded actions.
/// This ensures the exact same cards are dealt and the same game progression
/// occurs.
#[derive(Debug, Clone)]
pub struct GameReplay {
    /// All recorded actions from the original game
    actions: Vec<Action>,
    /// Current game state after applying actions up to current_action_index
    current_state: GameState,
    /// Index of the next action to be applied
    current_action_index: usize,
    /// Initial game state before any actions were applied
    initial_state: GameState,
}

impl GameReplay {
    /// Create a new replay from recorded actions and initial state
    pub fn new(initial_state: GameState, actions: Vec<Action>) -> Self {
        Self {
            actions,
            current_state: initial_state.clone(),
            current_action_index: 0,
            initial_state,
        }
    }

    /// Create a replay from just actions, extracting initial state from
    /// GameStart action
    pub fn from_actions(actions: Vec<Action>) -> Result<Self, ReplayError> {
        // Find the GameStart action to extract initial parameters
        let game_start = actions
            .iter()
            .find_map(|action| match action {
                Action::GameStart(payload) => Some(payload),
                _ => None,
            })
            .ok_or(ReplayError::MissingInitialGameState)?;

        // Extract player sit actions to get initial stacks
        let mut player_stacks = Vec::new();
        let dealer_idx = 0;

        for action in &actions {
            match action {
                Action::PlayerSit(PlayerSitPayload { idx, player_stack }) => {
                    // Resize vector if needed
                    if player_stacks.len() <= *idx {
                        player_stacks.resize(*idx + 1, 0.0);
                    }
                    player_stacks[*idx] = *player_stack;

                    // For simplicity, assume dealer is at index 0
                    // You might want to extract this from other actions if
                    // needed
                }
                _ => {}
            }
        }

        if player_stacks.is_empty() {
            return Err(ReplayError::MissingInitialGameState);
        }

        let initial_state = GameState::new_starting(
            player_stacks,
            game_start.big_blind,
            game_start.small_blind,
            game_start.ante,
            dealer_idx,
        );

        Ok(Self::new(initial_state, actions))
    }

    /// Apply the next action and advance the replay
    pub fn step_forward(&mut self) -> Result<Option<Action>, ReplayError> {
        if self.current_action_index >= self.actions.len() {
            return Ok(None);
        }

        let action = self.actions[self.current_action_index].clone();
        self.apply_action(&action)?;
        self.current_action_index += 1;

        Ok(Some(action))
    }

    /// Step to a specific action index
    pub fn step_to(&mut self, target_index: usize) -> Result<(), ReplayError> {
        if target_index > self.actions.len() {
            return Err(ReplayError::ActionIndexOutOfBounds);
        }

        // If we need to go backwards, reset to initial state
        if target_index < self.current_action_index {
            self.reset_to_start();
        }

        // Apply actions until we reach the target
        while self.current_action_index < target_index {
            self.step_forward()?;
        }

        Ok(())
    }

    /// Reset to the initial state
    pub fn reset_to_start(&mut self) {
        self.current_state = self.initial_state.clone();
        self.current_action_index = 0;
    }

    /// Get the current game state
    pub fn get_current_state(&self) -> &GameState {
        &self.current_state
    }

    /// Get the current action index
    pub fn get_current_action_index(&self) -> usize {
        self.current_action_index
    }

    /// Get all actions
    pub fn get_actions(&self) -> &[Action] {
        &self.actions
    }

    /// Check if there are more actions to replay
    pub fn has_more_actions(&self) -> bool {
        self.current_action_index < self.actions.len()
    }

    /// Apply a single action to the current game state
    fn apply_action(&mut self, action: &Action) -> Result<(), ReplayError> {
        match action {
            Action::GameStart(_) => {
                // Game start is already handled in initialization
            }
            Action::PlayerSit(_) => {
                // Player sitting is already handled in initialization
            }
            Action::DealStartingHand(DealStartingHandPayload { card, idx }) => {
                // Add the card to the player's hand
                if *idx < self.current_state.hands.len() {
                    self.current_state.hands[*idx].insert(*card);
                }
            }
            Action::DealCommunity(card) => {
                // Add the community card to the board
                self.current_state.board.push(*card);

                // Also add to all player hands (as the original simulation does)
                for hand in &mut self.current_state.hands {
                    hand.insert(*card);
                }
            }
            Action::RoundAdvance(round) => {
                self.current_state.round = *round;
            }
            Action::PlayedAction(payload) => {
                self.apply_played_action(payload)?;
            }
            Action::ForcedBet(payload) => {
                self.apply_forced_bet(payload)?;
            }
            Action::Award(payload) => {
                self.apply_award(payload)?;
            }
            Action::FailedAction(_) => {
                // Failed actions don't change game state
            }
        }
        Ok(())
    }

    fn apply_played_action(&mut self, payload: &PlayedActionPayload) -> Result<(), ReplayError> {
        // Update the game state based on the played action
        self.current_state.total_pot = payload.final_pot;
        self.current_state.player_bet[payload.idx] = payload.final_player_bet;
        self.current_state.round_data.bet = payload.final_bet;
        self.current_state.round_data.min_raise = payload.final_min_raise;
        self.current_state.player_active = payload.players_active;
        self.current_state.player_all_in = payload.players_all_in;

        // Update stack (pot increase comes from player stacks)
        let pot_increase = payload.final_pot - payload.starting_pot;
        if pot_increase > 0.0 {
            self.current_state.stacks[payload.idx] -= pot_increase;
        }

        Ok(())
    }

    fn apply_forced_bet(&mut self, payload: &ForcedBetPayload) -> Result<(), ReplayError> {
        // Apply forced bet (ante, small blind, big blind)
        self.current_state.stacks[payload.idx] = payload.player_stack;
        self.current_state.total_pot += payload.bet;
        Ok(())
    }

    fn apply_award(&mut self, payload: &AwardPayload) -> Result<(), ReplayError> {
        // Award money to the player
        self.current_state.stacks[payload.idx] += payload.award_amount;
        self.current_state.player_winnings[payload.idx] += payload.award_amount;
        Ok(())
    }
}

/// Create a simulation that will replay exact cards from recorded actions
pub struct ReplaySimulationBuilder {
    actions: Vec<Action>,
    initial_state: Option<GameState>,
}

impl ReplaySimulationBuilder {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
            initial_state: None,
        }
    }

    pub fn with_actions(mut self, actions: Vec<Action>) -> Self {
        self.actions = actions;
        self
    }

    pub fn with_initial_state(mut self, state: GameState) -> Self {
        self.initial_state = Some(state);
        self
    }

    /// Build a replay that can be stepped through action by action
    pub fn build_replay(self) -> Result<GameReplay, ReplayError> {
        if let Some(initial_state) = self.initial_state {
            Ok(GameReplay::new(initial_state, self.actions))
        } else {
            GameReplay::from_actions(self.actions)
        }
    }

    /// Build a full simulation that replays with predetermined cards and agents
    pub fn build_simulation_with_agents(
        self,
        agents: Vec<Box<dyn Agent>>,
    ) -> Result<HoldemSimulation, ReplayError> {
        let replay = self.build_replay()?;

        // Create a deck that will deal the exact same cards as recorded
        let predetermined_deck = PredetreminedDeck::from_actions(&replay.actions);

        let simulation = HoldemSimulationBuilder::default()
            .game_state(replay.initial_state.clone())
            .agents(agents)
            .deck(predetermined_deck.into())
            .build()
            .map_err(|_| ReplayError::GameStateInconsistency)?;

        Ok(simulation)
    }
}

/// A deck that deals cards in a predetermined order based on recorded actions
struct PredetreminedDeck {
    cards_to_deal: Vec<Card>,
}

impl PredetreminedDeck {
    fn from_actions(actions: &[Action]) -> Self {
        let mut cards = Vec::new();

        for action in actions {
            match action {
                Action::DealStartingHand(DealStartingHandPayload { card, .. }) => {
                    cards.push(*card);
                }
                Action::DealCommunity(card) => {
                    cards.push(*card);
                }
                _ => {}
            }
        }

        Self {
            cards_to_deal: cards,
        }
    }
}

impl From<PredetreminedDeck> for Deck {
    fn from(predetermined: PredetreminedDeck) -> Self {
        let mut deck = Deck::new();

        // Add cards in reverse order since Deck.deal() pops from the end
        for card in predetermined.cards_to_deal.into_iter().rev() {
            deck.insert(card);
        }

        deck
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arena::{
        HoldemSimulationBuilder,
        action::AgentAction,
        agent::{Agent, VecReplayAgent},
        historian::VecHistorian,
    };
    use crate::core::{Card, Suit, Value};
    use rand::{SeedableRng, rngs::StdRng};

    #[test]
    fn test_replay_basic() {
        let initial_state = GameState::new_starting(vec![100.0, 100.0], 10.0, 5.0, 0.0, 0);

        let actions = vec![
            Action::GameStart(GameStartPayload {
                ante: 0.0,
                small_blind: 5.0,
                big_blind: 10.0,
            }),
            Action::DealStartingHand(DealStartingHandPayload {
                card: Card::new(Value::Ace, Suit::Spade),
                idx: 0,
            }),
            Action::DealStartingHand(DealStartingHandPayload {
                card: Card::new(Value::King, Suit::Heart),
                idx: 0,
            }),
        ];

        let mut replay = GameReplay::new(initial_state, actions);

        // Step through actions
        replay.step_forward().unwrap();
        replay.step_forward().unwrap();
        replay.step_forward().unwrap();

        // Check that the cards were dealt correctly
        assert_eq!(replay.current_state.hands[0].count(), 2);
        assert!(replay.current_state.hands[0].contains(&Card::new(Value::Ace, Suit::Spade)));
        assert!(replay.current_state.hands[0].contains(&Card::new(Value::King, Suit::Heart)));
    }

    #[test]
    fn test_game_replay_creation_and_state() {
        let stacks = vec![100.0, 100.0];
        let initial_state = GameState::new_starting(stacks.clone(), 10.0, 5.0, 0.0, 0);

        let actions = vec![Action::GameStart(GameStartPayload {
            ante: 0.0,
            small_blind: 5.0,
            big_blind: 10.0,
        })];

        let replay = GameReplay::new(initial_state.clone(), actions.clone());

        // Test initial state
        assert_eq!(replay.get_current_action_index(), 0);
        assert!(replay.has_more_actions());
        assert_eq!(replay.get_actions().len(), 1);
        assert_eq!(replay.get_current_state().stacks, initial_state.stacks);
        assert_eq!(replay.get_current_state().total_pot, 0.0);
    }

    #[test]
    fn test_replay_step_through_actions() {
        let initial_state = GameState::new_starting(vec![100.0, 100.0], 10.0, 5.0, 0.0, 0);

        let actions = vec![
            Action::GameStart(GameStartPayload {
                ante: 0.0,
                small_blind: 5.0,
                big_blind: 10.0,
            }),
            Action::DealStartingHand(DealStartingHandPayload {
                card: Card::new(Value::Ace, Suit::Spade),
                idx: 0,
            }),
            Action::DealCommunity(Card::new(Value::King, Suit::Heart)),
        ];

        let mut replay = GameReplay::new(initial_state, actions.clone());

        // Step through each action
        let mut step_count = 0;
        while replay.has_more_actions() {
            let step_result = replay.step_forward();
            assert!(step_result.is_ok());
            step_count += 1;
            assert!(step_count <= actions.len());
        }

        assert_eq!(step_count, actions.len());
        assert!(!replay.has_more_actions());
        assert_eq!(replay.get_current_action_index(), actions.len());
    }

    #[test]
    fn test_time_travel_functionality() {
        let initial_state = GameState::new_starting(vec![100.0, 100.0], 10.0, 5.0, 0.0, 0);

        let actions = vec![
            Action::GameStart(GameStartPayload {
                ante: 0.0,
                small_blind: 5.0,
                big_blind: 10.0,
            }),
            Action::DealStartingHand(DealStartingHandPayload {
                card: Card::new(Value::Ace, Suit::Spade),
                idx: 0,
            }),
            Action::DealStartingHand(DealStartingHandPayload {
                card: Card::new(Value::King, Suit::Heart),
                idx: 1,
            }),
            Action::DealCommunity(Card::new(Value::Queen, Suit::Diamond)),
        ];

        let mut replay = GameReplay::new(initial_state.clone(), actions);

        // Jump to middle action
        let mid_action = 2;
        let jump_result = replay.step_to(mid_action);
        assert!(jump_result.is_ok());
        assert_eq!(replay.get_current_action_index(), mid_action);

        // Jump to early action (backward)
        let early_action = 1;
        let early_result = replay.step_to(early_action);
        assert!(early_result.is_ok());
        assert_eq!(replay.get_current_action_index(), early_action);

        // Jump to end
        let end_result = replay.step_to(4);
        assert!(end_result.is_ok());
        assert_eq!(replay.get_current_action_index(), 4);

        // Reset to start
        replay.reset_to_start();
        assert_eq!(replay.get_current_action_index(), 0);
        assert_eq!(replay.get_current_state().stacks, initial_state.stacks);
    }

    #[test]
    fn test_boundary_conditions() {
        let initial_state = GameState::new_starting(vec![100.0, 100.0], 10.0, 5.0, 0.0, 0);
        let actions = vec![Action::GameStart(GameStartPayload {
            ante: 0.0,
            small_blind: 5.0,
            big_blind: 10.0,
        })];

        let mut replay = GameReplay::new(initial_state, actions);

        // Test invalid action index
        let invalid_jump = replay.step_to(10);
        assert!(invalid_jump.is_err());

        // Test stepping beyond available actions
        replay.step_forward().unwrap(); // Valid step
        let beyond_result = replay.step_forward();
        assert_eq!(beyond_result.unwrap(), None); // No more actions
    }

    #[test]
    fn test_exact_card_reproduction() {
        // Run a full simulation with historian
        let historian = Box::new(VecHistorian::new());
        let records_storage = historian.get_storage();

        let stacks = vec![100.0, 100.0];
        let agents: Vec<Box<dyn Agent>> = vec![
            Box::new(VecReplayAgent::new(vec![
                AgentAction::Call,
                AgentAction::Call,
            ])),
            Box::new(VecReplayAgent::new(vec![
                AgentAction::Call,
                AgentAction::Call,
            ])),
        ];

        let game_state = GameState::new_starting(stacks.clone(), 10.0, 5.0, 0.0, 0);
        let mut rng = StdRng::seed_from_u64(12345);

        let mut sim = HoldemSimulationBuilder::default()
            .game_state(game_state.clone())
            .agents(agents)
            .historians(vec![historian])
            .build()
            .unwrap();

        sim.run(&mut rng);

        // Store original results
        let original_board = sim.game_state.board.clone();
        let original_hands = sim.game_state.hands.clone();
        let original_total_pot = sim.game_state.total_pot;
        let original_stacks = sim.game_state.stacks.clone();

        // Extract actions and create replay
        let records = records_storage.borrow();
        let actions: Vec<Action> = records.iter().map(|r| r.action.clone()).collect();
        drop(records);

        let initial_state = GameState::new_starting(stacks, 10.0, 5.0, 0.0, 0);
        let mut replay = GameReplay::new(initial_state, actions);

        // Step through entire replay
        while replay.has_more_actions() {
            replay.step_forward().unwrap();
        }

        // Verify exact reproduction
        let replay_state = replay.get_current_state();
        assert_eq!(replay_state.board, original_board);
        assert_eq!(replay_state.total_pot, original_total_pot);
        assert_eq!(replay_state.stacks, original_stacks);

        // Compare hands card by card
        for (i, (sim_hand, replay_hand)) in original_hands
            .iter()
            .zip(replay_state.hands.iter())
            .enumerate()
        {
            let sim_cards: Vec<_> = sim_hand.iter().collect();
            let replay_cards: Vec<_> = replay_hand.iter().collect();
            assert_eq!(
                sim_cards, replay_cards,
                "Player {} hands must match exactly",
                i
            );
        }
    }

    #[test]
    fn test_step_by_step_consistency() {
        let initial_state = GameState::new_starting(vec![100.0, 100.0], 10.0, 5.0, 0.0, 0);

        let actions = vec![
            Action::GameStart(GameStartPayload {
                ante: 0.0,
                small_blind: 5.0,
                big_blind: 10.0,
            }),
            Action::DealStartingHand(DealStartingHandPayload {
                card: Card::new(Value::Ace, Suit::Spade),
                idx: 0,
            }),
            Action::DealStartingHand(DealStartingHandPayload {
                card: Card::new(Value::King, Suit::Heart),
                idx: 1,
            }),
            Action::DealCommunity(Card::new(Value::Queen, Suit::Diamond)),
            Action::DealCommunity(Card::new(Value::Jack, Suit::Club)),
        ];

        let mut replay = GameReplay::new(initial_state, actions.clone());

        // Step through and verify state consistency at each step
        for i in 1..=actions.len() {
            replay.step_to(i).unwrap();
            assert_eq!(replay.get_current_action_index(), i);

            let current_state = replay.get_current_state();
            assert_eq!(current_state.stacks.len(), 2);
            assert!(current_state.total_pot >= 0.0);
            assert_eq!(current_state.num_players, 2);
        }
    }

    #[test]
    fn test_predertermined_deck() {
        let actions = vec![
            Action::DealStartingHand(DealStartingHandPayload {
                card: Card::new(Value::Ace, Suit::Spade),
                idx: 0,
            }),
            Action::DealStartingHand(DealStartingHandPayload {
                card: Card::new(Value::King, Suit::Heart),
                idx: 1,
            }),
            Action::DealCommunity(Card::new(Value::Queen, Suit::Diamond)),
        ];

        let deck = PredetreminedDeck::from_actions(&actions);

        // Should have extracted 3 cards from deal actions
        assert_eq!(deck.cards_to_deal.len(), 3);
        assert!(
            deck.cards_to_deal
                .contains(&Card::new(Value::Ace, Suit::Spade))
        );
        assert!(
            deck.cards_to_deal
                .contains(&Card::new(Value::King, Suit::Heart))
        );
        assert!(
            deck.cards_to_deal
                .contains(&Card::new(Value::Queen, Suit::Diamond))
        );
    }

    #[test]
    fn test_replay_simulation_builder() {
        let initial_state = GameState::new_starting(vec![100.0, 100.0], 10.0, 5.0, 0.0, 0);

        let actions = vec![
            Action::GameStart(GameStartPayload {
                ante: 0.0,
                small_blind: 5.0,
                big_blind: 10.0,
            }),
            Action::DealStartingHand(DealStartingHandPayload {
                card: Card::new(Value::Ace, Suit::Spade),
                idx: 0,
            }),
        ];

        // Test building replay
        let replay = ReplaySimulationBuilder::new()
            .with_initial_state(initial_state.clone())
            .with_actions(actions.clone())
            .build_replay()
            .unwrap();

        assert_eq!(replay.get_actions().len(), actions.len());
        assert_eq!(replay.get_current_state().stacks, initial_state.stacks);

        // Test building predetermined deck from actions
        let deck = PredetreminedDeck::from_actions(&actions);
        assert!(deck.cards_to_deal.len() > 0);
    }

    #[test]
    fn test_from_actions_constructor() {
        let actions = vec![
            Action::GameStart(GameStartPayload {
                ante: 0.0,
                small_blind: 5.0,
                big_blind: 10.0,
            }),
            Action::PlayerSit(crate::arena::action::PlayerSitPayload {
                idx: 0,
                player_stack: 100.0,
            }),
            Action::PlayerSit(crate::arena::action::PlayerSitPayload {
                idx: 1,
                player_stack: 100.0,
            }),
        ];

        let replay = GameReplay::from_actions(actions.clone());
        assert!(replay.is_ok());

        let replay = replay.unwrap();
        assert_eq!(replay.get_actions().len(), actions.len());
        assert_eq!(replay.get_current_state().stacks, vec![100.0, 100.0]);
    }

    #[test]
    fn test_from_actions_missing_game_start() {
        let actions = vec![Action::DealStartingHand(DealStartingHandPayload {
            card: Card::new(Value::Ace, Suit::Spade),
            idx: 0,
        })];

        let replay = GameReplay::from_actions(actions);
        assert!(replay.is_err());

        if let Err(ReplayError::MissingInitialGameState) = replay {
            // Expected error type
        } else {
            panic!("Expected MissingInitialGameState error");
        }
    }
}

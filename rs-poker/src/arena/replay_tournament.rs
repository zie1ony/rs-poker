use super::replay_game::{GameReplay, ReplayError};
use crate::arena::{
    GameState,
    action::Action,
    competition::TournamentResults,
    historian::{Historian, HistorianError},
};

/// Tournament replay data containing actions from all hands played
#[derive(Debug, Clone)]
pub struct TournamentReplayData {
    /// Actions from each hand in the tournament, indexed by hand number
    pub hands: Vec<Vec<Action>>,
    /// Initial tournament state
    pub initial_state: GameState,
    /// Final results of the tournament
    pub results: Option<TournamentResults>,
}

impl TournamentReplayData {
    pub fn new(initial_state: GameState) -> Self {
        Self {
            hands: Vec::new(),
            initial_state,
            results: None,
        }
    }

    pub fn add_hand(&mut self, actions: Vec<Action>) {
        self.hands.push(actions);
    }

    pub fn set_results(&mut self, results: TournamentResults) {
        self.results = Some(results);
    }

    pub fn num_hands(&self) -> usize {
        self.hands.len()
    }

    pub fn get_hand_actions(&self, hand_index: usize) -> Option<&Vec<Action>> {
        self.hands.get(hand_index)
    }
}

/// Replay system for tournaments that can step through each hand
pub struct TournamentReplay {
    replay_data: TournamentReplayData,
    current_hand_index: usize,
    current_tournament_state: GameState,
}

impl TournamentReplay {
    pub fn new(replay_data: TournamentReplayData) -> Self {
        Self {
            current_tournament_state: replay_data.initial_state.clone(),
            replay_data,
            current_hand_index: 0,
        }
    }

    /// Get the current tournament state (stacks, dealer position, etc.)
    pub fn get_current_tournament_state(&self) -> &GameState {
        &self.current_tournament_state
    }

    /// Get the replay data for the entire tournament
    pub fn get_replay_data(&self) -> &TournamentReplayData {
        &self.replay_data
    }

    /// Get the current hand index
    pub fn get_current_hand_index(&self) -> usize {
        self.current_hand_index
    }

    /// Check if there are more hands to replay
    pub fn has_more_hands(&self) -> bool {
        self.current_hand_index < self.replay_data.hands.len()
    }

    /// Replay the next hand and return the game replay for that hand
    pub fn step_to_next_hand(&mut self) -> Result<Option<GameReplay>, ReplayError> {
        if !self.has_more_hands() {
            return Ok(None);
        }

        let hand_actions = &self.replay_data.hands[self.current_hand_index];
        let game_replay =
            GameReplay::new(self.current_tournament_state.clone(), hand_actions.clone());

        // Update tournament state based on the hand result
        // We need to fast-forward the game replay to get the final state
        let mut temp_replay = game_replay.clone();
        while temp_replay.has_more_actions() {
            temp_replay.step_forward()?;
        }

        // Update our tournament state with the results
        self.update_tournament_state_from_hand(&temp_replay);

        self.current_hand_index += 1;
        Ok(Some(game_replay))
    }

    /// Jump to a specific hand in the tournament
    pub fn step_to_hand(&mut self, target_hand_index: usize) -> Result<(), ReplayError> {
        if target_hand_index > self.replay_data.hands.len() {
            return Err(ReplayError::ActionIndexOutOfBounds);
        }

        // If going backwards, reset to beginning
        if target_hand_index < self.current_hand_index {
            self.reset_to_start();
        }

        // Replay hands until we reach the target
        while self.current_hand_index < target_hand_index {
            self.step_to_next_hand()?;
        }

        Ok(())
    }

    /// Reset to the beginning of the tournament
    pub fn reset_to_start(&mut self) {
        self.current_tournament_state = self.replay_data.initial_state.clone();
        self.current_hand_index = 0;
    }

    /// Get a replay for a specific hand without changing the current state
    pub fn get_hand_replay(&self, hand_index: usize) -> Result<GameReplay, ReplayError> {
        let hand_actions = self
            .replay_data
            .hands
            .get(hand_index)
            .ok_or(ReplayError::ActionIndexOutOfBounds)?;

        // We need to determine the correct game state for this hand
        let mut temp_tournament_state = self.replay_data.initial_state.clone();

        // Replay all hands up to this point to get the correct state
        for i in 0..hand_index {
            let actions = &self.replay_data.hands[i];
            let mut hand_replay = GameReplay::new(temp_tournament_state.clone(), actions.clone());

            // Fast-forward to end of hand
            while hand_replay.has_more_actions() {
                hand_replay.step_forward()?;
            }

            // Update tournament state
            temp_tournament_state =
                self.calculate_updated_tournament_state(&temp_tournament_state, &hand_replay);
        }

        Ok(GameReplay::new(temp_tournament_state, hand_actions.clone()))
    }

    fn update_tournament_state_from_hand(&mut self, hand_replay: &GameReplay) {
        self.current_tournament_state =
            self.calculate_updated_tournament_state(&self.current_tournament_state, hand_replay);
    }

    fn calculate_updated_tournament_state(
        &self,
        current_state: &GameState,
        hand_replay: &GameReplay,
    ) -> GameState {
        let final_hand_state = hand_replay.get_current_state();

        // Update stacks from the hand result
        let new_stacks = final_hand_state.stacks.clone();

        // Find next dealer (next player with chips)
        let mut dealer_idx = (final_hand_state.dealer_idx + 1) % new_stacks.len();
        while new_stacks[dealer_idx] == 0.0 && new_stacks.iter().any(|&s| s > 0.0) {
            dealer_idx = (dealer_idx + 1) % new_stacks.len();
        }

        GameState::new_starting(
            new_stacks,
            current_state.big_blind,
            current_state.small_blind,
            current_state.ante,
            dealer_idx,
        )
    }
}

/// A historian that records actions for tournament replay
pub struct TournamentHistorian {
    tournament_data: TournamentReplayData,
    current_hand_actions: Vec<Action>,
}

impl TournamentHistorian {
    pub fn new(initial_state: GameState) -> Self {
        Self {
            tournament_data: TournamentReplayData::new(initial_state),
            current_hand_actions: Vec::new(),
        }
    }

    /// Call this when a hand ends to save the actions and prepare for the next
    /// hand
    pub fn finish_hand(&mut self) {
        if !self.current_hand_actions.is_empty() {
            self.tournament_data
                .add_hand(self.current_hand_actions.drain(..).collect());
        }
    }

    /// Call this when the tournament ends to save the final results
    pub fn finish_tournament(&mut self, results: TournamentResults) {
        self.finish_hand(); // Finish any remaining hand
        self.tournament_data.set_results(results);
    }

    /// Get the recorded tournament data
    pub fn get_tournament_data(self) -> TournamentReplayData {
        self.tournament_data
    }

    /// Get a reference to the tournament data (without consuming)
    pub fn get_tournament_data_ref(&self) -> &TournamentReplayData {
        &self.tournament_data
    }
}

impl Historian for TournamentHistorian {
    fn record_action(
        &mut self,
        _id: u128,
        _game_state: &GameState,
        action: Action,
    ) -> Result<(), HistorianError> {
        self.current_hand_actions.push(action);
        Ok(())
    }
}

/// Builder for creating replayed tournaments
pub struct ReplayTournamentBuilder {
    tournament_data: Option<TournamentReplayData>,
}

impl ReplayTournamentBuilder {
    pub fn new() -> Self {
        Self {
            tournament_data: None,
        }
    }

    pub fn with_tournament_data(mut self, data: TournamentReplayData) -> Self {
        self.tournament_data = Some(data);
        self
    }

    /// Build a tournament replay that can step through each hand
    pub fn build_tournament_replay(self) -> Result<TournamentReplay, ReplayError> {
        let data = self
            .tournament_data
            .ok_or(ReplayError::MissingInitialGameState)?;
        Ok(TournamentReplay::new(data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arena::{
        action::{Action, DealStartingHandPayload, GameStartPayload},
        agent::{AgentGenerator, AllInAgentGenerator, FoldingAgentGenerator},
        competition::SingleTableTournamentBuilder,
    };
    use crate::core::{Card, Suit, Value};

    #[test]
    fn test_tournament_replay_data_creation() {
        let stacks = vec![100.0, 100.0, 100.0, 100.0];
        let initial_state = GameState::new_starting(stacks.clone(), 10.0, 5.0, 1.0, 0);

        let mut tournament_data = TournamentReplayData::new(initial_state.clone());

        // Test initial state
        assert_eq!(tournament_data.num_hands(), 0);
        assert_eq!(tournament_data.initial_state.stacks, stacks);
        assert!(tournament_data.results.is_none());

        // Add hands
        let hand_1_actions = vec![Action::GameStart(GameStartPayload {
            ante: 1.0,
            small_blind: 5.0,
            big_blind: 10.0,
        })];
        let hand_2_actions = vec![Action::DealStartingHand(DealStartingHandPayload {
            card: Card::new(Value::Ace, Suit::Spade),
            idx: 0,
        })];

        tournament_data.add_hand(hand_1_actions.clone());
        tournament_data.add_hand(hand_2_actions.clone());

        assert_eq!(tournament_data.num_hands(), 2);
        assert_eq!(tournament_data.get_hand_actions(0), Some(&hand_1_actions));
        assert_eq!(tournament_data.get_hand_actions(1), Some(&hand_2_actions));
        assert_eq!(tournament_data.get_hand_actions(2), None);
    }

    #[test]
    fn test_tournament_replay_creation() {
        let stacks = vec![100.0, 100.0, 100.0, 100.0];
        let initial_state = GameState::new_starting(stacks.clone(), 10.0, 5.0, 1.0, 0);

        let mut tournament_data = TournamentReplayData::new(initial_state.clone());

        // Add mock hands
        tournament_data.add_hand(vec![]);
        tournament_data.add_hand(vec![]);
        tournament_data.add_hand(vec![]);

        let tournament_replay = TournamentReplay::new(tournament_data.clone());

        // Test initial state
        assert_eq!(tournament_replay.get_current_hand_index(), 0);
        assert!(tournament_replay.has_more_hands());
        assert_eq!(tournament_replay.get_replay_data().num_hands(), 3);
        assert_eq!(
            tournament_replay.get_current_tournament_state().stacks,
            stacks
        );
        assert_eq!(
            tournament_replay.get_current_tournament_state().big_blind,
            10.0
        );
        assert_eq!(
            tournament_replay.get_current_tournament_state().small_blind,
            5.0
        );
        assert_eq!(tournament_replay.get_current_tournament_state().ante, 1.0);
    }

    #[test]
    fn test_tournament_replay_navigation() {
        let initial_state = GameState::new_starting(vec![100.0, 100.0], 10.0, 5.0, 0.0, 0);
        let mut tournament_data = TournamentReplayData::new(initial_state);

        // Add 3 hands
        tournament_data.add_hand(vec![]);
        tournament_data.add_hand(vec![]);
        tournament_data.add_hand(vec![]);

        let mut tournament_replay = TournamentReplay::new(tournament_data);

        // Test initial state
        assert_eq!(tournament_replay.get_current_hand_index(), 0);
        assert!(tournament_replay.has_more_hands());

        // Step to next hand
        let step_result = tournament_replay.step_to_next_hand();
        assert!(step_result.is_ok());
        assert_eq!(tournament_replay.get_current_hand_index(), 1);

        // Jump to specific hand
        let jump_result = tournament_replay.step_to_hand(2);
        assert!(jump_result.is_ok());
        assert_eq!(tournament_replay.get_current_hand_index(), 2);

        // Test at end
        let end_step = tournament_replay.step_to_next_hand();
        assert!(end_step.is_ok());
        assert!(!tournament_replay.has_more_hands());

        // Reset to start
        tournament_replay.reset_to_start();
        assert_eq!(tournament_replay.get_current_hand_index(), 0);
        assert!(tournament_replay.has_more_hands());
    }

    #[test]
    fn test_tournament_replay_boundary_conditions() {
        let initial_state = GameState::new_starting(vec![100.0, 100.0], 10.0, 5.0, 0.0, 0);
        let mut tournament_data = TournamentReplayData::new(initial_state);

        tournament_data.add_hand(vec![]);
        tournament_data.add_hand(vec![]);

        let mut tournament_replay = TournamentReplay::new(tournament_data);

        // Test jumping beyond available hands
        let invalid_jump = tournament_replay.step_to_hand(10);
        assert!(invalid_jump.is_err());

        // Test getting non-existent hand
        let invalid_hand = tournament_replay.get_hand_replay(10);
        assert!(invalid_hand.is_err());

        // Step through all hands to test end condition
        let mut hands_stepped = 0;
        while tournament_replay.has_more_hands() {
            let step_result = tournament_replay.step_to_next_hand();
            assert!(step_result.is_ok());
            hands_stepped += 1;
            assert!(hands_stepped <= 2); // Should not step through more than 2 hands
        }
        assert_eq!(hands_stepped, 2);
        assert!(!tournament_replay.has_more_hands());
    }

    #[test]
    fn test_tournament_replay_hand_access() {
        let initial_state = GameState::new_starting(vec![100.0, 100.0], 10.0, 5.0, 0.0, 0);
        let mut tournament_data = TournamentReplayData::new(initial_state);

        let hand_0_actions = vec![Action::GameStart(GameStartPayload {
            ante: 0.0,
            small_blind: 5.0,
            big_blind: 10.0,
        })];
        tournament_data.add_hand(hand_0_actions.clone());
        tournament_data.add_hand(vec![]);

        let mut tournament_replay = TournamentReplay::new(tournament_data);

        // Get specific hand without changing state
        let hand_0_replay = tournament_replay.get_hand_replay(0);
        assert!(hand_0_replay.is_ok());
        assert_eq!(tournament_replay.get_current_hand_index(), 0); // Should not change

        let hand_replay = hand_0_replay.unwrap();
        assert_eq!(hand_replay.get_actions(), &hand_0_actions);

        // Move to hand 1, then get hand 0 again
        tournament_replay.step_to_hand(1).unwrap();
        assert_eq!(tournament_replay.get_current_hand_index(), 1);

        let hand_0_again = tournament_replay.get_hand_replay(0);
        assert!(hand_0_again.is_ok());
        assert_eq!(tournament_replay.get_current_hand_index(), 1); // Should still be at hand 1
    }

    #[test]
    fn test_tournament_historian() {
        let initial_state = GameState::new_starting(vec![100.0, 100.0], 10.0, 5.0, 0.0, 0);
        let mut historian = TournamentHistorian::new(initial_state.clone());

        // Record actions for first hand
        let action1 = Action::GameStart(GameStartPayload {
            ante: 0.0,
            small_blind: 5.0,
            big_blind: 10.0,
        });
        let action2 = Action::DealStartingHand(DealStartingHandPayload {
            card: Card::new(Value::Ace, Suit::Spade),
            idx: 0,
        });

        historian
            .record_action(1, &initial_state, action1.clone())
            .unwrap();
        historian
            .record_action(2, &initial_state, action2.clone())
            .unwrap();

        // Finish hand
        historian.finish_hand();

        // Record second hand
        let action3 = Action::GameStart(GameStartPayload {
            ante: 0.0,
            small_blind: 5.0,
            big_blind: 10.0,
        });
        historian
            .record_action(3, &initial_state, action3.clone())
            .unwrap();
        historian.finish_hand();

        // Check recorded data
        let tournament_data = historian.get_tournament_data_ref();
        assert_eq!(tournament_data.num_hands(), 2);

        let hand_0_actions = tournament_data.get_hand_actions(0).unwrap();
        assert_eq!(hand_0_actions.len(), 2);
        assert_eq!(hand_0_actions[0], action1);
        assert_eq!(hand_0_actions[1], action2);

        let hand_1_actions = tournament_data.get_hand_actions(1).unwrap();
        assert_eq!(hand_1_actions.len(), 1);
        assert_eq!(hand_1_actions[0], action3);
    }

    #[test]
    fn test_replay_tournament_builder() {
        let initial_state = GameState::new_starting(vec![100.0, 100.0], 10.0, 5.0, 0.0, 0);
        let mut tournament_data = TournamentReplayData::new(initial_state);

        tournament_data.add_hand(vec![]);
        tournament_data.add_hand(vec![]);

        // Test building tournament replay
        let tournament_replay = ReplayTournamentBuilder::new()
            .with_tournament_data(tournament_data)
            .build_tournament_replay()
            .unwrap();

        assert_eq!(tournament_replay.get_replay_data().num_hands(), 2);
        assert_eq!(tournament_replay.get_current_hand_index(), 0);

        // Test building without data should fail
        let builder_without_data = ReplayTournamentBuilder::new();
        let result = builder_without_data.build_tournament_replay();
        assert!(result.is_err());
    }

    #[test]
    fn test_tournament_with_results() {
        let stacks = vec![100.0, 100.0, 100.0, 100.0];
        let initial_state = GameState::new_starting(stacks.clone(), 10.0, 5.0, 1.0, 0);

        // Create a tournament to get real results
        let agent_generators: Vec<Box<dyn AgentGenerator>> = vec![
            Box::new(AllInAgentGenerator::default()),
            Box::new(FoldingAgentGenerator::default()),
            Box::new(FoldingAgentGenerator::default()),
            Box::new(FoldingAgentGenerator::default()),
        ];

        let tournament = SingleTableTournamentBuilder::default()
            .agent_generators(agent_generators)
            .starting_game_state(initial_state.clone())
            .build()
            .unwrap();

        let results = tournament.run().unwrap();

        // Create tournament replay data with results
        let mut tournament_data = TournamentReplayData::new(initial_state);
        tournament_data.add_hand(vec![]);
        tournament_data.add_hand(vec![]);
        tournament_data.set_results(results.clone());

        let tournament_replay = TournamentReplay::new(tournament_data);

        // Test results preservation
        let replay_results = tournament_replay.get_replay_data().results.as_ref();
        assert!(replay_results.is_some());

        let replay_results = replay_results.unwrap();
        assert_eq!(replay_results.places(), results.places());
        assert_eq!(replay_results.rounds(), results.rounds());
        assert_eq!(replay_results.places().len(), 4);
        assert!(replay_results.rounds() > 0);
    }

    #[test]
    fn test_step_through_all_hands() {
        let initial_state = GameState::new_starting(vec![100.0, 100.0], 10.0, 5.0, 0.0, 0);
        let mut tournament_data = TournamentReplayData::new(initial_state.clone());

        // Add hands with actions
        tournament_data.add_hand(vec![Action::GameStart(GameStartPayload {
            ante: 0.0,
            small_blind: 5.0,
            big_blind: 10.0,
        })]);
        tournament_data.add_hand(vec![Action::DealStartingHand(DealStartingHandPayload {
            card: Card::new(Value::King, Suit::Heart),
            idx: 0,
        })]);
        tournament_data.add_hand(vec![Action::DealStartingHand(DealStartingHandPayload {
            card: Card::new(Value::Queen, Suit::Diamond),
            idx: 1,
        })]);

        let mut tournament_replay = TournamentReplay::new(tournament_data);

        // Step through all hands and verify each one
        let mut hands_processed = 0;
        while tournament_replay.has_more_hands() {
            let hand_replay_result = tournament_replay.step_to_next_hand();
            assert!(hand_replay_result.is_ok());

            let hand_replay = hand_replay_result.unwrap();
            assert!(hand_replay.is_some());

            let hand_replay = hand_replay.unwrap();
            assert!(hand_replay.get_actions().len() > 0);

            hands_processed += 1;
        }

        assert_eq!(hands_processed, 3);
        assert_eq!(tournament_replay.get_current_hand_index(), 3);

        // Verify we can still get individual hands after stepping through all
        let hand_0 = tournament_replay.get_hand_replay(0);
        assert!(hand_0.is_ok());

        let hand_1 = tournament_replay.get_hand_replay(1);
        assert!(hand_1.is_ok());

        let hand_2 = tournament_replay.get_hand_replay(2);
        assert!(hand_2.is_ok());
    }

    #[test]
    fn test_tournament_state_consistency() {
        let stacks = vec![100.0, 200.0, 150.0];
        let initial_state = GameState::new_starting(stacks.clone(), 20.0, 10.0, 2.0, 1);
        let mut tournament_data = TournamentReplayData::new(initial_state.clone());

        tournament_data.add_hand(vec![]);
        tournament_data.add_hand(vec![]);

        let mut tournament_replay = TournamentReplay::new(tournament_data);

        // Verify initial state consistency
        let current_state = tournament_replay.get_current_tournament_state();
        assert_eq!(current_state.stacks, stacks);
        assert_eq!(current_state.big_blind, 20.0);
        assert_eq!(current_state.small_blind, 10.0);
        assert_eq!(current_state.ante, 2.0);
        assert_eq!(current_state.dealer_idx, 1);

        // Step through and verify state is maintained
        tournament_replay.step_to_next_hand().unwrap();
        let state_after_step = tournament_replay.get_current_tournament_state();
        assert_eq!(state_after_step.stacks, stacks);
        assert_eq!(state_after_step.big_blind, 20.0);

        // Reset and verify state is restored
        tournament_replay.reset_to_start();
        let state_after_reset = tournament_replay.get_current_tournament_state();
        assert_eq!(state_after_reset.stacks, stacks);
        assert_eq!(state_after_reset.big_blind, 20.0);
        assert_eq!(state_after_reset.small_blind, 10.0);
        assert_eq!(state_after_reset.ante, 2.0);
        assert_eq!(state_after_reset.dealer_idx, 1);
    }
}

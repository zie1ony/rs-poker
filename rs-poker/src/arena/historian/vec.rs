use std::{cell::RefCell, rc::Rc};

use crate::arena::{GameState, action::Action};

use super::{Historian, HistorianError};

#[derive(Debug, Clone)]
pub struct HistoryRecord {
    pub before_game_state: Option<GameState>,
    pub action: Action,
    pub after_game_state: GameState,
}

/// VecHistorian is a historian that will
/// append each action to a vector.
pub struct VecHistorian {
    previous: Option<GameState>,
    records: Rc<RefCell<Vec<HistoryRecord>>>,
}

impl VecHistorian {
    /// Create a new storage for the historian
    /// that can be introspected later.
    pub fn get_storage(&self) -> Rc<RefCell<Vec<HistoryRecord>>> {
        self.records.clone()
    }

    /// Create a new VecHistorian with the provided storage
    /// `Rc<RefCell<Vec<HistoryRecord>>>`
    pub fn new_with_actions(actions: Rc<RefCell<Vec<HistoryRecord>>>) -> Self {
        Self {
            records: actions,
            previous: None,
        }
    }

    pub fn new() -> Self {
        VecHistorian::new_with_actions(Rc::new(RefCell::new(vec![])))
    }
}

impl Default for VecHistorian {
    fn default() -> Self {
        VecHistorian::new()
    }
}

impl Historian for VecHistorian {
    fn record_action(
        &mut self,
        _id: u128,
        game_state: &GameState,
        action: Action,
    ) -> Result<(), HistorianError> {
        let mut act = self.records.try_borrow_mut()?;

        // Now that we have the lock, we can record the action
        act.push(HistoryRecord {
            before_game_state: self.previous.clone(),
            action,
            after_game_state: game_state.clone(),
        });

        // Record the game state for the next action
        self.previous = Some(game_state.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::arena::{
        Agent, HoldemSimulationBuilder,
        agent::{CallingAgent, RandomAgent},
    };

    use super::*;

    #[test]
    fn test_vec_historian() {
        let hist = Box::new(VecHistorian::default());
        let records = hist.get_storage();
        let mut rng = rand::rng();

        let stacks = vec![100.0; 5];
        let agents: Vec<Box<dyn Agent>> = vec![
            Box::<RandomAgent>::default(),
            Box::<RandomAgent>::default(),
            Box::<RandomAgent>::default(),
            Box::<RandomAgent>::default(),
            Box::<RandomAgent>::default(),
        ];
        let game_state = GameState::new_starting(stacks, 10.0, 5.0, 0.0, 0);

        let mut sim = HoldemSimulationBuilder::default()
            .game_state(game_state)
            .agents(agents)
            .historians(vec![hist])
            .build()
            .unwrap();

        sim.run(&mut rng);

        assert!(records.borrow().len() > 10);
    }

    #[test]
    fn test_restarting_simulations() {
        // The first records.
        let hist = Box::new(VecHistorian::default());
        let records = hist.get_storage();
        let mut rng = rand::rng();

        let stacks = vec![100.0; 2];
        let agents: Vec<Box<dyn Agent>> = vec![
            Box::<CallingAgent>::default(),
            Box::<CallingAgent>::default(),
        ];

        let game_state = GameState::new_starting(stacks, 10.0, 5.0, 0.0, 0);

        let mut sim = HoldemSimulationBuilder::default()
            .game_state(game_state)
            .agents(agents)
            .historians(vec![hist])
            .build()
            .unwrap();

        sim.run(&mut rng);

        // Now we have a set of records of what happenend in the first simulation.
        // It doesn't matter what actions the agents took, we just need to know that
        // the simulation always restarts at asking the same player to play

        for r in records.borrow().iter() {
            if let (Action::PlayedAction(played_action), Some(before_game_state)) =
                (&r.action, &r.before_game_state)
            {
                let inner_agents: Vec<Box<dyn Agent>> = vec![
                    Box::<CallingAgent>::default(),
                    Box::<CallingAgent>::default(),
                ];

                let inner_hist = Box::new(VecHistorian::default());
                let inner_records = inner_hist.get_storage();

                // We can now restart the simulation and see if the same player takes the next
                // turn
                let mut inner_sim = HoldemSimulationBuilder::default()
                    .game_state(before_game_state.clone())
                    .agents(inner_agents)
                    .historians(vec![inner_hist])
                    .build()
                    .unwrap();

                inner_sim.run(&mut rng);

                let first_record = inner_records.borrow().first().unwrap().clone();

                if let Action::PlayedAction(inner_played_action) = first_record.action {
                    assert_eq!(played_action.idx, inner_played_action.idx);
                } else {
                    panic!(
                        "The first action should be a played action, found {:?}",
                        first_record.action
                    );
                }
            }
        }
    }
}

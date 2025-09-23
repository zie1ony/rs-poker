use super::Historian;

pub struct NullHistorian;

impl Historian for NullHistorian {
    fn record_action(
        &mut self,
        _id: u128,
        _game_state: &crate::arena::GameState,
        _action: crate::arena::action::Action,
    ) -> Result<(), super::HistorianError> {
        Ok(())
    }
}

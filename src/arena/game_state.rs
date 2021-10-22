use fixedbitset::FixedBitSet;

use crate::core::{Card, FlatDeck, Hand};
pub enum Round {
    Starting,
    Preflop,
    Flop,
    Turn,
    River,
}

pub struct RoundData {
    round_bet: Vec<usize>,
    betting_complete: FixedBitSet,
    min_bet: usize,
}

pub struct GameState {
    pub initial_num_players: usize,
    pub stacks: Vec<usize>,
    pub hands: Vec<Hand>,
    pub player_active: FixedBitSet,
    pub dealer_idx: usize,
    pub round: Round,
    pub board: Vec<Card>,
}

impl GameState {
    pub fn new(stacks: Vec<usize>, _big_blind: usize, _small_blind: usize) -> Self {
        let num = stacks.len();
        let mut active_mask = FixedBitSet::with_capacity(num);
        active_mask.set_range(.., true);
        GameState {
            initial_num_players: num,
            stacks,
            player_active: active_mask,
            dealer_idx: 0,
            hands: vec![Hand::default(); num],
            round: Round::Starting,
            board: vec![],
        }
    }

    pub fn deal_preflop(&mut self, deck: &mut FlatDeck) {
        for h in &mut self.hands {
            h.push(deck.deal().unwrap());
            h.push(deck.deal().unwrap());
        }
        self.round = Round::Preflop;
    }
}

use fixedbitset::FixedBitSet;

use crate::core::{Card, Hand};
pub enum Round {
    Starting,
    Preflop,
    Flop,
    Turn,
    River,
    Showdown,
}

pub struct RoundData {
    pub player_active: FixedBitSet,
    // The minimum allowed bet.
    pub min_bet: usize,
    // The value to be called.
    pub bet: usize,
    // How much each player has put in so far.
    pub player_bet: Vec<usize>,
    // The number of times the player has put in money.
    pub bet_count: Vec<usize>,
    // The index of the next player to act.
    pub to_act_idx: usize,
}

impl RoundData {
    pub fn advance(&mut self) {
        loop {
            self.to_act_idx = (self.to_act_idx + 1) % self.player_active.len();
            if self.player_active.count_ones(..) == 0
                || self.player_active.contains(self.to_act_idx)
            {
                break;
            }
        }
    }
}

pub struct GameState {
    pub num_players: usize,
    pub player_active: FixedBitSet,
    pub stacks: Vec<usize>,
    pub big_blind: usize,
    pub small_blind: usize,
    pub hands: Vec<Hand>,
    pub dealer_idx: usize,
    pub last_raise: usize,
    pub round: Round,
    pub round_data: Vec<RoundData>,
    pub board: Vec<Card>,
}

impl GameState {
    pub fn new(stacks: Vec<usize>, big_blind: usize, small_blind: usize) -> Self {
        let num_players = stacks.len();

        // Everyone's active right now
        let mut active_mask = FixedBitSet::with_capacity(num_players);
        active_mask.set_range(.., true);
        GameState {
            num_players,
            stacks,
            big_blind,
            small_blind,
            player_active: active_mask,
            dealer_idx: 0,
            last_raise: 0,
            hands: vec![Hand::default(); num_players],
            round: Round::Starting,
            board: vec![],
            round_data: vec![],
        }
    }

    pub fn advance_round(&mut self) {
        match self.round {
            Round::Starting => self.advance_preflop(),
            Round::Preflop => self.advance_flop(),
            _ => todo!(),
        }
    }

    fn advance_preflop(&mut self) {
        self.round = Round::Preflop;
        self.round_data.push(self.round_data());
        self.do_bet(self.small_blind, true);
        self.do_bet(self.big_blind, true);
    }

    fn advance_flop(&mut self) {
        self.round = Round::Flop;
        self.round_data.push(self.round_data())
    }

    fn round_data(&self) -> RoundData {
        RoundData {
            player_bet: vec![0; self.num_players],
            bet_count: vec![0; self.num_players],
            bet: 0,
            min_bet: self.big_blind,
            player_active: self.player_active.clone(),
            to_act_idx: self.left_of_dealer(),
        }
    }

    pub fn do_bet(&mut self, ammount: usize, is_forced: bool) -> Option<usize> {
        if let Some(round_idx) = self.round_index() {
            // Grab the current round data.
            let rd = &mut self.round_data[round_idx];

            // Which player is next to act
            let idx = rd.to_act_idx;
            // Can't bet what's not there
            let bet_ammount = ammount.min(self.stacks[idx]);
            // Take the money out.
            self.stacks[idx] -= bet_ammount;
            // If they put money into the pot then they are done this turn.
            rd.player_active.set(idx, is_forced);

            rd.player_bet[idx] += bet_ammount;
            rd.bet_count[idx] += 1;

            rd.bet = rd.bet.max(bet_ammount);

            // We're out and can't continue.
            if self.stacks[idx] == 0 {
                self.player_active.set(idx, false);
                rd.player_active.set(idx, false);
            }

            // Advance the next to act.
            rd.advance();

            Some(bet_ammount)
        } else {
            None
        }
    }

    pub fn left_of_dealer(&self) -> usize {
        (self.dealer_idx + 1) % self.num_players
    }

    fn round_index(&self) -> Option<usize> {
        match self.round {
            Round::Preflop => Some(0),
            Round::Flop => Some(1),
            Round::Turn => Some(2),
            Round::River => Some(3),
            _ => None,
        }
    }
}

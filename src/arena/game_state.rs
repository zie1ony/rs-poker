use fixedbitset::FixedBitSet;

use crate::core::{Card, Hand};

use super::errors::GameStateError;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Round {
    Starting,
    Preflop,
    Flop,
    Turn,
    River,
    Showdown,
}

impl Round {
    pub fn advance(&self) -> Result<Self, GameStateError> {
        match *self {
            Round::Starting => Ok(Round::Preflop),
            Round::Preflop => Ok(Round::Flop),
            Round::Flop => Ok(Round::Turn),
            Round::Turn => Ok(Round::River),
            Round::River => Ok(Round::Showdown),
            _ => Err(GameStateError::CantAdvanceRound),
        }
    }
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

    pub fn do_bet(&mut self, bet_ammount: usize) {
        self.player_bet[self.to_act_idx] += bet_ammount;
        self.bet_count[self.to_act_idx] += 1;

        // The amount to be called is
        // the maximum anyone has wagered.
        self.bet = self.bet.max(self.player_bet[self.to_act_idx]);

        // Keep the maximum bet ammount. Anything
        // smaller should be due to going all in.
        self.min_bet = self.min_bet.max(bet_ammount);
    }
}

pub struct GameState {
    /// The number of players that started
    pub num_players: usize,
    /// Which players are still active in the game.
    pub player_active: FixedBitSet,
    /// The total ammount in all pots
    total_pot: usize,
    /// How much is left in each player's stack
    pub stacks: Vec<usize>,
    /// The big blind size
    pub big_blind: usize,
    /// The small blind size
    pub small_blind: usize,
    /// The hands for each player. We keep hands
    /// even if the player is not currently active.
    pub hands: Vec<Hand>,
    /// The index of the player who's the dealer
    pub dealer_idx: usize,
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
            total_pot: 0,
            hands: vec![Hand::default(); num_players],
            round: Round::Starting,
            board: vec![],
            round_data: vec![],
        }
    }

    pub fn num_active_players(&self) -> usize {
        self.player_active.count_ones(..)
    }

    pub fn is_complete(&self) -> bool {
        self.num_active_players() == 1 || self.round == Round::Showdown
    }

    pub fn advance_round(&mut self) -> Result<(), GameStateError> {
        match self.round {
            Round::Starting => self.advance_preflop(),
            _ => self.advance_normal(),
        }
    }

    fn advance_preflop(&mut self) -> Result<(), GameStateError> {
        self.round = Round::Preflop;
        self.round_data.push(self.round_data());
        self.do_bet(self.small_blind, true)?;
        self.do_bet(self.big_blind, true)?;
        Ok(())
    }

    fn advance_normal(&mut self) -> Result<(), GameStateError> {
        self.round = self.round.advance()?;
        self.round_data.push(self.round_data());
        Ok(())
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

    pub fn do_bet(&mut self, ammount: usize, is_forced: bool) -> Result<usize, GameStateError> {
        let round_idx = self.round_index()?;

        // Which player is next to act
        let idx = self.round_data[round_idx].to_act_idx;

        // Make sure the bet is a correct ammount and if not then cap it at the maximum
        let bet_ammount = if is_forced {
            ammount
        } else {
            self.validate_bet_ammount(ammount)?
        };
        // Take the money out.
        self.stacks[idx] -= bet_ammount;
        // Grab the current round data.
        let rd = &mut self.round_data[round_idx];
        // If they put money into the pot then they are done this turn.
        rd.player_active.set(idx, !is_forced);
        rd.do_bet(bet_ammount);

        self.total_pot += bet_ammount;

        // We're out and can't continue.
        if self.stacks[idx] == 0 {
            self.player_active.set(idx, false);
            // It doesn' matter if this is a forced
            // bet if the player is out of money.
            rd.player_active.set(idx, false);
        }

        // Advance the next to act.
        rd.advance();

        Ok(bet_ammount)
    }

    pub fn left_of_dealer(&self) -> usize {
        (self.dealer_idx + 1) % self.num_players
    }

    fn round_index(&self) -> Result<usize, GameStateError> {
        match self.round {
            Round::Preflop => Ok(0),
            Round::Flop => Ok(1),
            Round::Turn => Ok(2),
            Round::River => Ok(3),
            _ => Err(GameStateError::InvalidRoundIndex),
        }
    }

    fn validate_bet_ammount(&self, ammount: usize) -> Result<usize, GameStateError> {
        let round_idx = self.round_index()?;

        // Which player is next to act
        let idx = self.round_data[round_idx].to_act_idx;

        if self.round_data[round_idx].player_bet[idx] > ammount {
            // We've already bet more than this. No takes backs.
            Err(GameStateError::BetSizeDoesntCallSelf)
        } else {
            // How much extra are we putting in.
            let extra = ammount - self.round_data[round_idx].player_bet[idx];

            // How much more are we putting in this time. Capped at the stack
            let capped_extra = self.stacks[idx].min(extra);
            // What our new player bet will be
            let capped_new_player_bet = self.round_data[round_idx].player_bet[idx] + capped_extra;
            let current_bet = self.round_data[round_idx].bet;
            // How much this is a raise.
            let raise = (capped_new_player_bet as i64 - current_bet as i64).max(0) as usize;
            let is_all_in = capped_extra == self.stacks[idx];
            if capped_new_player_bet < self.round_data[round_idx].bet && !is_all_in {
                // If we're not even calling and it's not an all in.
                Err(GameStateError::BetSizeDoesntCall)
            } else if raise > 0 && raise < self.round_data[round_idx].min_bet && !is_all_in {
                // There's a raise the raise is less than the min bet and it's not an all in
                Err(GameStateError::RaiseSizeTooSmall)
            } else {
                // Yeah this looks ok.
                Ok(capped_extra)
            }
        }
    }
}

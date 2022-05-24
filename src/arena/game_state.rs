use core::fmt;

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
    Complete,
}

impl Round {
    pub fn advance(&self) -> Result<Self, GameStateError> {
        match *self {
            Round::Starting => Ok(Round::Preflop),
            Round::Preflop => Ok(Round::Flop),
            Round::Flop => Ok(Round::Turn),
            Round::Turn => Ok(Round::River),
            Round::River => Ok(Round::Showdown),
            Round::Showdown => Ok(Round::Complete),
            _ => Err(GameStateError::CantAdvanceRound),
        }
    }
}

#[derive(Clone)]
pub struct RoundData {
    pub player_active: FixedBitSet,
    // The minimum allowed bet.
    pub min_raise: i32,
    // The value to be called.
    pub bet: i32,
    // How much each player has put in so far.
    pub player_bet: Vec<i32>,
    // The number of times the player has put in money.
    pub bet_count: Vec<u8>,
    // The number of times the player has increased the bet voluntarily.
    pub raise_count: Vec<u8>,
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

    pub fn do_bet(&mut self, extra_ammount: i32, is_forced: bool) {
        self.player_bet[self.to_act_idx] += extra_ammount;
        self.bet_count[self.to_act_idx] += 1;

        // Before resetting check if this is a raise to keep count
        let is_bet = self.player_bet[self.to_act_idx] > self.bet;
        if !is_forced && is_bet {
            self.raise_count[self.to_act_idx] += 1;
        }

        // The amount to be called is
        // the maximum anyone has wagered.
        self.bet = self.bet.max(self.player_bet[self.to_act_idx]);

        // Keep the maximum bet ammount. Anything
        // smaller should be due to going all in.
        self.min_raise = self.min_raise.max(extra_ammount);
    }

    pub fn num_active_players(&self) -> usize {
        self.player_active.count_ones(..)
    }

    pub fn current_player_bet(&self) -> i32 {
        self.player_bet[self.to_act_idx]
    }
}

#[derive(Clone)]
pub struct GameState {
    /// The number of players that started
    pub num_players: usize,
    /// Which players are still active in the game.
    pub player_active: FixedBitSet,
    pub player_all_in: FixedBitSet,
    /// The total ammount in all pots
    pub total_pot: i32,
    /// How much is left in each player's stack
    pub stacks: Vec<i32>,
    pub player_bet: Vec<i32>,
    pub player_winnings: Vec<i32>,
    /// The big blind size
    pub big_blind: i32,
    /// The small blind size
    pub small_blind: i32,
    /// The hands for each player. We keep hands
    /// even if the player is not currently active.
    pub hands: Vec<Hand>,
    /// The index of the player who's the dealer
    pub dealer_idx: usize,
    // What round this is currently
    pub round: Round,
    // ALl the current state of the round.
    pub round_data: Vec<RoundData>,
    // The community cards.
    pub board: Vec<Card>,
}

impl GameState {
    pub fn new(stacks: Vec<i32>, big_blind: i32, small_blind: i32, dealer_idx: usize) -> Self {
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
            player_all_in: FixedBitSet::with_capacity(num_players),
            player_bet: vec![0; num_players],
            player_winnings: vec![0; num_players],
            dealer_idx,
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

    pub fn num_active_players_in_round(&self) -> usize {
        if let Ok(round_idx) = self.round_index() {
            self.round_data[round_idx].num_active_players()
        } else {
            let mut active = self.player_active.clone();
            active.union_with(&self.player_all_in);
            active.count_ones(..)
        }
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
        self.round_data.push(self.new_round_data());
        self.do_bet(self.small_blind, true)?;
        self.do_bet(self.big_blind, true)?;
        Ok(())
    }

    fn advance_normal(&mut self) -> Result<(), GameStateError> {
        self.round = self.round.advance()?;
        self.round_data.push(self.new_round_data());
        Ok(())
    }

    fn new_round_data(&self) -> RoundData {
        let mut rd = RoundData {
            player_bet: vec![0; self.num_players],
            bet_count: vec![0; self.num_players],
            raise_count: vec![0; self.num_players],
            bet: 0,
            min_raise: self.big_blind,
            player_active: self.player_active.clone(),
            to_act_idx: self.dealer_idx,
        };

        rd.advance();

        rd
    }

    pub fn current_round_data(&self) -> Option<&RoundData> {
        if let Ok(round_idx) = self.round_index() {
            Some(&self.round_data[round_idx])
        } else {
            None
        }
    }

    pub fn fold(&mut self) -> Result<(), GameStateError> {
        // round index doesn't change. So no need
        // to do the bounds check more than once.
        let round_idx = self.round_index()?;

        // Which player is next to act
        let idx = self.round_data[round_idx].to_act_idx;

        // We are going to change the current round since this player is out.
        let rd = &mut self.round_data[round_idx];
        rd.player_active.set(idx, false);
        rd.advance();
        self.player_active.set(idx, false);

        Ok(())
    }

    pub fn do_bet(&mut self, ammount: i32, is_forced: bool) -> Result<i32, GameStateError> {
        // round index doesn't change. So no need
        // to do the bounds check more than once.
        let round_idx = self.round_index()?;

        // Which player is next to act
        let idx = self.round_data[round_idx].to_act_idx;

        // Make sure the bet is a correct ammount and if not then cap it at the maximum
        let extra_ammount = if is_forced {
            self.validate_forced_bet_ammount(ammount)?
        } else {
            self.validate_bet_ammount(ammount)?
        };
        // Take the money out.
        self.stacks[idx] -= extra_ammount;
        // Grab the current round data.
        let rd = &mut self.round_data[round_idx];

        let prev_bet = rd.bet;

        rd.do_bet(extra_ammount, is_forced);

        self.player_bet[idx] += extra_ammount;

        self.total_pot += extra_ammount;

        let is_new_bet = prev_bet < rd.bet;

        if is_new_bet {
            // This is a new max bet. We need to reset who can act in the round
            rd.player_active = self.player_active.clone();
        }

        // If they put money into the pot then they are done this turn.
        if !is_forced {
            rd.player_active.set(idx, false);
        }

        // We're out and can't continue
        if self.stacks[idx] == 0 {
            // Keep track of who's still active.
            self.player_active.set(idx, false);
            // Keep track of going all in. We'll use that later on
            // to determine who's worth ranking.
            self.player_all_in.set(idx, true);
            // It doesn' matter if this is a forced
            // bet if the player is out of money.
            rd.player_active.set(idx, false);
        }

        // Advance the next to act.
        rd.advance();

        Ok(extra_ammount)
    }

    pub fn award(&mut self, player_idx: usize, ammount: i32) {
        self.stacks[player_idx] += ammount;
        self.player_winnings[player_idx] += ammount;
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

    fn validate_forced_bet_ammount(&self, ammount: i32) -> Result<i32, GameStateError> {
        let round_idx = self.round_index()?;

        // Which player is next to act
        let idx = self.round_data[round_idx].to_act_idx;

        Ok(self.stacks[idx].min(ammount))
    }

    fn validate_bet_ammount(&self, ammount: i32) -> Result<i32, GameStateError> {
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
            let raise = (capped_new_player_bet - current_bet).max(0);
            let is_all_in = capped_extra == self.stacks[idx];
            let is_raise = raise > 0;
            if capped_new_player_bet < self.round_data[round_idx].bet && !is_all_in {
                // If we're not even calling and it's not an all in.
                Err(GameStateError::BetSizeDoesntCall)
            } else if is_raise && !is_all_in && raise < self.round_data[round_idx].min_raise {
                // There's a raise the raise is less than the min bet and it's not an all in
                Err(GameStateError::RaiseSizeTooSmall)
            } else {
                // Yeah this looks ok.
                Ok(capped_extra)
            }
        }
    }
}

impl fmt::Debug for RoundData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RoundData")
            .field("player_active", &self.player_active)
            .field("num_active_players", &self.num_active_players())
            .field("min_raise", &self.min_raise)
            .field("bet", &self.bet)
            .field("player_bet", &self.player_bet)
            .field("bet_count", &self.bet_count)
            .field("raise_count", &self.raise_count)
            .field("to_act_idx", &self.to_act_idx)
            .finish()
    }
}
impl fmt::Debug for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GameState")
            .field("num_players", &self.num_players)
            .field("num_active_players", &self.num_active_players())
            .field("player_active", &self.player_active)
            .field("player_all_in", &self.player_all_in)
            .field("total_pot", &self.total_pot)
            .field("stacks", &self.stacks)
            .field("big_blind", &self.big_blind)
            .field("small_blind", &self.small_blind)
            .field("hands", &self.hands)
            .field("dealer_idx", &self.dealer_idx)
            .field("round", &self.round)
            .field("round_data", &self.round_data)
            .field("board", &self.board)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fold_around_call() {
        let stacks = vec![100; 4];
        let mut game_state = GameState::new(stacks, 10, 5, 1);
        game_state.advance_round().unwrap();

        // 0 player, 1 dealer, 2 small blind, 3 big blind
        assert_eq!(0, game_state.current_round_data().unwrap().to_act_idx);
        dbg!(game_state.clone());
        game_state.fold().unwrap();
        game_state.fold().unwrap();
        game_state.do_bet(10, false).unwrap();
        game_state.do_bet(10, false).unwrap();
        assert_eq!(
            0,
            game_state
                .current_round_data()
                .unwrap()
                .num_active_players()
        );
        assert_eq!(2, game_state.num_active_players());

        // Flop
        game_state.advance_round().unwrap();
        assert_eq!(2, game_state.current_round_data().unwrap().to_act_idx);
        game_state.do_bet(0, false).unwrap();
        assert_eq!(3, game_state.current_round_data().unwrap().to_act_idx);
        game_state.do_bet(0, false).unwrap();
        assert_eq!(
            0,
            game_state
                .current_round_data()
                .unwrap()
                .num_active_players()
        );
        assert_eq!(2, game_state.num_active_players());

        // Turn
        game_state.advance_round().unwrap();
        assert_eq!(2, game_state.current_round_data().unwrap().to_act_idx);
        assert_eq!(
            2,
            game_state
                .current_round_data()
                .unwrap()
                .num_active_players()
        );
        game_state.do_bet(0, false).unwrap();
        game_state.do_bet(0, false).unwrap();
        assert_eq!(
            0,
            game_state
                .current_round_data()
                .unwrap()
                .num_active_players()
        );
        assert_eq!(2, game_state.num_active_players());

        // River
        game_state.advance_round().unwrap();
        game_state.do_bet(0, false).unwrap();
        game_state.do_bet(0, false).unwrap();
        assert_eq!(
            0,
            game_state
                .current_round_data()
                .unwrap()
                .num_active_players()
        );
        assert_eq!(2, game_state.num_active_players());

        game_state.advance_round().unwrap();
        assert_eq!(Round::Showdown, game_state.round);
    }
}

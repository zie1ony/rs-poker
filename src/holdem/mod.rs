/// Module that can generate possible cards for a starting hand.
mod starting_hand;
/// Export `StartingHand`
pub use self::starting_hand::{Suitedness, StartingHand};

/// Module for `Game` that will hold the current state of the game.
mod game;
/// Export `Game`
pub use self::game::Game;

/// Module with all the starting hand parsing code.
mod parse;
/// Export `RangeParser`
pub use self::parse::RangeParser;

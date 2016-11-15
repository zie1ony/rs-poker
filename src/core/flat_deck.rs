use std::ops::{Index, RangeTo, RangeFrom, RangeFull};
use core::card::Card;
use core::deck::Deck;

/// `FlatDeck` is a deck of cards that allows easy
/// indexing into the cards. It does not provide
/// contains methods.
#[derive(Debug)]
pub struct FlatDeck {
    /// Card storage.
    cards: Vec<Card>,
}

impl FlatDeck {
    /// How many cards are there in the deck ?
    pub fn len(&self) -> usize {
        self.cards.len()
    }
    /// Have all cards been dealt ?
    /// This probably won't be used as it's unlikely
    /// that someone will deal all 52 cards from a deck.
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}

impl Index<usize> for FlatDeck {
    type Output = Card;
    fn index(&self, index: usize) -> &Card {
        &self.cards[index]
    }
}
impl Index<RangeTo<usize>> for FlatDeck {
    type Output = [Card];
    fn index(&self, index: RangeTo<usize>) -> &[Card] {
        &self.cards[index]
    }
}
impl Index<RangeFrom<usize>> for FlatDeck {
    type Output = [Card];
    fn index(&self, index: RangeFrom<usize>) -> &[Card] {
        &self.cards[index]
    }
}
impl Index<RangeFull> for FlatDeck {
    type Output = [Card];
    fn index(&self, index: RangeFull) -> &[Card] {
        &self.cards[index]
    }
}


/// Trait that means a deck can be made into a `FlatDeck`
pub trait Flattenable {
    /// Consume a `Deck` and produce a deck suitable for random index.
    fn flatten(self) -> FlatDeck;
}

/// Allow creating a flat deck from a Deck
impl Flattenable for Deck {
    /// Flatten this deck, consuming it to produce a `FlatDeck` that's
    /// easier to get random access to.
    fn flatten(self) -> FlatDeck {
        FlatDeck { cards: self.into_iter().collect() }
    }
}

impl Into<FlatDeck> for Deck {
    fn into(self) -> FlatDeck {
        self.flatten()
    }
}

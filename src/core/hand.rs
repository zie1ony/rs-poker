use core::card::*;
use std::ops::Index;
use std::ops::{RangeFull, RangeTo, RangeFrom};
use std::slice::Iter;

/// Struct to hold cards.
///
/// This doesn't have the ability to easily check if a card is
/// in the hand. So do that before adding/removing a card.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Hand {
    /// Where all the cards are placed un-ordered.
    cards: Vec<Card>,
}

impl Hand {
    /// Create the default empty hand.
    pub fn default() -> Hand {
        Hand { cards: Vec::with_capacity(5) }
    }
    /// Create the hand with specific hand.
    pub fn new_with_cards(cards: Vec<Card>) -> Hand {
        Hand { cards: cards }
    }
    /// Add card at to the hand.
    /// No verification is done at all.
    pub fn push(&mut self, c: Card) {
        self.cards.push(c);
    }
    /// How many cards are in this hand so far ?
    pub fn len(&self) -> usize {
        self.cards.len()
    }
    /// Are there any cards at all ?
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Create an iter on the cards.
    pub fn iter(&self) -> Iter<Card> {
        self.cards.iter()
    }
}

/// Allow indexing into the hand.
impl Index<usize> for Hand {
    type Output = Card;
    fn index(&self, index: usize) -> &Card {
        &self.cards[index]
    }
}

/// Allow the index to get refernce to every card.
impl Index<RangeFull> for Hand {
    type Output = [Card];
    fn index(&self, range: RangeFull) -> &[Card] {
        &self.cards[range]
    }
}

impl Index<RangeTo<usize>> for Hand {
    type Output = [Card];
    fn index(&self, index: RangeTo<usize>) -> &[Card] {
        &self.cards[index]
    }
}
impl Index<RangeFrom<usize>> for Hand {
    type Output = [Card];
    fn index(&self, index: RangeFrom<usize>) -> &[Card] {
        &self.cards[index]
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use core::card::Card;

    #[test]
    fn test_add_card() {
        assert!(true);
        let mut h = Hand::default();
        let c = Card {
            value: Value::Three,
            suit: Suit::Spade,
        };
        h.push(c);
        // Make sure that the card was added to the vec.
        //
        // This will also test that has len works
        assert_eq!(1, h.len());

    }

    #[test]
    fn test_index() {
        let mut h = Hand::default();
        h.push(Card {
            value: Value::Four,
            suit: Suit::Spade,
        });
        // Make sure the card is there
        assert_eq!(Card {
                       value: Value::Four,
                       suit: Suit::Spade,
                   },
                   h[0]);
    }
}

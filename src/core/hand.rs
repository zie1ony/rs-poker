use core::card::*;
use std::ops::Index;
use std::ops::RangeFull;

#[derive(Debug, Clone, Hash)]
pub struct Hand {
    cards: Vec<Card>,
}

impl Hand {
    pub fn default() -> Hand {
        Hand { cards: Vec::with_capacity(5) }
    }
    pub fn new_with_cards(cards: Vec<Card>) -> Hand {
        Hand { cards: cards }

    }
    /// Add card at to the hand.
    /// No verification is done at all.
    pub fn push(&mut self, c: Card) {
        self.cards.push(c);
    }
    pub fn len(&self) -> usize {
        self.cards.len()
    }
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}

impl Index<usize> for Hand {
    type Output = Card;
    fn index(&self, index: usize) -> &Card {
        &self.cards[index]
    }
}

impl Index<RangeFull> for Hand {
    type Output = [Card];
    fn index(&self, range: RangeFull) -> &[Card] {
        &self.cards[range]
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use core::card::*;

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

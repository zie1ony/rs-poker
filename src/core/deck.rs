use core::card::{Value, Suit, Card};
use std::collections::HashSet;
use std::ops::{Index, RangeTo, RangeFrom, RangeFull};

#[derive(Debug)]
pub struct Deck {
    cards: HashSet<Card>,
}

impl Deck {
    pub fn default() -> Deck {
        let mut cards: HashSet<Card> = HashSet::new();
        for v in &Value::values() {
            for s in &Suit::suits() {
                cards.insert(Card {
                    value: v.clone(),
                    suit: s.clone(),
                });
            }
        }
        Deck { cards: cards }
    }
    pub fn contains(&self, c: &Card) -> bool {
        self.cards.contains(c)
    }
    pub fn remove(&mut self, c: &Card) -> bool {
        self.cards.remove(c)
    }
    pub fn len(&self) -> usize {
        self.cards.len()
    }
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
    pub fn flatten(self) -> FlatDeck {
        FlatDeck { cards: self.cards.into_iter().collect() }
    }
}

#[derive(Debug)]
pub struct FlatDeck {
    cards: Vec<Card>,
}

impl FlatDeck {
    pub fn len(&self) -> usize {
        self.cards.len()
    }
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

impl Into<FlatDeck> for Deck {
    fn into(self) -> FlatDeck {
        self.flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::card::*;

    #[test]
    fn test_contains_in() {
        let d = Deck::default();
        assert!(d.contains(&Card {
            value: Value::Eight,
            suit: Suit::Heart,
        }));
    }

    #[test]
    fn test_remove() {
        let mut d = Deck::default();
        let c = Card {
            value: Value::Ace,
            suit: Suit::Heart,
        };
        assert!(d.contains(&c));
        assert!(d.remove(&c));
        assert!(d.contains(&c) == false);
        assert!(d.remove(&c) == false);
    }

}

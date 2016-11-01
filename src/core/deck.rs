use core::card::{Value, Suit, Card};
use std::ops::Index;
use std::ops::RangeFull;

#[derive(Debug)]
pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn default() -> Deck {
        let cards: Vec<Card> = iproduct!(&Value::values(), &Suit::suits())
            .map(|(v, s)| {
                Card {
                    value: v.clone(),
                    suit: s.clone(),
                }
            })
            .collect();
        Deck { cards: cards }
    }
    fn index(&self, c: &Card) -> Result<usize, usize> {
        self.cards[..].binary_search(c)
    }
    pub fn contains(&self, c: &Card) -> bool {
        if let Ok(_) = self.index(c) {
            true
        } else {
            false
        }
    }
    pub fn remove(&mut self, c: &Card) -> bool {
        if let Ok(idx) = self.index(c) {
            self.cards.remove(idx);
            true
        } else {
            false
        }
    }
    pub fn len(&self) -> usize {
        self.cards.len()
    }
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}

impl Index<usize> for Deck {
    type Output = Card;
    fn index(&self, index: usize) -> &Card {
        &self.cards[index]
    }
}

impl Index<RangeFull> for Deck {
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

    #[test]
    fn test_remove_all() {
        let mut count = 0;
        let mut d = Deck::default();

        while !d.is_empty() && count < 100 {
            count += 1;
            let c: Card = d[0].clone();
            d.remove(&c);
            assert!(d.contains(&c) == false);
        }
        assert_eq!(52, count);
    }

    #[test]
    fn test_find() {
        let d = Deck::default();
        for i in 0..52 {
            assert_eq!(Ok(i), d.index(&d[i]));
        }
    }

    #[test]
    fn test_find_after_removal() {
        let mut d = Deck::default();
        let c_one = d[0].clone();
        let c_two = d[19].clone();
        let c_three = d[51].clone();
        d.remove(&c_one);
        assert!(d.contains(&c_one) == false);

        d.remove(&c_two);
        assert!(d.contains(&c_two) == false);

        d.remove(&c_three);
        assert!(d.contains(&c_three) == false);

        for i in 0..d.len() {
            assert_eq!(Ok(i), d.index(&d[i]));
        }
    }
}

use card::*;
use hand::Hand;
use deck::Deck;

#[derive(Debug)]
pub struct CardIter {
    // All the possible cards that can be dealt
    possible_cards: Vec<Card>,

    // Set of current offsets being used to create card sets.
    idx: Vec<i64>,

    // size of card sets requested.
    num_cards: usize,
}

impl CardIter {
    pub fn new(possible_cards: Vec<Card>, num_cards: usize) -> CardIter {
        let mut idx: Vec<i64> = (0..(num_cards as i64)).collect();
        idx[num_cards - 1] -= 1;
        CardIter {
            possible_cards: possible_cards,
            idx: idx,
            num_cards: num_cards,
        }
    }
}

impl Iterator for CardIter {
    type Item = Vec<Card>;
    fn next(&mut self) -> Option<Vec<Card>> {
        // Keep track of where we are mutating
        let mut current_level = self.num_cards - 1;

        while current_level < self.num_cards {
            // Move the current level forward one.
            self.idx[current_level] += 1;


            // Now check if moving this level forward means that
            // We will need more cards to fill out the rest of the hand
            // then are there.
            let cards_needed_after = self.num_cards - (current_level + 1);
            if self.idx[current_level] as usize >=
               (self.possible_cards.len() - cards_needed_after) {
                if current_level == 0 {
                    return None;
                }
                current_level -= 1;
            } else {
                // If we aren't at the end then
                if current_level < self.num_cards - 1 {
                    self.idx[current_level + 1] = self.idx[current_level];
                }
                // Move forward one level
                current_level += 1;
            }
        }

        let result_cards: Vec<Card> = self.idx
            .iter()
            .map(|i| self.possible_cards[*i as usize].clone())
            .collect();
        Some(result_cards)
    }
}


/// The default card iter will give back 5 cards.
///
/// Useful for trying to find the best 5 card hand from 7 cards.
impl IntoIterator for Hand {
    type Item = Vec<Card>;
    type IntoIter = CardIter;

    fn into_iter(self) -> CardIter {
        let possible_cards: Vec<Card> = self[..].to_vec();
        CardIter::new(possible_cards, 5)
    }
}
/// This is useful for trying every possible 5 card hand
///
/// Probably not something that's going to be done in real
/// use cases, but still not bad.
impl IntoIterator for Deck {
    type Item = Vec<Card>;
    type IntoIter = CardIter;

    fn into_iter(self) -> CardIter {
        let possible_cards: Vec<Card> = self[..].to_vec();
        CardIter::new(possible_cards, 5)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use card::*;
    use hand::*;
    use deck::*;
    use rank::Rankable;

    #[test]
    fn test_iter_one() {
        let mut h = Hand::default();
        h.push(Card {
            value: Value::Two,
            suit: Suit::Spade,
        });

        for cards in CardIter::new(h[..].to_vec(), 1) {
            assert_eq!(1, cards.len());
        }


        assert_eq!(1, CardIter::new(h[..].to_vec(), 1).count());
    }

    #[test]
    fn test_iter_two() {
        let mut h = Hand::default();
        h.push(Card {
            value: Value::Two,
            suit: Suit::Spade,
        });
        h.push(Card {
            value: Value::Three,
            suit: Suit::Spade,
        });
        h.push(Card {
            value: Value::Four,
            suit: Suit::Spade,
        });


        // Make sure that we get the correct number back.
        assert_eq!(3, CardIter::new(h[..].to_vec(), 2).count());

        // Make sure that everything has two cards and they are different.
        //
        for cards in CardIter::new(h[..].to_vec(), 2) {
            assert_eq!(2, cards.len());
            assert!(cards[0] != cards[1]);
        }
    }

    #[test]
    fn test_iter_deck() {
        let d = Deck::default();
        assert_eq!(2598960, d.into_iter().count());
    }

    #[test]
    fn test_iter_rank() {
        let d = Deck::default();
        for cards in d.into_iter() {
            let h = Hand::new_with_cards(cards);
            h.rank();
        }
    }
}

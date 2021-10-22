use fixedbitset::FixedBitSet;

use crate::core::*;

/// Current state of a game.
#[derive(Debug)]
pub struct MonteCarloGame {
    /// Flatten deck
    deck: FlatDeck,
    /// Community cards.
    board: Vec<Card>,
    /// Hands still playing.
    hands: Vec<Hand>,
    current_offset: usize,
}

impl MonteCarloGame {
    /// If we already have hands then lets start there.
    pub fn new_with_hands(hands: Vec<Hand>, board: Vec<Card>) -> Result<Self, String> {
        let mut deck = Deck::default();
        if board.len() > 5 {
            return Err(String::from("Board passed in has more than 5 cards"));
        }

        for hand in &hands {
            if hand.len() != 2 {
                return Err(String::from("Hand passed in doesn't have 2 cards."));
            }
            for card in hand.iter() {
                if !deck.remove(card) {
                    return Err(format!("Card {} was already removed from the deck.", card));
                }
            }
        }

        for card in &board {
            if !deck.remove(card) {
                return Err(format!("Card {} was already removed from the deck.", card));
            }
        }

        let flat_deck: FlatDeck = deck.into();
        // Grab the deck.len() so that any call to shuffle_if_needed
        // will result in a shuffling.
        let offset = flat_deck.len();

        Ok(Self {
            deck: flat_deck,
            hands,
            board,
            current_offset: offset,
        })
    }

    /// Simulate finishing a holdem game.
    ///
    /// This will fill out the board and then return the tuple
    /// of which hand had the best rank in end.
    pub fn simulate(&mut self) -> (FixedBitSet, Rank) {
        // Add the board cards to all the hands.
        for c in &self.board {
            for h in &mut self.hands {
                h.push(*c);
            }
        }
        // Figure out how many cards to deal.
        let num_cards = 5 - self.board.len();
        // Now iterate over a sample of the deck.
        self.shuffle_if_needed();
        for c in &self.deck[self.current_offset..self.current_offset + num_cards] {
            for h in &mut self.hands {
                h.push(*c);
            }
        }
        self.current_offset += num_cards;

        // Now get the best rank of all the possible hands.
        self.hands.iter().map(|h| h.rank()).enumerate().fold(
            (
                FixedBitSet::with_capacity(self.hands.len()),
                Rank::HighCard(0),
            ),
            |(mut found, max_rank), (idx, rank)| {
                match rank.cmp(&max_rank) {
                    std::cmp::Ordering::Equal => {
                        // If this is a tie then add the index.
                        found.set(idx, true);
                        (found, rank)
                    }
                    std::cmp::Ordering::Greater => {
                        // If this is the higest then reset all the bitset that's 1's
                        // Then set only the current hand's index as true
                        found.clear();
                        found.set(idx, true);
                        (found, rank)
                    }
                    // Otherwise keep what we've already found.
                    _ => (found, max_rank),
                }
            },
        )
    }

    /// Reset the game state.
    pub fn reset(&mut self) {
        for h in &mut self.hands {
            h.truncate(2);
        }
    }
    fn shuffle_if_needed(&mut self) {
        if self.current_offset + 5 > self.deck.len() {
            self.current_offset = 0;
            self.deck.shuffle();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::Hand;
    use crate::core::Rank;

    #[test]
    fn test_simulate_pocket_pair() {
        let hands = ["AdAh", "2c2s"]
            .iter()
            .map(|s| Hand::new_from_str(s).unwrap())
            .collect();
        let mut g = MonteCarloGame::new_with_hands(hands, vec![]).unwrap();
        let result = g.simulate();
        assert!(result.1 >= Rank::OnePair(0));
    }

    #[test]
    fn test_simulate_pocket_pair_with_board() {
        let board = vec![
            Card {
                suit: Suit::Spade,
                value: Value::Ace,
            },
            Card {
                suit: Suit::Diamond,
                value: Value::Three,
            },
            Card {
                suit: Suit::Diamond,
                value: Value::Four,
            },
        ];
        let hands = ["AdAh", "2c2s"]
            .iter()
            .map(|s| Hand::new_from_str(s).unwrap())
            .collect();
        let mut g = MonteCarloGame::new_with_hands(hands, board).unwrap();
        let result = g.simulate();
        assert!(result.1 >= Rank::ThreeOfAKind(0));
    }

    #[test]
    fn test_simulate_set() {
        let hands: Vec<Hand> = ["6d6h", "3d3h"]
            .iter()
            .map(|s| Hand::new_from_str(s).unwrap())
            .collect();
        let board: Vec<Card> = vec![
            Card {
                value: Value::Six,
                suit: Suit::Spade,
            },
            Card {
                value: Value::King,
                suit: Suit::Diamond,
            },
            Card {
                value: Value::Queen,
                suit: Suit::Heart,
            },
        ];
        let mut g = MonteCarloGame::new_with_hands(hands, board).unwrap();
        let result = g.simulate();
        assert!(result.1 >= Rank::ThreeOfAKind(4));
    }
}

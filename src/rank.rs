use hand::Hand;
use card::Value;
use vec_map::VecMap;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Rank {
    HighCard(usize),
    OnePair(usize),
    TwoPair(usize),
    ThreeOfAKind(usize),
    Straight(usize),
    Flush(usize),
    FullHouse(usize),
    FourOfAKind(usize),
    StraightFlush(usize),
}

// Big ugly constant for all the straghts.
pub const STRAIGHTS: [usize; 10] =
    [// Wheel.
     1 << (Value::Ace as usize) | 1 << (Value::Two as usize) | 1 << (Value::Three as usize) |
     1 << (Value::Four as usize) | 1 << (Value::Five as usize),
     // "Normal" straights starting at two to six.
     1 << (Value::Two as usize) | 1 << (Value::Three as usize) | 1 << (Value::Four as usize) |
     1 << (Value::Five as usize) | 1 << (Value::Six as usize),
     // Three to Seven
     1 << (Value::Three as usize) | 1 << (Value::Four as usize) | 1 << (Value::Five as usize) |
     1 << (Value::Six as usize) | 1 << (Value::Seven as usize),
     // Four to Eight
     1 << (Value::Four as usize) | 1 << (Value::Five as usize) | 1 << (Value::Six as usize) |
     1 << (Value::Seven as usize) | 1 << (Value::Eight as usize),
     // Five to Nine
     1 << (Value::Five as usize) | 1 << (Value::Six as usize) | 1 << (Value::Seven as usize) |
     1 << (Value::Eight as usize) | 1 << (Value::Nine as usize),
     // Six to Ten
     1 << (Value::Six as usize) | 1 << (Value::Seven as usize) | 1 << (Value::Eight as usize) |
     1 << (Value::Nine as usize) | 1 << (Value::Ten as usize),
     // Seven to Jack.
     1 << (Value::Seven as usize) | 1 << (Value::Eight as usize) | 1 << (Value::Nine as usize) |
     1 << (Value::Ten as usize) | 1 << (Value::Jack as usize),
     // Eight to Queen
     1 << (Value::Eight as usize) | 1 << (Value::Nine as usize) | 1 << (Value::Ten as usize) |
     1 << (Value::Jack as usize) | 1 << (Value::Queen as usize),
     // Nine to king
     1 << (Value::Nine as usize) | 1 << (Value::Ten as usize) | 1 << (Value::Jack as usize) |
     1 << (Value::Queen as usize) | 1 << (Value::King as usize),
     // Royal straight
     1 << (Value::Ten as usize) | 1 << (Value::Jack as usize) | 1 << (Value::Queen as usize) |
     1 << (Value::King as usize) | 1 << (Value::Ace as usize)];

pub trait Rankable {
    fn rank(&self) -> Rank;
    fn rank_straight(&self, hand_rank: usize) -> Option<usize> {
        for (i, hand) in STRAIGHTS.iter().enumerate() {
            if *hand == hand_rank {
                return Some(i);
            }
        }
        None
    }
}

impl Rankable for Hand {
    fn rank(&self) -> Rank {
        // use for bitset
        let mut suit_set: usize = 0;
        // Use for bitset
        let mut value_set: usize = 0;
        let mut value_to_count: VecMap<usize> = VecMap::with_capacity(13);
        let mut count_to_value: VecMap<Vec<Value>> = VecMap::with_capacity(13);
        let mut potential_hand_rank = 0;
        // TODO(eclark): make this more generic
        for c in self[..].iter() {
            let v = c.value.clone() as usize;
            let s = c.suit.clone() as usize;

            // Will be used for flush
            suit_set |= 1 << s as usize;
            // Will be used to determine straights.
            value_set |= 1 << v as usize;

            // If this is high card or a flush we need this.
            // It will be used to differentiate strenght of the same rank
            potential_hand_rank |= 1 << v;
            // Keep track of counts for each card.
            let e = value_to_count.entry(v).or_insert(0);
            *e += 1;
        }

        // Now rotate the value to count map.
        for (value, count) in value_to_count {
            // Get the entry for the map, or insert it into the map.
            let e = count_to_value.entry(count).or_insert_with(|| vec![]);
            // Now that it has to be there, push the new value.
            (*e).push(Value::from_usize(value));
        }

        // The major deciding factor for hand rank
        // is the number of unique card values.
        let unique_card_count = value_set.count_ones();

        // Now that we should have all the information needed.
        // Lets do this.
        if unique_card_count == 5 {
            // If there are five different cards it can be a straight
            // a straight flush, a flush, or just a high card.
            // Need to check for all of them.
            let suit_count = suit_set.count_ones();
            let is_flush = suit_count == 1;
            match (self.rank_straight(potential_hand_rank), is_flush) {
                (Some(rank), true) => Rank::StraightFlush(rank),
                (Some(rank), false) => Rank::Straight(rank),
                (None, true) => Rank::Flush(potential_hand_rank),
                (None, false) => Rank::HighCard(potential_hand_rank),
            }
        } else if unique_card_count == 2 {
            // This can either be full house, or four of a kind.
            match count_to_value.get(3) {
                Some(three_value) => {
                    let major_rank = 1 << three_value[0].clone() as usize;
                    // Remove the card that we have three of from the minor rank.
                    let minor_rank = potential_hand_rank ^ major_rank;
                    // then join the two ranks
                    Rank::FullHouse(major_rank << 13 | minor_rank)
                }
                None => {
                    let major_rank = 1 << (count_to_value[4][0].clone() as usize);
                    let minor_rank = potential_hand_rank ^ major_rank;
                    Rank::FourOfAKind(major_rank << 13 | minor_rank)
                }
            }
        } else if unique_card_count == 3 {
            // this can be three of a kind or two pair.
            match count_to_value.get(3) {
                Some(three_value) => {
                    let major_rank = 1 << (three_value[0].clone() as usize);
                    let minor_rank = potential_hand_rank ^ major_rank;
                    Rank::ThreeOfAKind(major_rank << 13 | minor_rank)
                }
                None => {
                    // get the values of the pairs
                    let pairs = &count_to_value[2];
                    let major_rank = 1 << pairs[0].clone() as usize |
                                     1 << pairs[1].clone() as usize;
                    let minor_rank = potential_hand_rank ^ major_rank;
                    Rank::TwoPair(major_rank << 13 | minor_rank)
                }
            }
        } else {
            // this is unique_card_count == 4
            assert!(unique_card_count == 4);
            let major_rank = 1 << count_to_value[2][0].clone() as usize;
            let minor_rank = potential_hand_rank ^ major_rank;
            Rank::OnePair(major_rank << 13 | minor_rank)
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use hand::*;
    use card::*;


    #[test]
    fn test_cmp() {
        assert!(Rank::HighCard(0) < Rank::StraightFlush(0));
        assert!(Rank::HighCard(0) < Rank::FourOfAKind(0));
        assert!(Rank::HighCard(0) < Rank::ThreeOfAKind(0));
    }

    #[test]
    fn test_cmp_high() {
        assert!(Rank::HighCard(0) < Rank::HighCard(100));
    }

    #[test]
    fn test_high_card_hand() {
        let hand = Hand::new_with_cards(vec![
                                             Card{value: Value::Ace, suit: Suit::Diamond},
                                             Card{value: Value::Eight, suit: Suit::Heart},
                                             Card{value: Value::Nine, suit: Suit::Club},
                                             Card{value: Value::Ten, suit: Suit::Club},
                                             Card{value: Value::Five, suit: Suit::Club},
                                             ]);

        let rank = 1 << Value::Ace as usize | 1 << Value::Eight as usize |
                   1 << Value::Nine as usize | 1 << Value::Ten as usize |
                   1 << Value::Five as usize;

        assert!(Rank::HighCard(rank) == hand.rank());
    }

    #[test]
    fn test_flush() {
        let hand = Hand::new_with_cards(vec![
                                             Card{value: Value::Ace, suit: Suit::Diamond},
                                             Card{value: Value::Eight, suit: Suit::Diamond},
                                             Card{value: Value::Nine, suit: Suit::Diamond},
                                             Card{value: Value::Ten, suit: Suit::Diamond},
                                             Card{value: Value::Five, suit: Suit::Diamond},
                                             ]);

        let rank = 1 << Value::Ace as usize | 1 << Value::Eight as usize |
                   1 << Value::Nine as usize | 1 << Value::Ten as usize |
                   1 << Value::Five as usize;

        assert!(Rank::Flush(rank) == hand.rank());
    }

    #[test]
    fn test_full_house() {
        let hand = Hand::new_with_cards(vec![
                                             Card{value: Value::Ace, suit: Suit::Diamond},
                                             Card{value: Value::Ace, suit: Suit::Club},
                                             Card{value: Value::Nine, suit: Suit::Diamond},
                                             Card{value: Value::Nine, suit: Suit::Club},
                                             Card{value: Value::Nine, suit: Suit::Spade},
                                             ]);

        let rank = (1 << (Value::Nine as usize)) << 13 | 1 << (Value::Ace as usize);
        assert!(Rank::FullHouse(rank) == hand.rank());
    }

    #[test]
    fn test_two_pair() {
        // Make a two pair hand.
        let hand = Hand::new_with_cards(vec![
                                             Card{value: Value::Ace, suit: Suit::Diamond},
                                             Card{value: Value::Ace, suit: Suit::Club},
                                             Card{value: Value::Nine, suit: Suit::Diamond},
                                             Card{value: Value::Nine, suit: Suit::Club},
                                             Card{value: Value::Ten, suit: Suit::Spade},
                                             ]);

        let rank = (1 << Value::Ace as usize | 1 << Value::Nine as usize) << 13 |
                   1 << Value::Ten as usize;
        assert!(Rank::TwoPair(rank) == hand.rank());
    }

    #[test]
    fn test_one_pair() {
        let hand = Hand::new_with_cards(vec![
                                             Card{value: Value::Ace, suit: Suit::Diamond},
                                             Card{value: Value::Ace, suit: Suit::Club},
                                             Card{value: Value::Nine, suit: Suit::Diamond},
                                             Card{value: Value::Eight, suit: Suit::Club},
                                             Card{value: Value::Ten, suit: Suit::Spade},
                                             ]);

        let rank = (1 << Value::Ace as usize) << 13 | 1 << Value::Nine as usize |
                   1 << Value::Eight as usize | 1 << Value::Ten as usize;

        assert!(Rank::OnePair(rank) == hand.rank());
    }

    #[test]
    fn test_four_of_a_kind() {
        let hand = Hand::new_with_cards(vec![
                                             Card{value: Value::Ace, suit: Suit::Diamond},
                                             Card{value: Value::Ace, suit: Suit::Club},
                                             Card{value: Value::Ace, suit: Suit::Spade},
                                             Card{value: Value::Ace, suit: Suit::Heart},
                                             Card{value: Value::Ten, suit: Suit::Spade},
                                             ]);

        assert!(Rank::FourOfAKind((1 << (Value::Ace as usize) << 13) |
                                  1 << (Value::Ten as usize)) == hand.rank());
    }

    #[test]
    fn test_wheel() {
        let hand = Hand::new_with_cards(vec![
                                             Card{value: Value::Ace, suit: Suit::Diamond},
                                             Card{value: Value::Two, suit: Suit::Club},
                                             Card{value: Value::Three, suit: Suit::Spade},
                                             Card{value: Value::Four, suit: Suit::Heart},
                                             Card{value: Value::Five, suit: Suit::Spade},
                                             ]);

        assert!(Rank::Straight(0) == hand.rank());
    }


    #[test]
    fn test_straight() {
        let hand = Hand::new_with_cards(vec![
                                             Card{value: Value::Two, suit: Suit::Club},
                                             Card{value: Value::Three, suit: Suit::Spade},
                                             Card{value: Value::Four, suit: Suit::Heart},
                                             Card{value: Value::Five, suit: Suit::Spade},
                                             Card{value: Value::Six, suit: Suit::Diamond},
                                             ]);

        assert!(Rank::Straight(1) == hand.rank());
    }

    #[test]
    fn test_three_of_a_kind() {
        let hand = Hand::new_with_cards(vec![
                                             Card{value: Value::Two, suit: Suit::Club},
                                             Card{value: Value::Two, suit: Suit::Spade},
                                             Card{value: Value::Two, suit: Suit::Heart},
                                             Card{value: Value::Five, suit: Suit::Spade},
                                             Card{value: Value::Six, suit: Suit::Diamond},
                                             ]);


        let rank = (1 << (Value::Two as usize)) << 13 | 1 << (Value::Five as usize) |
                   1 << (Value::Six as usize);

        assert!(Rank::ThreeOfAKind(rank) == hand.rank());
    }


    #[test]
    fn test_straight_constants() {
        for c in STRAIGHTS.iter() {
            // Make sure that all of the constant hands have exactly 5 ones.
            assert!(5 == c.count_ones());
        }
    }
}

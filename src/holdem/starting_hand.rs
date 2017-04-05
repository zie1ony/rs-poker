use core::{Value, Suit, Card, Hand};


#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Clone, Copy)]
pub enum Suitedness {
    Suited,
    OffSuit,
    Any,
}



/// `HoldemStartingHand` represents the two card starting hand of texas holdem.
/// It can generate all the possible actual starting hands.
///
/// Give two values and if you only want suited variants.
#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Clone)]
pub struct DefaultStartingHand {
    /// The first value.
    value_one: Value,
    /// The second value.
    value_two: Value,
    /// should we only consider possible starting hands of the same suit?
    suited: Suitedness,
}

impl DefaultStartingHand {
    fn create_suited(&self) -> Vec<Hand> {
        // Can't have a suited pair. Not unless you're cheating.
        if self.value_one == self.value_two {
            return vec![];
        }
        Suit::suits()
            .iter()
            .map(|s| {
                Hand::new_with_cards(vec![Card {
                                              value: self.value_one,
                                              suit: *s,
                                          },
                                          Card {
                                              value: self.value_two,
                                              suit: *s,
                                          }])
            })
            .collect()
    }

    fn create_offsuit(&self) -> Vec<Hand> {
        // Since the values are the same there is no reason to swap the suits.
        let expected_hands = if self.value_one == self.value_two {
            6
        } else {
            12
        };
        self.append_offsuit(Vec::with_capacity(expected_hands))
    }


    fn append_offsuit(&self, mut hands: Vec<Hand>) -> Vec<Hand> {
        let suits = Suit::suits();
        for (i, suit_one) in suits.iter().enumerate() {
            for suit_two in &suits[i + 1..] {
                // Push the hands in.
                hands.push(Hand::new_with_cards(vec![Card {
                                                         value: self.value_one,
                                                         suit: *suit_one,
                                                     },
                                                     Card {
                                                         value: self.value_two,
                                                         suit: *suit_two,
                                                     }]));

                // If this isn't a pair then the flipped suits is needed.
                if self.value_one != self.value_two {
                    hands.push(Hand::new_with_cards(vec![Card {
                                                             value: self.value_one,
                                                             suit: *suit_two,
                                                         },
                                                         Card {
                                                             value: self.value_two,
                                                             suit: *suit_one,
                                                         }]));
                }

            }
        }
        hands
    }

    /// Get all the possible starting hands represented by the
    /// two values of this starting hand.
    fn possible_hands(&self) -> Vec<Hand> {
        match self.suited {
            Suitedness::Suited => self.create_suited(),
            Suitedness::OffSuit => self.create_offsuit(),
            Suitedness::Any => self.append_offsuit(self.create_suited()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct SingleCardRangeStartingHand {
    value_one: Value,
    /// Inclusive start range
    start: Value,
    /// Inclusive end range
    end: Value,
    // What Suits can this have.
    suited: Suitedness,
}


impl SingleCardRangeStartingHand {
    fn possible_hands(&self) -> Vec<Hand> {
        let mut cur_value = self.start;
        let mut hands = vec![];
        while cur_value <= self.end {
            let mut new_hands = DefaultStartingHand {
                    value_one: self.value_one,
                    value_two: cur_value,
                    suited: self.suited,
                }
                .possible_hands();
            hands.append(&mut new_hands);
            cur_value = Value::from_u8(cur_value as u8 + 1);
        }

        hands
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub enum StartingHand {
    Def(DefaultStartingHand),
    SingleCardRange(SingleCardRangeStartingHand),
}

impl StartingHand {
    pub fn default(value_one: Value, value_two: Value, suited: Suitedness) -> StartingHand {
        StartingHand::Def(DefaultStartingHand {
                              value_one: value_one,
                              value_two: value_two,
                              suited: suited,
                          })
    }

    pub fn single_range(value_one: Value,
                        start: Value,
                        end: Value,
                        suited: Suitedness)
                        -> StartingHand {
        StartingHand::SingleCardRange(SingleCardRangeStartingHand {
                                          value_one: value_one,
                                          start: start,
                                          end: end,
                                          suited: suited,
                                      })
    }

    /// Create every possible unique StartingHand.
    pub fn all() -> Vec<StartingHand> {
        let mut hands = Vec::with_capacity(169);
        let values = Value::values();
        for (i, value_one) in values.iter().enumerate() {
            for value_two in &values[i..] {
                hands.push(StartingHand::Def(DefaultStartingHand {
                                                 value_one: *value_one,
                                                 value_two: *value_two,
                                                 suited: Suitedness::Any,
                                             }));
            }
        }
        hands
    }

    pub fn possible_hands(&self) -> Vec<Hand> {
        match *self {
            StartingHand::Def(ref h) => h.possible_hands(),
            StartingHand::SingleCardRange(ref h) => h.possible_hands(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::Value;

    #[test]
    fn test_aces() {
        let sh = DefaultStartingHand {
            value_one: Value::Ace,
            value_two: Value::Ace,
            suited: Suitedness::OffSuit,
        };
        println!("{:?}", sh.possible_hands());
        assert!(6 == sh.possible_hands().len());
    }

    #[test]
    fn test_suited_connector() {
        let sh = DefaultStartingHand {
            value_one: Value::Ace,
            value_two: Value::King,
            suited: Suitedness::Suited,
        };
        assert!(4 == sh.possible_hands().len());
    }
    #[test]
    fn test_unsuited_connector() {
        let sh = DefaultStartingHand {
            value_one: Value::Ace,
            value_two: Value::King,
            suited: Suitedness::OffSuit,
        };
        assert!(12 == sh.possible_hands().len());
    }

    #[test]
    fn test_starting_hand_count() {
        let num_to_test: usize = StartingHand::all()
            .iter()
            .map(|h| h.possible_hands().len())
            .sum();
        assert!(1326 == num_to_test);
    }
}

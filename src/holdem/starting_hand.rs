use core::{Value, Suit, Card, Hand};

/// `HoldemStartingHand` represents the two card starting hand of texas holdem.
/// It can generate all the possible actual starting hands.
///
/// Give two values and if you only want suited variants.
#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Clone)]
pub struct StartingHand {
    /// The first value.
    value_one: Value,
    /// The second value.
    value_two: Value,
    /// should we only consider possible starting hands of the same suit?
    suited: bool,
}

impl StartingHand {
    /// Get all the possible starting hands represented by the two values of this starting hand.
    pub fn possible_hands(&self) -> Vec<Hand> {
        if self.suited {
            Suit::suits()
                .iter()
                .map(|s| {
                    Hand::new_with_cards(vec![Card {
                                                  value: self.value_one.clone(),
                                                  suit: s.clone(),
                                              },
                                              Card {
                                                  value: self.value_two.clone(),
                                                  suit: s.clone(),
                                              }])
                })
                .collect()
        } else {
            // Since the values are the same there is no reason to swap the suits.
            let expected_hands = if self.value_one == self.value_two {
                6
            } else {
                12
            };
            let mut hands: Vec<Hand> = Vec::with_capacity(expected_hands);
            let suits = Suit::suits();
            for (i, suit_one) in suits.iter().enumerate() {
                for suit_two in &suits[i + 1..] {
                    // Push the hands in.
                    hands.push(Hand::new_with_cards(vec![Card {
                                                             value: self.value_one.clone(),
                                                             suit: suit_one.clone(),
                                                         },
                                                         Card {
                                                             value: self.value_two.clone(),
                                                             suit: suit_two.clone(),
                                                         }]));

                    // If this isn't a pair then the flipped suits is needed.
                    if self.value_one != self.value_two {
                        hands.push(Hand::new_with_cards(vec![Card {
                                                                 value: self.value_one.clone(),
                                                                 suit: suit_two.clone(),
                                                             },
                                                             Card {
                                                                 value: self.value_two.clone(),
                                                                 suit: suit_one.clone(),
                                                             }]));
                    }

                }
            }
            hands
        }
    }

    pub fn all() -> Vec<StartingHand> {
        let mut hands: Vec<StartingHand> = vec![];
        let values = Value::values();
        for (i, value_one) in values.iter().enumerate() {
            for value_two in &values[i..] {
                hands.push(StartingHand {
                    value_one: value_one.clone(),
                    value_two: value_two.clone(),
                    suited: false,
                });

                if value_one != value_two {
                    hands.push(StartingHand {
                        value_one: value_one.clone(),
                        value_two: value_two.clone(),
                        suited: true,
                    });
                }

            }
        }
        hands
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::Value;

    #[test]
    fn test_aces() {
        let sh = StartingHand {
            value_one: Value::Ace,
            value_two: Value::Ace,
            suited: false,
        };
        println!("{:?}", sh.possible_hands());
        assert!(6 == sh.possible_hands().len());
    }

    #[test]
    fn test_suited_connector() {
        let sh = StartingHand {
            value_one: Value::Ace,
            value_two: Value::King,
            suited: true,
        };
        assert!(4 == sh.possible_hands().len());
    }
    #[test]
    fn test_unsuited_connector() {
        let sh = StartingHand {
            value_one: Value::Ace,
            value_two: Value::King,
            suited: false,
        };
        assert!(12 == sh.possible_hands().len());
    }

    #[test]
    fn test_starting_hand_count() {

        let num_to_test: usize = StartingHand::all().iter().map(|h| h.possible_hands().len()).sum();
        assert!(1326 == num_to_test);
    }
}

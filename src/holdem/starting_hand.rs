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
    pub fn get_possible_hands(&self) -> Vec<Hand> {
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
            iproduct!(&Suit::suits(), &Suit::suits())
                .map(|(s_one, s_two)| {
                    (Card {
                        value: self.value_one.clone(),
                        suit: s_one.clone(),
                    },
                     Card {
                        value: self.value_two.clone(),
                        suit: s_two.clone(),
                    })
                })
                .filter_map(|(a, b)| {
                    if a != b {
                        Some(Hand::new_with_cards(vec![a, b]))
                    } else {
                        None
                    }
                })
                .collect()
        }
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
        assert!(12 == sh.get_possible_hands().len());
    }

    #[test]
    fn test_suited_connector() {
        let sh = StartingHand {
            value_one: Value::Ace,
            value_two: Value::King,
            suited: true,
        };
        assert!(4 == sh.get_possible_hands().len());
    }
    #[test]
    fn test_unsuited_connector() {
        let sh = StartingHand {
            value_one: Value::Ace,
            value_two: Value::King,
            suited: false,
        };
        assert!(16 == sh.get_possible_hands().len());
    }
}

use std::mem;

#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Hash)]
pub enum Value {
    Two = 0,
    Three = 1,
    Four = 2,
    Five = 3,
    Six = 4,
    Seven = 5,
    Eight = 6,
    Nine = 7,
    Ten = 8,
    Jack = 9,
    Queen = 10,
    King = 11,
    Ace = 12,
}

impl Value {
    pub fn from_usize(v: usize) -> Value {
        unsafe { mem::transmute(v as u8) }
    }
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Hash)]
pub enum Suit {
    Spade = 0,
    Club = 1,
    Heart = 2,
    Diamond = 3,
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Hash)]
pub struct Card {
    pub value: Value,
    pub suit: Suit,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn test_constructor() {
        let c = Card {
            value: Value::Three,
            suit: Suit::Spade,
        };
        assert_eq!(Suit::Spade, c.suit);
        assert_eq!(Value::Three, c.value);
    }

    #[test]
    fn test_compare() {
        let c1 = Card {
            value: Value::Three,
            suit: Suit::Spade,
        };
        let c2 = Card {
            value: Value::Four,
            suit: Suit::Spade,
        };
        let c3 = Card {
            value: Value::Four,
            suit: Suit::Club,
        };

        // Make sure that equals works
        assert!(c1 == c1);
        // Make sure that the values are ordered
        assert!(c1 < c2);
        assert!(c2 > c1);
        // Make sure that suit is used.
        assert!(c3 > c2);
    }

    #[test]
    fn test_value_cmp() {
        assert!(Value::Two < Value::Ace);
        assert!(Value::King < Value::Ace);
        assert_eq!(Value::Two, Value::Two);
    }

    #[test]
    fn test_size() {
        // Card should be really small. Hopefully just two u8's
        assert!(mem::size_of::<Card>() <= 4);
    }
}

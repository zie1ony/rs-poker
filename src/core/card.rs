use std::mem;

#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Hash)]
pub enum Value {
    /// 2
    Two = 0,
    /// 3
    Three = 1,
    /// 4
    Four = 2,
    /// 5
    Five = 3,
    /// 6
    Six = 4,
    /// 7
    Seven = 5,
    /// 8
    Eight = 6,
    /// 9
    Nine = 7,
    /// T
    Ten = 8,
    /// J
    Jack = 9,
    /// Q
    Queen = 10,
    /// K
    King = 11,
    /// A
    Ace = 12,
}

const VALUES: [Value; 13] = [Value::Two,
                             Value::Three,
                             Value::Four,
                             Value::Five,
                             Value::Six,
                             Value::Seven,
                             Value::Eight,
                             Value::Nine,
                             Value::Ten,
                             Value::Jack,
                             Value::Queen,
                             Value::King,
                             Value::Ace];

impl Value {
    pub fn from_usize(v: usize) -> Value {
        unsafe { mem::transmute(v as u8) }
    }
    pub fn values() -> [Value; 13] {
        VALUES
    }
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Hash)]
pub enum Suit {
    /// Spades
    Spade = 0,
    /// Clubs
    Club = 1,
    /// Hearts
    Heart = 2,
    /// Diamonds
    Diamond = 3,
}

const SUITS: [Suit; 4] = [Suit::Spade, Suit::Club, Suit::Heart, Suit::Diamond];

/// Impl of Suit
///
/// This is just here to provide a list of all `Suit`'s.
impl Suit {
    /// Provide all the Suit's that there are.
    pub fn suits() -> [Suit; 4] {
        SUITS
    }
}

/// The main struct of this library.
/// This is a carrier for Suit and Value combined.
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

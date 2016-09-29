
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Rank {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    Straight,
    Flush,
    FullHouse,
    FourOfAKind,
    StraightFlush,
}

pub trait Rankable {}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmp() {
        assert!(Rank::HighCard < Rank::StraightFlush);
        assert!(Rank::HighCard < Rank::FourOfAKind);
        assert!(Rank::HighCard < Rank::ThreeOfAKind);
    }
}

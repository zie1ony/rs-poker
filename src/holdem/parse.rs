use holdem::{StartingHand, Suitedness};
use core::{Suit, Value};
use std::iter::Peekable;
use std::str::Chars;


/// Struct to contain cards as they are being built.
/// These will not be passed outside this module. They are just for parsing.
#[derive(Debug)]
struct CardBuilder {
    suit: Option<Suit>,
    value: Value,
}

impl CardBuilder {
    fn set_suit(self, suit: Suit) -> CardBuilder {
        CardBuilder {
            suit: Some(suit),
            value: self.value,
        }
    }
}

#[derive(Debug)]
enum Modifier {
    Plus,
    Range,
    Suited,
    Offsuit,
}

impl Modifier {
    fn from_char(c: char) -> Option<Modifier> {
        match c {
            '+' => Some(Modifier::Plus),
            's' => Some(Modifier::Suited),
            'o' => Some(Modifier::Offsuit),
            '-' => Some(Modifier::Range),
            _ => None,
        }
    }
}

pub struct RangeParser;

impl RangeParser {
    /// Parse a string and return all the starting hands
    pub fn parse_one(range_str: &str) -> Result<StartingHand, String> {

        // TODO(eclark): Fix the error types to be better.

        // Consume the string, turning it into an iterator of chars.
        let mut iter = range_str.chars().peekable();
        // Create a card.
        let first_card = RangeParser::create_card(&mut iter, true)?;
        // Create the second card. If there is some suit for the second card then
        // we assume that there will be a second suit.
        let second_card = RangeParser::create_card(&mut iter, first_card.suit.is_some())?;

        // After we have two cards, there can be other things that specify
        let mod_char = *iter.peek().unwrap_or(&'_');


        // Assume that we have only specified on card.
        let mut plus = false;
        let mut range = false;

        // Assume that there's no suit stuff specified.
        let mut suitedness = Suitedness::Any;

        // Try and parse the modifier.
        if let Some(modifier) = Modifier::from_char(mod_char) {
            iter.next();
            match modifier {
                Modifier::Offsuit => suitedness = Suitedness::OffSuit,
                Modifier::Suited => suitedness = Suitedness::Suited,
                Modifier::Plus => plus = true,
                Modifier::Range => range = true,
            }
        }

        if plus {
            if range {
                return Err(String::from("Can't specify range and plus in the same hands."));
            }
            Ok(StartingHand::single_range(first_card.value,
                                          second_card.value,
                                          first_card.value,
                                          suitedness))
        } else if range {
            unimplemented!()
        } else {
            // TODO Right now this doesn't work with specified suits.
            Ok(StartingHand::default(first_card.value, second_card.value, suitedness))
        }

    }

    /// From a mut Peekable<Chars> this will take the first chars and bring out
    /// a CardBuilder.
    fn create_card(peekable: &mut Peekable<Chars>,
                   allow_suit: bool)
                   -> Result<CardBuilder, String> {
        // Try and get the first char.
        // Use something that should never be in a range as the default.
        let cv = *peekable.peek().unwrap_or(&'_');
        // Now try and parse it as a value char.
        if let Some(value) = Value::from_char(cv) {
            let mut card = CardBuilder {
                suit: None,
                value: value,
            };
            // Need to consume that character since it was useful.
            peekable.next();
            // Now if we allow suits try it aall again, this time with suit.
            if allow_suit {
                let cs = *peekable.peek().unwrap_or(&'_');
                if let Some(suit) = Suit::from_char(cs) {
                    card = card.set_suit(suit);
                    peekable.next();
                }
            }
            Ok(card)
        } else {
            Err(String::from("Unable to parse a card."))
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use core::*;

    #[test]
    fn test_explicit_card_builder() {
        let mut i = "AK".chars().peekable();
        let c = RangeParser::create_card(&mut i, false).unwrap();
        assert_eq!(Value::Ace, c.value);
        assert_eq!(None, c.suit);
    }

    #[test]
    fn test_easy_parse() {
        let shs = RangeParser::parse_one(&String::from("AK")).unwrap();
        assert_eq!(StartingHand::default(Value::Ace, Value::King, Suitedness::Any),
                   shs);
        assert_eq!(StartingHand::default(Value::Ace, Value::King, Suitedness::Any)
                       .possible_hands(),
                   shs.possible_hands());

    }
}

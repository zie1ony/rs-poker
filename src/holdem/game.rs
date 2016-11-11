use core::{Deck, FlatDeck, Card, Hand};

#[derive(Debug)]
pub struct Game {
    deck: FlatDeck,
    board: Vec<Card>,
    hands: Vec<Hand>,
}

impl Game {
    pub fn new(num_players: usize) -> Game {
        Game {
            deck: Deck::default().flatten(),
            board: Vec::with_capacity(5),
            hands: (0..num_players).map(|_| Hand::default()).collect(),
        }
    }

    pub fn new_with_hands(hands: Vec<Hand>) -> Game {
        let mut d = Deck::default();
        for h in &hands {
            for c in h.iter() {
                let _ = d.remove(c);
            }
        }
        Game {
            deck: d.flatten(),
            hands: hands,
            board: vec![],
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_create_game() {
        let g = Game::new(9);
        assert!(g.deck.len() == 52);
        assert!(g.hands.len() == 9);
    }
}

#![feature(test)]
extern crate furry_fiesta;
extern crate test;
extern crate rand;

use furry_fiesta::core::{Deck, Hand, FlatDeck, CardIter};

#[bench]
fn iter_in_deck(b: &mut test::Bencher) {
    b.iter(|| {
        let d: FlatDeck = Deck::default().into();
        d.into_iter().count()
    });
}

#[bench]
fn iter_hand(b: &mut test::Bencher) {
    let d: FlatDeck = Deck::default().into();
    let hand = Hand::new_with_cards(d.sample(7));

    b.iter(|| CardIter::new(&hand[..], 5).count())
}
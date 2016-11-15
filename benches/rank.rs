#![feature(test)]
extern crate furry_fiesta;
extern crate test;
extern crate rand;

use furry_fiesta::core::{Deck, Rankable, Hand, Flattenable};

#[bench]
fn rank_one(b: &mut test::Bencher) {
    let d = Deck::default().flatten();
    let hand = Hand::new_with_cards(d.sample(5));
    b.iter(|| hand.rank())
}
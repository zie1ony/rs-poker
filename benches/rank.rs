#![feature(test)]
extern crate furry_fiesta;
extern crate test;
extern crate rand;

use furry_fiesta::core::{Deck, Rankable, Hand, Rank};
use rand::{thread_rng, sample};

#[bench]
fn rank_one(b: &mut test::Bencher) {
    let mut rng = thread_rng();
    let d = Deck::default();
    let cards = sample(&mut rng, &d[..], 5).iter().map(|c| (*c).clone()).collect();
    let hand = Hand::new_with_cards(cards);
    b.iter(|| hand.rank())
}
#![feature(test)]
extern crate furry_fiesta;
extern crate test;
extern crate rand;

use furry_fiesta::holdem::StartingHand;
use rand::{thread_rng, sample};


#[bench]
fn all_starting(b: &mut test::Bencher) {
    b.iter(|| StartingHand::all())
}

#[bench]
fn iter_everything(b: &mut test::Bencher) {
    b.iter(|| {
        let s: usize = StartingHand::all().iter().map(|sh| sh.possible_hands().len()).sum();
        s
    })
}
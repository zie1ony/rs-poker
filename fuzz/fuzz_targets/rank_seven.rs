#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate rs_poker;
use rs_poker::core::{CardIter, FlatHand, Rankable};
use std::str;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = str::from_utf8(data) {
        if let Ok(h) = FlatHand::new_from_str(s) {
            if h.len() == 7 {
                let r_seven = h.rank();
                let r_five_max = CardIter::new(&h[..], 5)
                    .map(|cv| cv.rank_five())
                    .max()
                    .unwrap();
                assert_eq!(r_five_max, r_seven);
            }
        }
    }
});

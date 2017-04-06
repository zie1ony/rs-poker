#![feature(test)]
extern crate rs_poker;
extern crate test;
extern crate rand;

use rs_poker::holdem::RangeParser;

#[bench]
fn parse_ak_off(b: &mut test::Bencher) {
    b.iter(|| RangeParser::parse_one("AKo"));
}

#[bench]
fn parse_pairs(b: &mut test::Bencher) {
    b.iter(|| RangeParser::parse_one("22+"));
}

#[bench]
fn parse_connectors(b: &mut test::Bencher) {
    b.iter(|| RangeParser::parse_one("32+"));
}

#[bench]
fn parse_plus(b: &mut test::Bencher) {
    b.iter(|| RangeParser::parse_one("A2+"));
}

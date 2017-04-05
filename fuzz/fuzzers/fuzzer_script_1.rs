#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate furry_fiesta;
use std::str;
use furry_fiesta::holdem::RangeParser;


fuzz_target!(|data: &[u8]| {
    if let Ok(s) = str::from_utf8(data) {
      if let Ok(h) = RangeParser::parse_one(s) {
        assert!(h.len() > 0);
      }
    }
});

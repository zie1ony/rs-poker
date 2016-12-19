//! Furry Fiesta is a library for poker.
//! It is not the fastest hand ranking. However it is
//! clean well tested code.
#![feature(box_syntax, box_patterns)]

/// Allow all the core poker functionality to be used
/// externally. Everything in core should be agnostic
/// to poker style.
pub mod core;
/// Allow all the holdem specific code to be used externally.
pub mod holdem;
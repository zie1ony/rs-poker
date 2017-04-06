# rs-poker

RS Poker is a rust library aimed at being a good starting place
for lots of poker rust code.

## Core

The Core module contains code that is not specific to different
types of poker games. It contains:

* Suit type
* Value type
* Card type
* Deck
* Hand iteration
* Poker hand rank type
* Poker hand evaluation

The poker hand evaluation will rank a hand in ~16 nanoseconds
per hand. That means that 62 Million hands per second can be
ranked.

The hand evaluation is for 5 card hands. It is fully accurate,
it does not rely on just single kicker. This allows for breaking
ties on hands that are closer.


## Holdem

The holdem module contains code that is specific to holdem. It
currently contains:

* Starting hand enumeration
* Hand range parsing


## Testing

The code is pretty well tested and benchmarked. If you find 
something that looks like a bug please submit a pr with test
code.

The parsing code has been fuzzed. However the rest of the code
has not.

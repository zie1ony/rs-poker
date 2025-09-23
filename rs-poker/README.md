# rs-poker

[![Crates.io](https://img.shields.io/crates/v/rs-poker.svg)](https://crates.io/crates/rs-poker)
[![Docs.rs](https://docs.rs/rs_poker/badge.svg)](https://docs.rs/rs_poker)

RS Poker is a rust library aimed to be a good starting place for many poker rust
codes. Correctness and performance are the two primary goals.

## Core

The Core module contains code not specific to different types of poker games. It
contains:

- Suit type
- Value type
- Card type
- Deck
- Hand iteration
- Poker hand rank type
- Poker hand evaluation for five-card hands.
- Poker hand evaluation for seven card hands.
- PlayerBitSet is suitable for keeping track of boolean values on a table.

The poker hand (5 cards) evaluation will rank a hand in ~20 nanoseconds per
hand. That means that 50 Million hands per second can be ranked per CPU core.
The seven-card hand evaluation will rank a hand in < 25 ns.

The hand evaluation is accurate. `rs-poker` does not rely on just a single
kicker. This accuracy allows for breaking ties on hands that are closer.

## Holdem

The holdem module contains code that is specific to holdem. It currently
contains:

- Starting hand enumeration
- Hand range parsing
- Monte Carlo game simulation helpers.

## Arena

Arena is a feature that allows the creating of agents that play a simulated
Texas Holdem poker game. These autonomous agent vs agent games are ideal for
determining the strength of automated strategies. Additionally, agent vs agent
arenas are a good way of quickly playing lots of GTO poker.

Do you think you can create a better poker agent? The Arena module is a good place to
start. The Arena module contains:

- Holdem simulation struct for the overall status of the simulation
- Game state for the state of the current game
- Agent trait that you can implement to create your more potent poker agent.
- A few example Agents.
- Historians who can watch every action in a simulation as it happens

### Arena CFR Agent

`CFRAgent` is an agent that uses the Counterfactual Regret Minimization
algorithm to choose the best action. The agent is a good starting point for
creating a strong poker agent.

To implement your own strategy you will need to build a new `ActionGenerator`.
The `ActionGenerator` is responsible for generating all possible actions for a
given game state. The `CFRAgent` will then explore possible results of trying
the actions suggested by `ActionGenerator`. The Agent will choose the action it
would most regret not taking.

## Testing

The code is well-tested and benchmarked. If you find something that looks like a
bug, please submit a PR with an updated test code.

5 Card + Hand iteration is used with fuzzing to validate the seven-card hand
evaluation.

Fuzzing is used to validate game simulation via replay generation.

Multi-agent simulations are used to validate correctness and performance.

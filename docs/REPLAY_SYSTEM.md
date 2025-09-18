# Poker Game Replay System

This document explains how to replay poker games with the exact same cards dealt to players and on the board.

## Overview

The rs-poker library provides a comprehensive replay system that allows you to:

1. **Record all actions** during a poker game
2. **Replay the exact same cards** dealt to players and community board
3. **Time travel** to any point in the game history
4. **Reconstruct game state** from minimal action data

This is perfect for implementing async HTTP servers where bots can make decisions at their own pace.

## How It Works

### 1. Action Recording

During gameplay, the simulation automatically records all actions via the `Historian` pattern:

- `DealStartingHand` - Records each card dealt to each player
- `DealCommunity` - Records each community card dealt
- `PlayedAction` - Records player actions (bet, call, fold, etc.)
- `ForcedBet` - Records blinds and antes
- `RoundAdvance` - Records round transitions
- `Award` - Records pot distributions

### 2. State Reconstruction

The `GameReplay` system can reconstruct any game state by replaying actions sequentially:

```rust
use rs_poker::arena::{GameReplay, GameState, action::Action};

// Create replay from recorded actions
let replay = GameReplay::new(initial_state, actions);

// Step through actions one by one
while replay.has_more_actions() {
    let action = replay.step_forward()?;
    // Game state is automatically updated
}

// Or jump to any specific point
replay.step_to(action_index)?;
```

## Basic Usage

### Recording a Game

```rust
use rs_poker::arena::{
    HoldemSimulationBuilder, GameState, Agent,
    historian::VecHistorian,
    agent::VecReplayAgent,
    action::AgentAction,
};
use rand::{SeedableRng, rngs::StdRng};

// Set up game with historian to record actions
let historian = Box::new(VecHistorian::new());
let records_storage = historian.get_storage();

let stacks = vec![100.0, 100.0];
let agents: Vec<Box<dyn Agent>> = vec![
    Box::new(VecReplayAgent::new(vec![AgentAction::Call])),
    Box::new(VecReplayAgent::new(vec![AgentAction::Call])),
];

let game_state = GameState::new_starting(stacks, 10.0, 5.0, 0.0, 0);
let mut rng = StdRng::seed_from_u64(12345);

let mut sim = HoldemSimulationBuilder::default()
    .game_state(game_state)
    .agents(agents)
    .historians(vec![historian])
    .build()
    .unwrap();

sim.run(&mut rng);

// Extract recorded actions
let records = records_storage.borrow();
let actions: Vec<Action> = records.iter().map(|r| r.action.clone()).collect();
```

### Replaying a Game

```rust
use rs_poker::arena::{GameReplay, GameState};

// Create initial game state
let initial_state = GameState::new_starting(stacks, 10.0, 5.0, 0.0, 0);

// Create replay system
let mut replay = GameReplay::new(initial_state, actions);

// Replay to the end
while replay.has_more_actions() {
    replay.step_forward()?;
}

// The final board will be identical to the original game
println!("Final board: {:?}", replay.get_current_state().board);
```

## Advanced Features

### Time Travel

Jump to any point in the game:

```rust
// Go to action 10 (e.g., after flop dealing)
replay.step_to(10)?;
let flop_board = replay.get_current_state().board.clone();

// Go to action 20 (e.g., after turn)
replay.step_to(20)?;
let turn_board = replay.get_current_state().board.clone();

// Go back to action 5
replay.step_to(5)?;
```

### Inspecting Game State

```rust
let current_state = replay.get_current_state();

// Check community cards
println!("Board: {:?}", current_state.board);

// Check player hands (hole cards + community cards)
for (i, hand) in current_state.hands.iter().enumerate() {
    println!("Player {} hand: {:?}", i, hand);
}

// Check stacks
println!("Stacks: {:?}", current_state.stacks);

// Check current round
println!("Round: {:?}", current_state.round);
```

### Async HTTP Server Integration

This replay system is perfect for async HTTP servers:

```rust
// Pseudo-code for async server endpoint
async fn get_game_state_at_action(
    game_id: String, 
    action_index: usize
) -> Result<GameState, ServerError> {
    // Load recorded actions from database
    let actions = load_game_actions(&game_id).await?;
    let initial_state = load_initial_game_state(&game_id).await?;
    
    // Create replay and jump to desired point
    let mut replay = GameReplay::new(initial_state, actions);
    replay.step_to(action_index)?;
    
    Ok(replay.get_current_state().clone())
}

async fn bot_make_decision(
    game_id: String,
    current_action_index: usize
) -> Result<AgentAction, ServerError> {
    // Get current game state
    let state = get_game_state_at_action(game_id, current_action_index).await?;
    
    // Bot makes decision based on current state
    let action = bot_ai_decision(&state).await?;
    
    Ok(action)
}
```

## Key Benefits

### 1. **Deterministic Reproduction**
- Same actions → Same game state
- Perfect for debugging and testing
- Enables deterministic simulations

### 2. **Storage Efficiency**
- Store only actions, not full game states
- Actions are much smaller than complete states
- Easy to serialize/deserialize

### 3. **Async-Ready**
- No blocking operations
- Can pause/resume at any point
- Perfect for web servers

### 4. **Time Travel**
- Jump to any point in game history
- Great for analysis and visualization
- Enables "what-if" scenarios

## Example: Complete Workflow

See `examples/replay_demo.rs` for a complete working example that demonstrates:

1. Running a game with action recording
2. Extracting recorded actions
3. Replaying the exact same cards
4. Verifying card reproduction
5. Time travel functionality

Run it with:
```bash
cargo run --example replay_demo
```

## Use Cases

### 1. **Async Poker Servers**
- Bots connect via HTTP
- Each bot gets current game state
- Server waits for bot decisions
- Game progresses when all decisions received

### 2. **Game Analysis**
- Replay tournaments
- Analyze decision points
- Study bot behavior

### 3. **Testing & Debugging**
- Reproduce specific scenarios
- Debug game logic
- Validate rule implementations

### 4. **Game Visualization**
- Step-by-step game visualization
- Interactive game browser
- Historical game analysis

This system provides everything you need to implement a robust, async-capable poker server with perfect game state reconstruction.

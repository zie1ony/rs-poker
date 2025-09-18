# Tournament Replay System

This document explains how to replay entire poker tournaments with exact card reproduction for every hand.

## Overview

The tournament replay system extends the game replay functionality to handle complete tournaments, allowing you to:

1. **Record entire tournaments** hand by hand
2. **Replay any specific hand** with exact cards
3. **Step through tournament progression** 
4. **Time travel** to any point in the tournament
5. **Analyze tournament results** and player performance

This is perfect for implementing async HTTP servers for tournament analysis, bot training, and tournament visualization.

## Key Components

### 1. `TournamentReplayData`

Stores the complete tournament history:

```rust
pub struct TournamentReplayData {
    /// Actions from each hand, indexed by hand number
    pub hands: Vec<Vec<Action>>,
    /// Initial tournament state
    pub initial_state: GameState,
    /// Final results of the tournament
    pub results: Option<TournamentResults>,
}
```

### 2. `TournamentReplay`

Provides navigation through tournament history:

```rust
pub struct TournamentReplay {
    replay_data: TournamentReplayData,
    current_hand_index: usize,
    current_tournament_state: GameState,
}
```

### 3. `TournamentHistorian`

Records actions during live tournament play:

```rust
pub struct TournamentHistorian {
    tournament_data: TournamentReplayData,
    current_hand_actions: Vec<Action>,
}
```

## Recording Tournaments

### Basic Tournament Recording

```rust
use rs_poker::arena::{
    GameState, TournamentHistorian,
    competition::SingleTableTournamentBuilder,
    agent::{AllInAgentGenerator, FoldingAgentGenerator},
    historian::CloneHistorianGenerator,
};

// Create initial tournament state
let stacks = vec![1000.0, 1000.0, 1000.0, 1000.0];
let initial_state = GameState::new_starting(stacks, 20.0, 10.0, 5.0, 0);

// Create tournament historian
let tournament_historian = TournamentHistorian::new(initial_state.clone());
let historian_generator = CloneHistorianGenerator::new(tournament_historian);

// Set up agents
let agent_generators = vec![
    Box::new(AllInAgentGenerator::default()),
    Box::new(FoldingAgentGenerator::default()),
    Box::new(FoldingAgentGenerator::default()),
    Box::new(FoldingAgentGenerator::default()),
];

// Run tournament with recording
let tournament = SingleTableTournamentBuilder::default()
    .agent_generators(agent_generators)
    .historian_generators(vec![Box::new(historian_generator)])
    .starting_game_state(initial_state)
    .build()
    .unwrap();

let results = tournament.run().unwrap();

// Extract recorded data
let tournament_data = tournament_historian.get_tournament_data();
```

### Manual Hand Recording

For more control, you can record individual hands:

```rust
use rs_poker::arena::{
    TournamentReplayData, TournamentHistorian,
    historian::VecHistorian,
};

let mut tournament_data = TournamentReplayData::new(initial_state);

// For each hand in the tournament:
for hand_index in 0..num_hands {
    let historian = VecHistorian::new();
    let records_storage = historian.get_storage();
    
    // Run the hand with historian
    let mut sim = HoldemSimulationBuilder::default()
        .game_state(current_game_state)
        .agents(agents)
        .historians(vec![Box::new(historian)])
        .build()
        .unwrap();
    
    sim.run(&mut rng);
    
    // Extract actions and add to tournament data
    let records = records_storage.borrow();
    let hand_actions: Vec<Action> = records.iter()
        .map(|r| r.action.clone())
        .collect();
    
    tournament_data.add_hand(hand_actions);
    
    // Update game state for next hand
    current_game_state = calculate_next_hand_state(&sim.game_state);
}

tournament_data.set_results(final_results);
```

## Replaying Tournaments

### Creating Tournament Replay

```rust
use rs_poker::arena::{TournamentReplay, ReplayTournamentBuilder};

// Create tournament replay
let tournament_replay = ReplayTournamentBuilder::new()
    .with_tournament_data(tournament_data)
    .build_tournament_replay()
    .unwrap();
```

### Navigation Through Tournament

```rust
// Get current tournament state
let current_state = tournament_replay.get_current_tournament_state();
println!("Current stacks: {:?}", current_state.stacks);

// Step through hands one by one
while tournament_replay.has_more_hands() {
    if let Ok(Some(hand_replay)) = tournament_replay.step_to_next_hand() {
        // hand_replay is a GameReplay for this specific hand
        println!("Hand {} completed", tournament_replay.get_current_hand_index() - 1);
        println!("Updated stacks: {:?}", 
            tournament_replay.get_current_tournament_state().stacks);
        
        // You can also replay the hand step by step
        let mut hand_copy = hand_replay.clone();
        while hand_copy.has_more_actions() {
            let action = hand_copy.step_forward().unwrap();
            // Process each action in the hand
        }
    }
}
```

### Time Travel in Tournaments

```rust
// Jump to a specific hand
tournament_replay.step_to_hand(5).unwrap();
println!("State after hand 5: {:?}", 
    tournament_replay.get_current_tournament_state().stacks);

// Get a specific hand without changing current state
let hand_3_replay = tournament_replay.get_hand_replay(3).unwrap();

// Reset to tournament start
tournament_replay.reset_to_start();
```

### Analyzing Specific Hands

```rust
// Replay a specific hand with exact cards
let hand_index = 10;
let hand_replay = tournament_replay.get_hand_replay(hand_index).unwrap();

// Get the exact cards dealt in this hand
let mut cards_dealt = Vec::new();
let mut temp_replay = hand_replay.clone();

while temp_replay.has_more_actions() {
    if let Ok(Some(action)) = temp_replay.step_forward() {
        match action {
            Action::DealStartingHand(payload) => {
                println!("Player {} dealt {}", payload.idx, payload.card);
                cards_dealt.push((payload.idx, payload.card));
            }
            Action::DealCommunity(card) => {
                println!("Community card: {}", card);
                cards_dealt.push((999, card)); // 999 = community
            }
            _ => {}
        }
    }
}
```

## Async HTTP Server Integration

Perfect for serving tournament data asynchronously:

```rust
// Pseudo-code for async tournament API
async fn get_tournament_state_at_hand(
    tournament_id: String,
    hand_index: usize
) -> Result<GameState, ServerError> {
    // Load tournament data from database
    let tournament_data = load_tournament_data(&tournament_id).await?;
    
    // Create replay and navigate to desired hand
    let mut tournament_replay = ReplayTournamentBuilder::new()
        .with_tournament_data(tournament_data)
        .build_tournament_replay()?;
        
    tournament_replay.step_to_hand(hand_index)?;
    
    Ok(tournament_replay.get_current_tournament_state().clone())
}

async fn get_hand_details(
    tournament_id: String,
    hand_index: usize
) -> Result<HandDetails, ServerError> {
    let tournament_data = load_tournament_data(&tournament_id).await?;
    let tournament_replay = ReplayTournamentBuilder::new()
        .with_tournament_data(tournament_data)
        .build_tournament_replay()?;
        
    let hand_replay = tournament_replay.get_hand_replay(hand_index)?;
    
    // Extract hand details with exact cards
    let mut hand_details = HandDetails::new();
    let mut temp_replay = hand_replay.clone();
    
    while temp_replay.has_more_actions() {
        if let Ok(Some(action)) = temp_replay.step_forward() {
            hand_details.add_action(action);
        }
    }
    
    Ok(hand_details)
}

async fn bot_tournament_decision(
    tournament_id: String,
    hand_index: usize,
    action_index: usize
) -> Result<AgentAction, ServerError> {
    // Get exact game state at this point
    let tournament_data = load_tournament_data(&tournament_id).await?;
    let tournament_replay = ReplayTournamentBuilder::new()
        .with_tournament_data(tournament_data)
        .build_tournament_replay()?;
        
    let mut hand_replay = tournament_replay.get_hand_replay(hand_index)?;
    hand_replay.step_to(action_index)?;
    
    let current_state = hand_replay.get_current_state();
    
    // Bot makes decision based on exact game state
    let decision = bot_ai_decision(current_state).await?;
    
    Ok(decision)
}
```

## Storage and Serialization

### Storing Tournament Data

```rust
// Tournament data can be serialized for storage
#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

// Store in database
async fn save_tournament(
    tournament_id: &str,
    tournament_data: &TournamentReplayData
) -> Result<(), DatabaseError> {
    let serialized = serde_json::to_string(tournament_data)?;
    database.store(tournament_id, serialized).await?;
    Ok(())
}

// Load from database
async fn load_tournament(
    tournament_id: &str
) -> Result<TournamentReplayData, DatabaseError> {
    let serialized = database.load(tournament_id).await?;
    let tournament_data = serde_json::from_str(&serialized)?;
    Ok(tournament_data)
}
```

## Use Cases

### 1. Tournament Analysis Platform

```rust
// Web interface for tournament analysis
async fn tournament_viewer_api() {
    // Get tournament list
    app.get("/tournaments", list_tournaments);
    
    // Get tournament overview
    app.get("/tournaments/:id", get_tournament_overview);
    
    // Get specific hand
    app.get("/tournaments/:id/hands/:hand", get_hand_details);
    
    // Get state at specific point
    app.get("/tournaments/:id/hands/:hand/actions/:action", get_game_state);
}
```

### 2. Bot Training and Testing

```rust
// Train bots on historical tournament data
async fn train_bot_on_tournament(tournament_id: &str) {
    let tournament_data = load_tournament_data(tournament_id).await?;
    let tournament_replay = create_replay(tournament_data)?;
    
    // Train on every decision point
    for hand_index in 0..tournament_replay.get_replay_data().num_hands() {
        let hand_replay = tournament_replay.get_hand_replay(hand_index)?;
        
        // Train on every action in the hand
        while hand_replay.has_more_actions() {
            let state = hand_replay.get_current_state();
            let expected_action = get_next_action(&hand_replay);
            
            train_bot_on_state(state, expected_action).await?;
            hand_replay.step_forward()?;
        }
    }
}
```

### 3. Tournament Broadcasting

```rust
// Live tournament replay for spectators
async fn tournament_broadcast() {
    let tournament_replay = load_tournament_replay().await?;
    
    // Broadcast each hand with timing
    for hand_index in 0..tournament_replay.get_replay_data().num_hands() {
        broadcast_hand_start(hand_index).await;
        
        let hand_replay = tournament_replay.get_hand_replay(hand_index)?;
        
        // Broadcast each action with delays
        while hand_replay.has_more_actions() {
            let action = hand_replay.step_forward()?;
            broadcast_action(action).await;
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        
        broadcast_hand_end().await;
    }
}
```

## Benefits

### 1. **Complete Tournament History**
- Every hand recorded with exact cards
- Full tournament progression captured
- Player elimination tracking

### 2. **Flexible Navigation**
- Jump to any hand in the tournament
- Step through hands sequentially
- Reset and replay from any point

### 3. **Exact Card Reproduction**
- Same cards dealt in same order
- Perfect for analysis and debugging
- Enables deterministic bot testing

### 4. **Async-Ready Architecture**
- Non-blocking operations
- Perfect for web servers
- Scalable tournament analysis

### 5. **Storage Efficient**
- Only actions stored, not full states
- Compact representation
- Easy to serialize/deserialize

This tournament replay system provides everything needed for comprehensive tournament analysis, bot development, and async tournament services!

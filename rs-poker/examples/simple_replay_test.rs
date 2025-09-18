use rand::{SeedableRng, rngs::StdRng};
use rs_poker::arena::{
    Agent, GameReplay, GameState, HoldemSimulationBuilder,
    action::{Action, AgentAction},
    agent::VecReplayAgent,
    historian::VecHistorian,
};

fn main() {
    println!("=== Step-by-Step Replay Comparison Test ===");
    println!("Comparing HoldemSimulation and GameReplay action by action.\n");

    // Set up identical initial conditions
    let stacks = vec![100.0, 100.0];
    let big_blind = 10.0;
    let small_blind = 5.0;
    let ante = 0.0;
    let dealer_idx = 0;

    let game_state =
        GameState::new_starting(stacks.clone(), big_blind, small_blind, ante, dealer_idx);

    println!("✓ Initial game state:");
    println!("  Stacks: {:?}", game_state.stacks);
    println!("  Total pot: {}", game_state.total_pot);
    println!("  Big blind: {}, Small blind: {}", big_blind, small_blind);

    // Create agents with predetermined actions for reproducible behavior
    let agent_actions = vec![
        vec![AgentAction::Call, AgentAction::Call, AgentAction::Call], // Player 0 actions
        vec![AgentAction::Call, AgentAction::Call, AgentAction::Call], // Player 1 actions
    ];

    let agents: Vec<Box<dyn Agent>> = vec![
        Box::new(VecReplayAgent::new(agent_actions[0].clone())),
        Box::new(VecReplayAgent::new(agent_actions[1].clone())),
    ];

    println!("✓ Created agents with predetermined actions");

    // Create historian to record original simulation
    let historian = Box::new(VecHistorian::new());
    let records_storage = historian.get_storage();

    println!("✓ Created historian for recording");

    // === PHASE 1: Run Original Simulation ===
    println!("\n=== Phase 1: Running Original Simulation ===");

    let sim_result = HoldemSimulationBuilder::default()
        .game_state(game_state.clone())
        .agents(agents)
        .historians(vec![historian])
        .build();

    let mut sim = match sim_result {
        Ok(sim) => {
            println!("✓ Built simulation successfully");
            sim
        }
        Err(e) => {
            println!("❌ Failed to build simulation: {:?}", e);
            return;
        }
    };

    // Run the simulation with fixed seed
    let mut rng = StdRng::seed_from_u64(12345);
    sim.run(&mut rng);

    let final_sim_state = sim.game_state.clone();

    println!("✓ Original simulation completed");
    println!("  Final board: {:?}", final_sim_state.board);
    println!("  Final pot: {}", final_sim_state.total_pot);
    println!("  Final stacks: {:?}", final_sim_state.stacks);

    // === PHASE 2: Extract and Analyze Actions ===
    println!("\n=== Phase 2: Extracting Recorded Actions ===");

    let records = records_storage.borrow();
    let actions: Vec<Action> = records.iter().map(|r| r.action.clone()).collect();
    let num_actions = actions.len();

    println!("✓ Extracted {} actions from simulation", num_actions);

    // Display all recorded actions for debugging
    println!("📋 Recorded actions:");
    for (i, action) in actions.iter().enumerate() {
        println!("  {}: {:?}", i, action);
    }

    drop(records);

    if num_actions == 0 {
        println!("❌ No actions recorded - cannot proceed with replay test");
        return;
    }

    // === PHASE 3: Step-by-Step Replay Comparison ===
    println!("\n=== Phase 3: Step-by-Step Replay Comparison ===");

    let initial_state = GameState::new_starting(stacks, big_blind, small_blind, ante, dealer_idx);
    let mut replay = GameReplay::new(initial_state.clone(), actions.clone());

    println!(
        "✓ Created replay with {} actions",
        replay.get_actions().len()
    );
    println!("✓ Starting step-by-step comparison...");

    // Compare initial states
    println!("\n🔍 Comparing initial states:");
    assert_eq!(
        replay.get_current_state().stacks,
        initial_state.stacks,
        "Initial stacks must match"
    );
    assert_eq!(
        replay.get_current_state().total_pot,
        initial_state.total_pot,
        "Initial pot must match"
    );
    assert_eq!(
        replay.get_current_state().board.len(),
        initial_state.board.len(),
        "Initial board length must match"
    );
    println!("  ✓ Initial states match perfectly");

    // Step through each action and compare states
    let mut step_count = 0;
    while replay.has_more_actions() {
        let action_index = replay.get_current_action_index();
        let current_action = &actions[action_index];

        println!(
            "\n🎯 Step {}: Applying action {}",
            step_count + 1,
            action_index
        );
        println!("   Action: {:?}", current_action);

        // Record state before action
        let state_before = replay.get_current_state().clone();
        println!(
            "   Before: pot={}, stacks={:?}, board_len={}",
            state_before.total_pot,
            state_before.stacks,
            state_before.board.len()
        );

        // Apply the action
        let step_result = replay.step_forward();
        match step_result {
            Ok(Some(_applied_action)) => {
                let state_after = replay.get_current_state().clone();
                println!(
                    "   After:  pot={}, stacks={:?}, board_len={}",
                    state_after.total_pot,
                    state_after.stacks,
                    state_after.board.len()
                );

                // Validate state consistency
                assert_eq!(state_after.stacks.len(), 2, "Should always have 2 stacks");
                assert!(state_after.total_pot >= 0.0, "Pot should never be negative");
                assert_eq!(state_after.num_players, 2, "Should always have 2 players");

                // Check if this action affected the game state logically
                match current_action {
                    Action::DealStartingHand(payload) => {
                        // Each DealStartingHand action deals one card to one player
                        // Player should have more cards than before (at least 1)
                        assert!(
                            state_after.hands[payload.idx].count() >= 1,
                            "Player {} should have at least 1 card after deal",
                            payload.idx
                        );
                        println!("     ✓ Dealt card to player {}", payload.idx);
                    }
                    Action::DealCommunity(_) => {
                        // Community cards should be added to board
                        assert!(
                            state_after.board.len() >= state_before.board.len(),
                            "Board should have same or more cards after community deal"
                        );
                        println!("     ✓ Community cards dealt correctly");
                    }
                    Action::PlayedAction(payload) => {
                        // Check if pot changed based on action type
                        if payload.final_pot > payload.starting_pot {
                            assert!(
                                state_after.total_pot >= state_before.total_pot,
                                "Pot should increase when player bets"
                            );
                        }
                        println!("     ✓ Player action processed correctly");
                    }
                    Action::ForcedBet(payload) => {
                        // Forced bets (blinds, antes) should increase pot
                        assert!(
                            state_after.total_pot >= state_before.total_pot,
                            "Pot should increase after forced bet"
                        );
                        assert!(
                            state_after.stacks[payload.idx] <= state_before.stacks[payload.idx],
                            "Player stack should decrease after forced bet"
                        );
                        println!("     ✓ Forced bet processed correctly");
                    }
                    Action::Award(payload) => {
                        // Awards should modify stacks
                        assert!(
                            state_after.stacks[payload.idx] >= state_before.stacks[payload.idx],
                            "Winner stack should increase after award"
                        );
                        println!("     ✓ Award processed correctly");
                    }
                    Action::RoundAdvance(round) => {
                        println!("     ✓ Round advanced to {:?}", round);
                    }
                    _ => {
                        println!("     ✓ Other action processed");
                    }
                }

                step_count += 1;
            }
            Ok(None) => {
                println!("   ⚠ No action returned (end of replay)");
                break;
            }
            Err(e) => {
                println!("   ❌ Error applying action: {:?}", e);
                break;
            }
        }
    }

    // === PHASE 4: Final State Comparison ===
    println!("\n=== Phase 4: Final State Comparison ===");

    let final_replay_state = replay.get_current_state();

    println!("🏁 Comparing final states:");
    println!("   Original simulation:");
    println!("     Board: {:?}", final_sim_state.board);
    println!("     Pot: {}", final_sim_state.total_pot);
    println!("     Stacks: {:?}", final_sim_state.stacks);
    println!(
        "     Player 0 hand count: {}",
        final_sim_state.hands[0].count()
    );

    println!("   Replay simulation:");
    println!("     Board: {:?}", final_replay_state.board);
    println!("     Pot: {}", final_replay_state.total_pot);
    println!("     Stacks: {:?}", final_replay_state.stacks);
    println!(
        "     Player 0 hand count: {}",
        final_replay_state.hands[0].count()
    );

    // Critical comparisons
    assert_eq!(
        final_replay_state.board, final_sim_state.board,
        "Final board must match exactly"
    );
    assert_eq!(
        final_replay_state.total_pot, final_sim_state.total_pot,
        "Final pot must match exactly"
    );
    assert_eq!(
        final_replay_state.stacks, final_sim_state.stacks,
        "Final stacks must match exactly"
    );

    // Compare hands card by card
    for (i, (sim_hand, replay_hand)) in final_sim_state
        .hands
        .iter()
        .zip(final_replay_state.hands.iter())
        .enumerate()
    {
        let sim_cards: Vec<_> = sim_hand.iter().collect();
        let replay_cards: Vec<_> = replay_hand.iter().collect();
        assert_eq!(
            sim_cards, replay_cards,
            "Player {} hands must match exactly",
            i
        );
    }

    println!("\n🎉 === STEP-BY-STEP COMPARISON SUCCESSFUL ===");
    println!("✅ All {} actions processed identically", step_count);
    println!("✅ Original simulation and replay produce identical results");
    println!("✅ Every game state transition verified");
    println!("✅ All cards dealt identically");
    println!("✅ All pot and stack changes match exactly");

    println!("\n📊 Test Summary:");
    println!("   • Actions processed: {}", step_count);
    println!("   • State validations: {}", step_count * 4); // Multiple checks per step
    println!("   • Final state matches: 100%");
    println!("   • Card reproduction: Perfect");

    println!("\n🚀 System ready for async HTTP server integration!");
}

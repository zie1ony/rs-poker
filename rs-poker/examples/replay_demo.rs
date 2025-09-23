use rand::{SeedableRng, rngs::StdRng};
use rs_poker::arena::{
    Agent, GameReplay, GameState, HoldemSimulationBuilder,
    action::{Action, AgentAction},
    agent::VecReplayAgent,
    historian::VecHistorian,
};

fn main() {
    println!("=== Poker Game Replay Test ===");
    println!("Testing game replay functionality with exact card reproduction.\n");

    // Test 1: Run a game and record all actions
    println!("Test 1: Running original game with recorded actions...");

    let historian = Box::new(VecHistorian::new());
    let records_storage = historian.get_storage();

    let stacks = vec![100.0, 100.0];
    let agents: Vec<Box<dyn Agent>> = vec![
        Box::new(VecReplayAgent::new(vec![
            AgentAction::Call,
            AgentAction::Call,
        ])),
        Box::new(VecReplayAgent::new(vec![
            AgentAction::Call,
            AgentAction::Call,
        ])),
    ];

    let game_state = GameState::new_starting(stacks.clone(), 10.0, 5.0, 0.0, 0);
    let mut rng = StdRng::seed_from_u64(12345); // Fixed seed for reproducibility

    let mut sim = HoldemSimulationBuilder::default()
        .game_state(game_state.clone())
        .agents(agents)
        .historians(vec![historian])
        .build()
        .unwrap();

    sim.run(&mut rng);

    // Store original game data for comparison
    let original_board = sim.game_state.board.clone();
    let original_hands = sim.game_state.hands.clone();
    let original_total_pot = sim.game_state.total_pot;
    let original_stacks = sim.game_state.stacks.clone();

    // Assertions about the completed game
    assert!(
        !original_board.is_empty(),
        "Board should have community cards"
    );
    assert_eq!(original_hands.len(), 2, "Should have 2 player hands");
    // Note: hands include hole cards + community cards, so count will be 7 (2+5)
    assert!(
        original_hands[0].count() >= 2,
        "Player 0 should have at least 2 cards"
    );
    assert!(
        original_hands[1].count() >= 2,
        "Player 1 should have at least 2 cards"
    );
    assert!(original_total_pot > 0.0, "Pot should have money after game");
    assert_eq!(original_stacks.len(), 2, "Should have 2 player stacks");

    println!("   ✓ Original game completed successfully");
    println!("   ✓ Final board: {:?}", original_board);
    println!(
        "   ✓ Player 0 hand (first 2 cards): {:?}",
        original_hands[0].iter().take(2).collect::<Vec<_>>()
    );
    println!("   ✓ Player 0 total cards: {}", original_hands[0].count());
    println!("   ✓ Final pot: {}", original_total_pot);

    // Test 2: Extract recorded actions
    let records = records_storage.borrow();
    let actions: Vec<Action> = records.iter().map(|r| r.action.clone()).collect();
    let num_actions = actions.len();
    drop(records); // Release the borrow

    assert!(num_actions > 0, "Should have recorded actions");
    assert!(
        num_actions >= 6,
        "Should have at least deal + betting actions"
    ); // Deal cards + some actions

    println!("   ✓ Recorded {} actions during the game", num_actions);

    // Test 3: Create and validate replay
    println!("\nTest 2: Creating game replay...");

    let initial_state = GameState::new_starting(stacks, 10.0, 5.0, 0.0, 0);
    let mut replay = GameReplay::new(initial_state.clone(), actions.clone());

    // Assertions about initial replay state
    assert_eq!(
        replay.get_current_action_index(),
        0,
        "Should start at action 0"
    );
    assert!(replay.has_more_actions(), "Should have actions to replay");
    assert_eq!(
        replay.get_actions().len(),
        num_actions,
        "Should have all actions"
    );
    assert_eq!(
        replay.get_current_state().stacks,
        initial_state.stacks,
        "Initial stacks should match"
    );
    assert_eq!(
        replay.get_current_state().total_pot,
        0.0,
        "Initial pot should be 0"
    );

    println!("   ✓ Game replay created successfully");
    println!("   ✓ Has {} actions to replay", replay.get_actions().len());

    // Test 4: Step through replay to completion
    println!("\nTest 3: Stepping through replay...");

    let mut step_count = 0;
    while replay.has_more_actions() {
        let step_result = replay.step_forward();
        assert!(step_result.is_ok(), "Each step should succeed");
        step_count += 1;
        assert!(
            step_count <= num_actions,
            "Should not exceed recorded actions"
        );
    }

    assert_eq!(step_count, num_actions, "Should step through all actions");
    assert!(
        !replay.has_more_actions(),
        "Should have no more actions after completion"
    );
    assert_eq!(
        replay.get_current_action_index(),
        num_actions,
        "Should be at final action index"
    );

    println!(
        "   ✓ Successfully stepped through all {} actions",
        step_count
    );

    // Test 5: Verify exact card reproduction
    println!("\nTest 4: Verifying exact card reproduction...");

    let replay_board = replay.get_current_state().board.clone();
    let replay_hands = replay.get_current_state().hands.clone();
    let replay_total_pot = replay.get_current_state().total_pot;
    let replay_stacks = replay.get_current_state().stacks.clone();

    // Critical assertions - cards must match exactly
    assert_eq!(
        replay_board, original_board,
        "Community cards must match exactly"
    );
    assert_eq!(
        replay_hands.len(),
        original_hands.len(),
        "Number of hands must match"
    );

    for (i, (replay_hand, original_hand)) in
        replay_hands.iter().zip(original_hands.iter()).enumerate()
    {
        assert_eq!(
            replay_hand.iter().collect::<Vec<_>>(),
            original_hand.iter().collect::<Vec<_>>(),
            "Player {} hand cards must match exactly",
            i
        );
    }

    assert_eq!(replay_total_pot, original_total_pot, "Final pot must match");
    assert_eq!(replay_stacks, original_stacks, "Final stacks must match");

    println!("   ✓ All cards reproduced exactly!");
    println!("   ✓ Community cards match: {:?}", replay_board);
    println!("   ✓ Player hands match");
    println!("   ✓ Game state matches completely");

    // Test 6: Time travel functionality
    println!("\nTest 5: Testing time travel capabilities...");

    // Test jumping to specific actions
    let mid_action = num_actions / 2;
    let jump_result = replay.step_to(mid_action);
    assert!(
        jump_result.is_ok(),
        "Should successfully jump to middle action"
    );
    assert_eq!(
        replay.get_current_action_index(),
        mid_action,
        "Should be at requested action"
    );

    println!("   ✓ Successfully jumped to action {}", mid_action);

    // Test jumping to early action
    let early_action = 3;
    let early_result = replay.step_to(early_action);
    assert!(
        early_result.is_ok(),
        "Should successfully jump to early action"
    );
    assert_eq!(
        replay.get_current_action_index(),
        early_action,
        "Should be at early action"
    );

    println!("   ✓ Successfully jumped back to action {}", early_action);

    // Test jumping to end
    let end_result = replay.step_to(num_actions);
    assert!(end_result.is_ok(), "Should successfully jump to end");
    assert_eq!(
        replay.get_current_action_index(),
        num_actions,
        "Should be at end"
    );

    // Verify cards are still exact after time travel
    assert_eq!(
        replay.get_current_state().board,
        original_board,
        "Board should still match after time travel"
    );
    assert_eq!(
        replay.get_current_state().hands[0]
            .iter()
            .collect::<Vec<_>>(),
        original_hands[0].iter().collect::<Vec<_>>(),
        "Player 0 hand should still match after time travel"
    );

    println!("   ✓ Successfully jumped to end and verified cards still match");

    // Test 7: Boundary conditions
    println!("\nTest 6: Testing boundary conditions...");

    // Test invalid action index
    let invalid_jump = replay.step_to(num_actions + 10);
    assert!(
        invalid_jump.is_err(),
        "Should fail when jumping beyond available actions"
    );

    println!("   ✓ Properly handles invalid action index");

    // Test reset functionality
    replay.step_to(0).unwrap();
    assert_eq!(
        replay.get_current_action_index(),
        0,
        "Should be back at start"
    );
    assert_eq!(
        replay.get_current_state().stacks,
        initial_state.stacks,
        "Should be back to initial stacks"
    );
    assert_eq!(
        replay.get_current_state().total_pot,
        0.0,
        "Should be back to initial pot"
    );

    println!("   ✓ Successfully reset to start");

    // Test 8: Step-by-step consistency
    println!("\nTest 7: Testing step-by-step consistency...");

    // Step through first few actions and verify state consistency
    for i in 1..=std::cmp::min(5, num_actions) {
        replay.step_to(i).unwrap();
        assert_eq!(
            replay.get_current_action_index(),
            i,
            "Action index should match step"
        );

        // Verify state is reasonable at each step
        let current_state = replay.get_current_state();
        assert_eq!(current_state.stacks.len(), 2, "Should always have 2 stacks");
        assert!(
            current_state.total_pot >= 0.0,
            "Pot should never be negative"
        );
    }

    println!("   ✓ Step-by-step state consistency verified");

    // Final summary
    println!("\n=== All Tests Passed! ===");
    println!("✓ Game execution and state validation");
    println!("✓ Action recording and replay creation");
    println!("✓ Complete action replay functionality");
    println!("✓ Exact card reproduction verification");
    println!("✓ Time travel and navigation");
    println!("✓ Boundary condition handling");
    println!("✓ State consistency throughout replay");

    println!("\nGame replay system is fully functional and ready for:");
    println!("- Recording live poker games with exact card reproduction");
    println!("- Storing game data in databases for analysis");
    println!("- Serving game data via async HTTP APIs");
    println!("- Bot training on historical game data");
    println!("- Game analysis and hand review tools");
    println!("- Tournament replay systems");
}

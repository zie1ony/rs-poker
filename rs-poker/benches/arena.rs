use criterion::BenchmarkId;
use criterion::Criterion;

use criterion::criterion_group;
use criterion::criterion_main;
use rand::rng;
use rs_poker::arena::Agent;
use rs_poker::arena::GameState;
use rs_poker::arena::HoldemSimulationBuilder;
use rs_poker::arena::agent::RandomAgent;
use rs_poker::arena::agent::RandomPotControlAgent;

const STARTING_STACK: f32 = 100_000.0;
const ANTE: f32 = 50.0;
const SMALL_BLIND: f32 = 250.0;
const BIG_BLIND: f32 = 500.0;

const DEFAULT_FOLD: f64 = 0.15;
const DEFAULT_CALL: f64 = 0.5;

const RANDOM_CHANCES: [(f64, f64); 5] =
    [(0.0, 0.5), (0.15, 0.5), (0.5, 0.4), (0.0, 1.0), (0.0, 0.1)];

fn run_one_arena(num_players: usize, percent_fold: f64, percent_call: f64) -> GameState {
    let stacks = vec![STARTING_STACK; num_players];
    let game_state = GameState::new_starting(stacks, BIG_BLIND, SMALL_BLIND, ANTE, 0);
    let agents: Vec<Box<dyn Agent>> = (0..num_players)
        .map(|_| -> Box<dyn Agent> {
            Box::new(RandomAgent::new(vec![percent_fold], vec![percent_call]))
        })
        .collect();
    let mut sim = HoldemSimulationBuilder::default()
        .game_state(game_state)
        .agents(agents)
        .build()
        .unwrap();

    let mut rand = rng();

    sim.run(&mut rand);
    sim.game_state
}

fn run_one_pot_control_arena(num_players: usize) -> GameState {
    let stacks = vec![STARTING_STACK; num_players];
    let game_state = GameState::new_starting(stacks, BIG_BLIND, SMALL_BLIND, ANTE, 0);
    let agents: Vec<Box<dyn Agent>> = (0..num_players)
        .map(|_idx| -> Box<dyn Agent> { Box::new(RandomPotControlAgent::new(vec![0.3])) })
        .collect();

    let mut sim = HoldemSimulationBuilder::default()
        .game_state(game_state)
        .agents(agents)
        .build()
        .unwrap();

    let mut rand = rng();

    sim.run(&mut rand);
    sim.game_state
}

fn bench_num_random_agent_players(c: &mut Criterion) {
    let mut group = c.benchmark_group("arena_random_agents");
    for num_players in 2..9 {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_players),
            &num_players,
            |b, num_players| {
                b.iter(|| run_one_arena(*num_players, DEFAULT_FOLD, DEFAULT_CALL));
            },
        );
    }

    group.finish();
}

fn bench_random_chances_agents(c: &mut Criterion) {
    let mut group = c.benchmark_group("arena_random_agents");
    for input in RANDOM_CHANCES {
        let (percent_fold, percent_call) = input;
        let id = format!("percent_fold: {percent_fold} percent_call: {percent_call}");
        group.bench_with_input(
            BenchmarkId::new("arena_random_agent_choices", id),
            &input,
            |b, input| {
                let (percent_fold, percent_call) = input;
                b.iter(|| run_one_arena(6, *percent_fold, *percent_call));
            },
        );
    }

    group.finish();
}

fn bench_pot_control_agents(c: &mut Criterion) {
    let mut group = c.benchmark_group("pot_control_agents");

    for num_players in 2..9 {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_players),
            &num_players,
            |b, num_players| {
                b.iter(|| run_one_pot_control_arena(*num_players));
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_num_random_agent_players,
    bench_pot_control_agents,
    bench_random_chances_agents
);
criterion_main!(benches);

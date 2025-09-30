use rs_poker_engine::{game_instance::GameInstance, game_summary::GameSummary};
use rs_poker_types::{
    game::GameId,
    player::{AutomatType, Player, PlayerName},
};

fn main() {
    let num_of_players = 3;
    let initial_stack = 100.0;
    let small_blind = 5.0;
    let big_blind = 10.0;
    let game_id = GameId::random();

    let players: Vec<Player> = (1..=num_of_players)
        .map(|i| Player::Automat {
            name: PlayerName::new(&format!("Player{}", i)),
            automat_type: AutomatType::Random,
        })
        .collect();

    let mut game_instance = GameInstance::new_with_random_cards(
        game_id.clone(),
        None,
        players,
        vec![initial_stack; num_of_players],
        big_blind,
        small_blind,
    );

    game_instance.run();

    let summary = GameSummary::full(game_instance.simulation.events.clone()).summary();
    println!("{}", summary);
}

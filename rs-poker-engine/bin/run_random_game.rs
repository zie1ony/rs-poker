use rs_poker_engine::{game_instance::GameInstance, game_summary::GameSummary};
use rs_poker_types::{
    game::GameSettings,
    player::{AutomatType, Player, PlayerName},
};

fn main() {
    let num_of_players = 3;
    let initial_stack = 100.0;
    let small_blind = 5.0;

    let players: Vec<Player> = (1..=num_of_players)
        .map(|i| Player::Automat {
            name: PlayerName::new(&format!("Player{}", i)),
            automat_type: AutomatType::Random,
        })
        .collect();

    let game_settings = GameSettings {
        game_id: None,
        tournament_id: None,
        tournament_game_number: None,
        players: players.clone(),
        stacks: vec![initial_stack; num_of_players],
        small_blind,
        hands: None,
        community_cards: None,
        dealer_index: 0,
    };

    let mut game_instance = GameInstance::new(game_settings);

    game_instance.run();

    let summary = GameSummary::full(game_instance.simulation.events.clone()).summary();
    println!("{}", summary);
}

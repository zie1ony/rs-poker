use std::collections::HashMap;

use rs_poker_engine::{
    game_instance::GameInstance, tournament_instance::TournamentInstance,
    tournament_summary::TournamentSummary,
};
use rs_poker_llm_client::count_tokens;
use rs_poker_types::{
    game::GameId,
    player::{Player, PlayerName},
    tournament::{TournamentEndCondition, TournamentId, TournamentSettings},
};

fn main() {
    let players_n = 10;
    let players = (0..players_n)
        .map(|i| Player::random(&format!("Player{}", i)))
        .collect::<Vec<_>>();
    // Step 1: Create a tournament.
    let settings = TournamentSettings {
        tournament_id: TournamentId::random(),
        players,
        starting_player_stack: 100.0,
        starting_small_blind: 5.0,
        double_blinds_every_n_games: Some(3),
        end_condition: TournamentEndCondition::SingleWinner,
        see_historical_thoughts: false,
        public_chat: false,
    };

    let mut tournament = TournamentInstance::new(&settings);
    let mut games: HashMap<GameId, GameInstance> = HashMap::new();

    while !tournament.is_completed() {
        let game_settings = tournament.start_next_game().unwrap();
        let mut game_instance = GameInstance::new(game_settings);
        game_instance.run();
        let game_result = game_instance.game_final_results().unwrap();
        games.insert(game_instance.game_id.clone(), game_instance);
        tournament.finish_game(&game_result).unwrap();
    }

    let summary = TournamentSummary::for_player(
        tournament.events.clone(),
        games
            .into_iter()
            .map(|(id, gi)| (id, gi.simulation.events.clone()))
            .collect(),
        PlayerName::new("Player1"),
    );
    let summary = summary.summary();
    println!("{}", summary);
    println!("Tokens: {}", count_tokens(&summary));
}

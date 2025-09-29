use rs_poker_server::poker_client::PokerClient;
use rs_poker_types::{series::SeriesSettings, tournament::TournamentSettings};

pub async fn run_series(client: &PokerClient, series: SeriesSettings) {
    let tournaments = series.generate_all_tournaments();

    for tournament in tournaments {
        run_tournament(client, tournament).await;
    }
}

pub async fn run_tournament(client: &PokerClient, tournament: TournamentSettings) {
    // Check if the tournament is already created.
    match client.tournament_info(&tournament.tournament_id).await {
        Ok(info) => {
            println!("Tournament already exists: {:?}", info);
        }
        Err(_) => {
            // Create the tournament if it doesn't exist.
            println!("Creating tournament: {:?}", tournament.tournament_id);
            match client.new_tournament(tournament.clone()).await {
                Ok(res) => println!("Tournament created with ID: {:?}", res.tournament_id),
                Err(err) => {
                    println!("Failed to create tournament: {:?}", err);
                    return;
                }
            }
        }
    }
}
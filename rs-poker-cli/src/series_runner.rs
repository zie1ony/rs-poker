use std::collections::HashMap;

use rs_poker_server::{error::ServerError, poker_client::{PokerClient, PokerClientError}};
use rs_poker_types::{player::PlayerName, series::SeriesSettings, tournament::{TournamentId, TournamentSettings}};

pub async fn run_series(client: &PokerClient, series: SeriesSettings) {
    println!("Running series: {:?}", &series.series_id);
    let tournaments = series.generate_all_tournaments();

    println!("Spawning {} tournaments...", tournaments.len());
    for tournament in &tournaments {
        run_tournament(client, tournament).await;
    }

    // Load info for all tournaments in the series.
    println!("Fetching tournaments infos...");
    let mut infos = vec![];
    for tournament in &tournaments {
        match client.tournament_info(&tournament.tournament_id).await {
            Ok(info) => infos.push(info),
            Err(err) => {
                panic!(
                    "Failed to fetch info for tournament {:?}: {:?}",
                    &tournament.tournament_id, err
                );
            }
        }
    }

    // Check if all tournaments are complete.
    let mut completed_ids: Vec<TournamentId> = vec![];
    let mut in_progress_ids: Vec<TournamentId> = vec![];
    for info in &infos {
        if info.status.is_completed() {
            completed_ids.push(info.tournament_id());
        } else {
            in_progress_ids.push(info.tournament_id());
        }
    }

    let series_complete = in_progress_ids.is_empty();

    if series_complete {
        let mut winners: HashMap<PlayerName, usize> = HashMap::new();
        for info in &infos {
            let winner = info.winner.clone().unwrap();
            *winners.entry(winner).or_insert(0) += 1;
        }
        // Sort winners by number of wins.
        let mut winners: Vec<(PlayerName, usize)> = winners.into_iter().collect();
        winners.sort_by(|a, b| b.1.cmp(&a.1));
        println!("Series complete! Winners:");
        for (player, wins) in winners {
            println!(" - {:?} with {} wins!", player, wins);
        }
    } else {
        for id in completed_ids {
            println!(" - Tournament {:?} is complete.", id);
        }
        for id in in_progress_ids {
            println!(" - Tournament {:?} is still in progress.", id);
        }
    }

}

pub async fn run_tournament(client: &PokerClient, tournament: &TournamentSettings) {
    // Check if the tournament is already created.
    let tournament_id = &tournament.tournament_id;
    match client.tournament_info(tournament_id).await {
        Ok(_) => {
            // println!("[x] Tournament already exists: {:?}", tournament_id);
        }
        Err(PokerClientError::ServerError(ServerError::TournamentNotFound(_))) => {
            println!("[x] Creating tournament: {:?}", tournament.tournament_id);
            match client.new_tournament(tournament.clone()).await {
                Ok(res) => println!("-> Tournament created: {:?}", res.tournament_id),
                Err(err) => {
                    panic!("Failed to create tournament: {:?}", err);
                }
            }
        }
        Err(err) => {
            panic!("Failed to check tournament existence: {:?}", err);
        }
    }
}

use rand::{SeedableRng, seq::SliceRandom};

use crate::{
    player::Player,
    tournament::{TournamentEndCondition, TournamentId, TournamentSettings},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SeriesId(String);

impl SeriesId {
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

pub struct SeriesSettings {
    pub series_id: SeriesId,
    pub players: Vec<Player>,
    pub number_of_tournaments: usize,
    pub starting_player_stack: f32,
    pub starting_small_blind: f32,
    pub double_blinds_every_n_games: Option<usize>,
    pub end_condition: TournamentEndCondition,
    pub see_historical_thoughts: bool,
    pub public_chat: bool,
    pub random_seed: u64,
}

impl SeriesSettings {
    pub fn generate_all_tournaments(&self) -> Vec<TournamentSettings> {
        let mut rng = rand::rngs::StdRng::seed_from_u64(self.random_seed);
        let n = self.number_of_tournaments;
        let players_permutations: Vec<Vec<Player>> = (0..n)
            .map(|_| {
                let mut players = self.players.clone();
                players.shuffle(&mut rng);
                players
            })
            .collect();

        let mut tournaments = Vec::new();
        for i in 0..n {
            tournaments.push(TournamentSettings {
                tournament_id: TournamentId::new(&format!(
                    "{}-tournament-{}",
                    self.series_id.as_str(),
                    i + 1
                )),
                players: players_permutations[i].clone(),
                starting_player_stack: self.starting_player_stack,
                starting_small_blind: self.starting_small_blind,
                double_blinds_every_n_games: self.double_blinds_every_n_games,
                end_condition: self.end_condition.clone(),
                see_historical_thoughts: self.see_historical_thoughts,
                public_chat: self.public_chat,
            });
        }
        tournaments
    }
}

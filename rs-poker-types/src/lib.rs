use rand::Rng;

pub mod game;
pub mod game_event;
pub mod player;
pub mod series;
pub mod tournament;
pub mod tournament_event;

pub fn random_id(prefix: &str) -> String {
    format!("{}-{}", prefix, random_string(6))
}

pub fn random_string(len: usize) -> String {
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();

    (0..len)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

pub mod game_instance;
pub mod game_simulation;
pub mod summary;

pub fn random_game_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

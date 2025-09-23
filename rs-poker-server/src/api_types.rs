use axum::Json;
use reqwest::Method;
use rs_poker::{core::Card};
use rs_poker_types::{
    game::{Decision, GameFullView, GameId, GameInfo, GamePlayerView, GameStatus},
    player::{Player, PlayerName},
};

use crate::error::ServerError;

pub trait ServerRequest {
    type Response: serde::de::DeserializeOwned;

    fn path(&self) -> String;
    fn method(&self) -> Method;
}

pub type ServerResponse<T> = Json<Result<T, ServerError>>;

// --- health_check ---

#[derive(serde::Serialize, serde::Deserialize)]
pub struct HealthCheckRequest {
    pub id: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct HealthCheckResponse {
    pub id: String,
    pub status: String,
}

impl ServerRequest for HealthCheckRequest {
    type Response = HealthCheckResponse;

    fn path(&self) -> String {
        "/health_check".to_string()
    }

    fn method(&self) -> Method {
        Method::GET
    }
}

// --- as_json ---

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AsJsonRequest {
    pub payload: serde_json::Value,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct AsJsonResponse {
    pub data: serde_json::Value,
}

impl ServerRequest for AsJsonRequest {
    type Response = AsJsonResponse;

    fn path(&self) -> String {
        "/as_json".to_string()
    }

    fn method(&self) -> Method {
        Method::POST
    }
}

// --- new-game ---

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
pub struct NewGameRequest {
    pub game_id: GameId,
    pub players: Vec<Player>,
    pub small_blind: f32,
    pub initial_stacks: Vec<f32>,
    pub predefined_hands: Option<Vec<(Card, Card)>>,
    pub predefined_board: Option<Vec<Card>>,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
pub struct GameCreatedResponse {
    pub game_id: GameId,
}

impl ServerRequest for NewGameRequest {
    type Response = GameCreatedResponse;

    fn path(&self) -> String {
        "/new_game".to_string()
    }

    fn method(&self) -> Method {
        Method::POST
    }
}

// --- list games ---

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
pub struct ListGamesRequest {
    pub active_only: bool,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
pub struct ListGamesResponse {
    pub game_ids: Vec<(String, GameStatus)>,
}

impl ServerRequest for ListGamesRequest {
    type Response = ListGamesResponse;

    fn path(&self) -> String {
        "/list_games".to_string()
    }

    fn method(&self) -> Method {
        Method::GET
    }
}

// --- game full view ---

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
pub struct GameFullViewRequest {
    pub game_id: GameId,
    pub debug: bool,
}

impl ServerRequest for GameFullViewRequest {
    type Response = GameFullView;

    fn path(&self) -> String {
        "/game_full_view".to_string()
    }

    fn method(&self) -> Method {
        Method::GET
    }
}

// --- game player view ---

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
pub struct GamePlayerViewRequest {
    pub game_id: GameId,
    pub player_name: PlayerName,
}

impl ServerRequest for GamePlayerViewRequest {
    type Response = GamePlayerView;

    fn path(&self) -> String {
        "/game_player_view".to_string()
    }

    fn method(&self) -> Method {
        Method::GET
    }
}

// --- game info ---

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
pub struct GameInfoRequest {
    pub game_id: GameId,
}

impl ServerRequest for GameInfoRequest {
    type Response = GameInfo;

    fn path(&self) -> String {
        "/game_info".to_string()
    }

    fn method(&self) -> Method {
        Method::GET
    }
}

// --- make action ---

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct MakeActionRequest {
    pub game_id: GameId,
    pub decision: Decision,
}

impl ServerRequest for MakeActionRequest {
    type Response = GameCreatedResponse;

    fn path(&self) -> String {
        "/make_action".to_string()
    }

    fn method(&self) -> Method {
        Method::POST
    }
}

use axum::Json;
use reqwest::Method;
use rs_poker_types::{
    game::{Decision, GameFullView, GameId, GameInfo, GamePlayerView, GameStatus},
    player::{Player, PlayerName},
    tournament::{TournamentId, TournamentSettings},
};

use crate::{error::ServerError, handler::game_new::GameCreatedResponse};

pub trait ServerRequest {
    type Response: serde::de::DeserializeOwned;

    fn path(&self) -> String;
    fn method(&self) -> Method;
}

pub type ServerResponse<T> = Json<Result<T, ServerError>>;

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

// --- new tournament ---

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
pub struct NewTournamentRequest {
    pub settings: TournamentSettings,
}

impl ServerRequest for NewTournamentRequest {
    type Response = TournamentCreatedResponse;

    fn path(&self) -> String {
        "/new_tournament".to_string()
    }

    fn method(&self) -> Method {
        Method::POST
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
pub struct TournamentCreatedResponse {
    pub tournament_id: TournamentId,
}

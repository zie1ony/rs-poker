use axum::{body::Body, extract::Request, Router};
use http_body_util::BodyExt;
use rs_poker_types::{
    game::{GameFullView, GameId, GameInfo, GamePlayerView},
    tournament::{TournamentId, TournamentInfo, TournamentSettings},
};
use tower::ServiceExt;

use crate::{
    error::ServerError,
    handler::{
        game_full_view::{GameFullViewHandler, GameFullViewRequest},
        game_info::{GameInfoHandler, GameInfoRequest},
        game_list::{ListGamesHandler, ListGamesRequest, ListGamesResponse},
        game_make_action::{MakeActionHandler, MakeActionRequest},
        game_new::{GameCreatedResponse, NewGameHandler, NewGameRequest},
        game_player_view::{GamePlayerViewHandler, GamePlayerViewRequest},
        health_check::{HealthCheckHandler, HealthCheckRequest, HealthCheckResponse},
        tournament_info::{TournamentInfoHandler, TournamentInfoRequest},
        tournament_list::{ListTournamentsHandler, ListTournamentsRequest, ListTournamentsResponse},
        tournament_new::{NewTournamentHandler, TournamentCreatedResponse},
        Handler,
    },
};

#[derive(Debug)]
pub enum PokerClientError {
    RequestError(String),
    JsonParseError(String),
    HttpError(String),
    ServerError(ServerError),
}

impl std::fmt::Display for PokerClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PokerClientError::RequestError(msg) => write!(f, "Request error: {}", msg),
            PokerClientError::JsonParseError(msg) => write!(f, "JSON parse error: {}", msg),
            PokerClientError::HttpError(msg) => write!(f, "HTTP error: {}", msg),
            PokerClientError::ServerError(err) => write!(f, "Server error: {}", err),
        }
    }
}

impl std::error::Error for PokerClientError {}

type ClientResult<T> = Result<T, PokerClientError>;

pub enum PokerClient {
    /// For testing - uses the axum app directly
    Test(Router),
    /// For production - would use HTTP client (placeholder for now)
    Http { base_url: String },
}

impl PokerClient {
    /// Create a new test client that uses the axum app directly
    pub fn new_test(app: Router) -> Self {
        PokerClient::Test(app)
    }

    /// Create a new HTTP client for production use
    pub fn new_http(base_url: &str) -> Self {
        PokerClient::Http {
            base_url: base_url.to_string(),
        }
    }

    pub async fn query<T: Handler>(&self, request: T::Request) -> ClientResult<T::Response> {
        match self {
            PokerClient::Test(router) => make_test_query::<T>(&router, request).await,
            PokerClient::Http { base_url } => make_http_query::<T>(base_url, request).await,
        }
    }

    // Health check.

    pub async fn health_check(
        &self,
        request: HealthCheckRequest,
    ) -> ClientResult<HealthCheckResponse> {
        self.query::<HealthCheckHandler>(request).await
    }

    // Game.

    pub async fn new_game(&self, new_game: NewGameRequest) -> ClientResult<GameCreatedResponse> {
        self.query::<NewGameHandler>(new_game).await
    }

    pub async fn list_games(&self, params: ListGamesRequest) -> ClientResult<ListGamesResponse> {
        self.query::<ListGamesHandler>(params).await
    }

    pub async fn game_full_view(&self, params: GameFullViewRequest) -> ClientResult<GameFullView> {
        self.query::<GameFullViewHandler>(params).await
    }

    pub async fn game_player_view(
        &self,
        params: GamePlayerViewRequest,
    ) -> ClientResult<GamePlayerView> {
        self.query::<GamePlayerViewHandler>(params).await
    }

    pub async fn game_info(&self, game_id: &GameId) -> ClientResult<GameInfo> {
        let request = GameInfoRequest {
            game_id: game_id.clone(),
        };
        self.query::<GameInfoHandler>(request).await
    }

    pub async fn make_decision(&self, decision: MakeActionRequest) -> ClientResult<GameInfo> {
        self.query::<MakeActionHandler>(decision).await
    }

    // Tournament.

    pub async fn new_tournament(
        &self,
        settings: TournamentSettings,
    ) -> ClientResult<TournamentCreatedResponse> {
        self.query::<NewTournamentHandler>(settings).await
    }

    pub async fn list_tournaments(&self, params: ListTournamentsRequest) -> ClientResult<ListTournamentsResponse> {
        self.query::<ListTournamentsHandler>(params).await
    }

    pub async fn tournament_info(
        &self,
        tournament_id: &TournamentId,
    ) -> ClientResult<TournamentInfo> {
        let request = TournamentInfoRequest {
            tournament_id: tournament_id.clone(),
        };
        self.query::<TournamentInfoHandler>(request).await
    }
}

async fn make_http_query<T: Handler>(
    base_url: &str,
    request: T::Request,
) -> ClientResult<T::Response> {
    let client = reqwest::Client::new();
    let url = format!("{}{}", base_url, T::path());

    let response = match T::method() {
        reqwest::Method::GET => {
            // For GET requests, serialize request as query parameters
            let query_string = serde_urlencoded::to_string(&request)
                .map_err(|e| PokerClientError::JsonParseError(e.to_string()))?;
            let full_url = if query_string.is_empty() {
                url
            } else {
                format!("{}?{}", url, query_string)
            };
            client.get(&full_url).send().await
        }
        reqwest::Method::POST => {
            // For POST requests, send JSON body
            client.post(&url).json(&request).send().await
        }
        _ => {
            return Err(PokerClientError::HttpError(
                "Unsupported HTTP method".to_string(),
            ));
        }
    }
    .map_err(|e| PokerClientError::HttpError(e.to_string()))?;

    if !response.status().is_success() {
        return Err(PokerClientError::HttpError(format!(
            "HTTP error: {}",
            response.status()
        )));
    }

    response
        .json::<Result<T::Response, ServerError>>()
        .await
        .map_err(|e| PokerClientError::HttpError(e.to_string()))?
        .map_err(PokerClientError::ServerError)
}

async fn make_test_query<T: Handler>(
    router: &Router,
    request: T::Request,
) -> ClientResult<T::Response> {
    let path = T::path();

    let axum_request = match T::method() {
        reqwest::Method::GET => {
            // For GET requests, serialize request as query parameters
            let query_string = serde_urlencoded::to_string(&request)
                .map_err(|e| PokerClientError::JsonParseError(e.to_string()))?;
            let full_path = if query_string.is_empty() {
                path.to_string()
            } else {
                format!("{}?{}", path, query_string)
            };
            Request::builder()
                .uri(full_path)
                .body(Body::empty())
                .map_err(|e| PokerClientError::RequestError(e.to_string()))?
        }
        reqwest::Method::POST => {
            // For POST requests, send JSON body
            let body = serde_json::to_string(&request)
                .map_err(|e| PokerClientError::JsonParseError(e.to_string()))?;
            Request::builder()
                .uri(path)
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .map_err(|e| PokerClientError::RequestError(e.to_string()))?
        }
        _ => {
            return Err(PokerClientError::RequestError(
                "Unsupported HTTP method".to_string(),
            ));
        }
    };

    let response = router
        .clone()
        .oneshot(axum_request)
        .await
        .map_err(|e| PokerClientError::RequestError(e.to_string()))?;

    let body = response
        .into_body()
        .collect()
        .await
        .map_err(|e| PokerClientError::RequestError(e.to_string()))?
        .to_bytes();

    let body_str = String::from_utf8(body.to_vec())
        .map_err(|e| PokerClientError::JsonParseError(e.to_string()))?;

    let server_result: Result<T::Response, ServerError> = serde_json::from_str(&body_str)
        .map_err(|e| PokerClientError::JsonParseError(e.to_string()))?;

    server_result.map_err(PokerClientError::ServerError)
}

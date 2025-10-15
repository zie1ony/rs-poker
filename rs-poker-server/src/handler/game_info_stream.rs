use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::Response;
use axum::routing::MethodRouter;
use futures_util::StreamExt;
use rs_poker_types::game::{GameId, GameInfo};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{broadcast, RwLock};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use crate::poker_client::{ClientResult, PokerClient, PokerClientError, WsStream};
use crate::{
    handler::{game_info::GameInfoRequest, Handler},
    poker_server::ServerState,
};

pub struct GameInfoStreamHandler;

impl Handler for GameInfoStreamHandler {
    type Request = GameInfoRequest;
    type Response = GameInfo;

    fn router() -> MethodRouter<ServerState> {
        axum::routing::any(game_info_stream_handler)
    }

    fn method() -> reqwest::Method {
        reqwest::Method::GET
    }

    fn path() -> &'static str {
        "/game/stream"
    }
}

async fn game_info_stream_handler(
    ws: WebSocketUpgrade,
    State(state): State<ServerState>,
    Query(params): Query<GameInfoRequest>,
) -> Response {
    let game_id = params.game_id.clone();

    // Check if game exists and get its current status
    let game_info = {
        let engine = state.engine.lock().unwrap();
        engine.game_info(&game_id).ok()
    };

    ws.on_upgrade(move |socket| web_socket_handler(socket, game_id, game_info, state))
}

async fn web_socket_handler(
    mut socket: WebSocket,
    game_id: GameId,
    game_info: Option<GameInfo>,
    state: ServerState,
) {
    // Check if game exists
    let Some(initial_info) = game_info else {
        let _ = socket
            .send(axum::extract::ws::Message::Close(Some(
                axum::extract::ws::CloseFrame {
                    code: axum::extract::ws::close_code::UNSUPPORTED,
                    reason: "Game not found".into(),
                },
            )))
            .await;
        return;
    };

    // Send initial game info
    if let Ok(json) = serde_json::to_string(&initial_info) {
        if socket
            .send(axum::extract::ws::Message::Text(json.into()))
            .await
            .is_err()
        {
            return;
        }
    }

    // Check if game is already finished - send info then close
    if initial_info.is_finished() {
        let _ = socket
            .send(axum::extract::ws::Message::Close(Some(
                axum::extract::ws::CloseFrame {
                    code: axum::extract::ws::close_code::NORMAL,
                    reason: "Game already finished".into(),
                },
            )))
            .await;
        return;
    }

    // Subscribe to game updates
    let mut receiver = state.game_subscribers.subscribe(&game_id).await;

    // Handle broadcast updates
    loop {
        match receiver.recv().await {
            Ok(game_info_json) => {
                // Send the update to client
                if socket
                    .send(axum::extract::ws::Message::Text(game_info_json.clone().into()))
                    .await
                    .is_err()
                {
                    break;
                }

                // Check if game is finished and close connection
                if let Ok(game_info) = serde_json::from_str::<GameInfo>(&game_info_json) {
                    if game_info.is_finished() {
                        let _ = socket
                            .send(axum::extract::ws::Message::Close(Some(
                                axum::extract::ws::CloseFrame {
                                    code: axum::extract::ws::close_code::NORMAL,
                                    reason: "Game finished".into(),
                                },
                            )))
                            .await;
                        break;
                    }
                }
            }
            Err(broadcast::error::RecvError::Closed) => {
                break;
            }
            Err(broadcast::error::RecvError::Lagged(_)) => {
                // Client is too slow, skip this update
                continue;
            }
        }
    }

    // Cleanup empty channels when client disconnects
    state
        .game_subscribers
        .cleanup_empty_channels(&game_id)
        .await;
}

pub struct GameInfoStream {
    pub stream: WsStream,
}

impl GameInfoStream {
    pub async fn next(&mut self) -> Option<Result<GameInfo, PokerClientError>> {
        let msg = self.stream.next().await;
        match msg {
            // If we get a text message, try to parse it as GameInfo.
            Some(Ok(Message::Text(text))) => match serde_json::from_str::<GameInfo>(&text) {
                Ok(info) => Some(Ok(info)),
                Err(e) => Some(Err(PokerClientError::JsonParseError(e.to_string()))),
            },
            // Handle close frames gracefully by ending the stream
            Some(Ok(Message::Close(_))) => None,
            // Handle other message types as errors
            Some(Ok(msg)) => Some(Err(PokerClientError::RequestError(format!(
                "unexpedted WebSocket message: {:?}",
                msg
            )))),
            Some(Err(e)) => Some(Err(PokerClientError::RequestError(e.to_string()))),
            None => None,
        }
    }
}

impl PokerClient {
    /// Creates a WebSocket connection to the game info stream
    /// Returns the receiver stream for incoming messages
    pub async fn game_info_stream(&self, game_id: &GameId) -> ClientResult<GameInfoStream> {
        match self {
            PokerClient::Test(_) => Err(PokerClientError::RequestError(
                "WebSocket connections not supported in test mode".to_string(),
            )),
            PokerClient::Http { base_url } => {
                let ws_url = format!(
                    "{}/game/stream?game_id={}",
                    base_url
                        .replace("http://", "ws://")
                        .replace("https://", "wss://"),
                    game_id
                );

                let (ws_stream, _) = connect_async(&ws_url)
                    .await
                    .map_err(|e| PokerClientError::RequestError(e.to_string()))?;

                let (_write, read) = ws_stream.split();
                Ok(GameInfoStream { stream: read })
            }
        }
    }
}

#[derive(Clone)]
pub struct GameInfoStreamSubscribers {
    // Map of game_id to broadcast sender for that game
    subscribers: Arc<RwLock<HashMap<String, broadcast::Sender<String>>>>,
}

impl GameInfoStreamSubscribers {
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn broadcast_game_info(&self, game_id: &GameId, game_info: &GameInfo) {
        let subscribers = self.subscribers.read().await;
        if let Some(sender) = subscribers.get(&game_id.to_string()) {
            if let Ok(json) = serde_json::to_string(game_info) {
                // Send to all subscribers, ignore failures (disconnected clients)
                let _ = sender.send(json);
            }
        }
    }

    pub async fn subscribe(&self, game_id: &GameId) -> broadcast::Receiver<String> {
        let mut subscribers = self.subscribers.write().await;
        let sender = subscribers.entry(game_id.to_string()).or_insert_with(|| {
            let (tx, _) = broadcast::channel(100);
            tx
        });
        sender.subscribe()
    }

    pub async fn cleanup_empty_channels(&self, game_id: &GameId) {
        let mut subscribers = self.subscribers.write().await;
        if let Some(sender) = subscribers.get(&game_id.to_string()) {
            if sender.receiver_count() == 0 {
                subscribers.remove(&game_id.to_string());
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::poker_server::app_no_storage;

    use super::*;

    #[tokio::test]
    async fn test_healthcheck() {
        let app = app_no_storage();
        let client = PokerClient::new_test(app);
        let resp = client.health_check("test_id").await.unwrap();
        assert_eq!(resp.id, "test_id");
    }
}

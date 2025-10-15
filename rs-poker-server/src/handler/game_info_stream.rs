use axum::routing::MethodRouter;
use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use axum::response::Response;
use futures_util::{SinkExt, StreamExt};
use rs_poker_types::game::{GameId, GameInfo};
use axum::{
    extract::{Query, State},
};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async};

use crate::poker_client::{ClientResult, PokerClient, PokerClientError, WsStream};
use crate::{handler::{game_info::GameInfoRequest, Handler}, poker_server::ServerState};


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
    State(_state): State<ServerState>,
    Query(_params): Query<GameInfoRequest>,
) -> Response {
    ws.on_upgrade(web_socket_handler)
}

async fn web_socket_handler(
    mut socket: WebSocket
) {
    let message = "Hello from the game info stream!";
    socket.send(message.into()).await.unwrap();
    socket.close().await.unwrap();
}

pub struct GameInfoStream {
    pub stream: WsStream,
}

impl GameInfoStream {
    pub async fn next(&mut self) -> Option<Result<GameInfo, PokerClientError>> {
        let msg = self.stream.next().await;
        match msg {
            // If we get a text message, try to parse it as GameInfo.
            Some(Ok(Message::Text(text))) => {
                match serde_json::from_str::<GameInfo>(&text) {
                    Ok(info) => Some(Ok(info)),
                    Err(e) => Some(Err(PokerClientError::JsonParseError(e.to_string()))),
                }
            }
            Some(Ok(msg)) => Some(Err(PokerClientError::RequestError(format!("unexpedted WebSocket message: {:?}", msg)))),
            Some(Err(e)) => Some(Err(PokerClientError::RequestError(e.to_string()))),
            None => None,
        }
    }
}

impl PokerClient {
        /// Creates a WebSocket connection to the game info stream
    /// Returns the receiver stream for incoming messages
    pub async fn game_info_stream(
        &self,
        game_id: &GameId,
    ) -> ClientResult<GameInfoStream> {
        match self {
            PokerClient::Test(_) => {
                Err(PokerClientError::RequestError(
                    "WebSocket connections not supported in test mode".to_string(),
                ))
            }
            PokerClient::Http { base_url } => {
                let ws_url = format!("{}/game/stream?game_id={}", 
                    base_url.replace("http://", "ws://").replace("https://", "wss://"), 
                    game_id
                );
                
                let (ws_stream, _) = connect_async(&ws_url).await
                    .map_err(|e| PokerClientError::RequestError(e.to_string()))?;
                
                let (_write, read) = ws_stream.split();
                Ok(GameInfoStream { stream: read })
            }
        }
    }
}

pub struct GameInfoStreamSubscribers {
}

impl GameInfoStreamSubscribers {
    pub async broadcast_game_info(&self, game_info: &GameInfo) {
        // For each subscriber, send the game_info as a JSON text message
        // If sending fails, remove the subscriber from the list
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
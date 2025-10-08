use axum::{extract::Query, Json};

use crate::{
    define_handler,
    handler::HandlerResponse,
    poker_client::{ClientResult, PokerClient},
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct HealthCheckRequest {
    pub id: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct HealthCheckResponse {
    pub id: String,
    pub status: String,
}

pub async fn handler(
    Query(params): Query<HealthCheckRequest>,
) -> HandlerResponse<HealthCheckResponse> {
    Json(Ok(HealthCheckResponse {
        id: params.id,
        status: "ok".to_string(),
    }))
}

define_handler!(
    HealthCheckHandler {
        Request = HealthCheckRequest;
        Response = HealthCheckResponse;
        Method = GET;
        Path = "/health_check";
        FN = handler;
    }
);

impl PokerClient {
    pub async fn health_check(
        &self,
        request: HealthCheckRequest,
    ) -> ClientResult<HealthCheckResponse> {
        self.query::<HealthCheckHandler>(request).await
    }
}

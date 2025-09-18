use axum::{routing::{get, post}, Json, Router};


/// Having a function that produces our app makes it easy to call it from tests
/// without having to create an HTTP server.
pub fn app() -> Router {
    Router::new()
        .route("/ping", get(|| async { "pong" }))
        .route(
            "/as_json",
            post(|payload: Json<serde_json::Value>| async move {
                Json(serde_json::json!({ "data": payload.0 }))
            }),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    use axum::{body::Body, extract::Request, http::StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt; 

    #[tokio::test]
    async fn ping() {
        let app = app();

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(Request::builder().uri("/ping").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"pong");
    }
    
}
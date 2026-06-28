use serde::{Serialize, Deserialize};
use axum::{Router, routing, response::Json};
use crate::web::error::Result;
use std::sync::Arc;
use crate::state::State;

pub fn routes(state: Arc<State>) -> Router {
    Router::new()
        .route("/health", routing::get(health_handler))
        .with_state(state)
}

async fn health_handler() -> Result<Json<HealthResponse>> {
    let payload = HealthResponse {
        status: "OK".to_string(),
    };

    Ok(Json(payload))
}

#[derive(Serialize, Deserialize)]
struct HealthResponse {
    status: String,
}
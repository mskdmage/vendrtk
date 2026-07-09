use std::sync::Arc;

use axum::{response::Json, routing, Router};
use serde::{Deserialize, Serialize};

use crate::state::AppState;
use crate::web::error::Result;

pub fn routes(state: Arc<AppState>) -> Router {
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

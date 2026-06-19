use std::time::Instant;

use axum::{body::Body, extract::Request, middleware::Next, response::Response};
use chrono::{DateTime, Utc};
use serde::Serialize;
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct LogLine {
    pub id: Uuid,
    pub datetime: DateTime<Utc>,
    pub method: String,
    pub path: String,
    pub status: u16,
    pub latency_ms: u128,
}

impl LogLine {
    fn new(method: String, path: String, status: u16, latency_ms: u128) -> Self {
        Self {
            id: Uuid::now_v7(),
            datetime: Utc::now(),
            method,
            path,
            status,
            latency_ms,
        }
    }
}

pub async fn mw_logging(req: Request<Body>, next: Next) -> Response {
    let method = req.method().to_string();
    let path = req.uri().path().to_owned();
    let started = Instant::now();

    let res = next.run(req).await;

    let line = LogLine::new(
        method,
        path,
        res.status().as_u16(),
        started.elapsed().as_millis(),
    );

    write_log(&line).await;

    res
}

async fn write_log(line: &LogLine) {
    info!(
        method = %line.method,
        path = %line.path,
        status = line.status,
        latency_ms = line.latency_ms,
        datetime = %line.datetime,
        "request completed"
    );

    // TODO: persist `line` (file, db, queue, etc.)
}

pub mod config;
pub mod state;
pub mod web;

use axum::{Router, serve::Serve};
use config::config;
use std::sync::Arc;
use tokio::net::TcpListener;
use web::routes::{default, health, jobs};

pub struct App {
    server: Serve<TcpListener, Router, Router>,
}

impl App {
    pub async fn new() -> Self {
        let state = Arc::new(
            state::AppState::new()
                .await
                .expect("failed to initialize application state"),
        );

        let addr = std::net::SocketAddr::from((config().ip, config().port));

        let listener = TcpListener::bind(addr).await.unwrap();

        tracing::info!(
            "server listening on {} (public_dir={})",
            addr,
            config().public_dir
        );

        let router = Router::new()
            .nest(
                "/api",
                health::routes(state.clone()).nest("/jobs", jobs::routes(state)),
            )
            .merge(default::routes(&config().public_dir));

        Self {
            server: axum::serve(listener, router),
        }
    }

    pub async fn run(self) {
        self.server.await.unwrap();
    }
}

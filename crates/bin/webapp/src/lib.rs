pub mod config;
pub mod state;
pub mod web;

use std::sync::Arc;
use tokio::net::TcpListener;
use axum::{Router, serve::Serve};
use config::config;
use web::routes::{default, health};

pub struct App {
    server: Serve<TcpListener, Router, Router>
}

impl App {

    pub async fn new() -> Self {

        let state = Arc::new(state::State::new().await);

        let addr = std::net::SocketAddr::from((config().ip, config().port));

        let listener = TcpListener::bind(addr).await.unwrap();

        let router = Router::new()
            .merge(default::routes(&config().public_dir))
            .nest("/api", health::routes(state.clone()));

        Self {
            server: axum::serve(listener, router),
        }

    }

    pub async fn run(self) {
        self.server.await.unwrap();
    }

}
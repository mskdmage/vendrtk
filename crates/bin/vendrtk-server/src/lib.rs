mod config;
pub mod middleware;
pub mod services;
pub mod state;
pub mod web;

pub use config::{Config, config};

use std::{net::SocketAddr, sync::Arc};

use axum::{Router, serve::Serve};
use tokio::net::TcpListener;

use crate::state::AppState;

pub struct App {
    server: Serve<TcpListener, Router, Router>,
    addr: SocketAddr,
}

impl App {
    pub async fn build(addr: SocketAddr, public_dir: &str) -> std::io::Result<Self> {
        let state = Arc::new(
            AppState::new(
                "testfiles/samples",
                "testfiles/ocr",
                "testfiles/parsed/invoices",
                "testfiles/parsed/sows",
            )
            .await?,
        );
        let listener = TcpListener::bind(addr).await?;
        let addr = listener.local_addr()?;

        let router = Router::new()
            .nest("/api/files", web::routes_files::routes(state.clone()))
            .merge(web::routes_default::routes(public_dir))
            .layer(axum::middleware::from_fn(middleware::mw_logging))
            .layer(axum::Extension(state.clone()));

        Ok(Self {
            server: axum::serve(listener, router),
            addr,
        })
    }

    pub async fn run(self) -> std::io::Result<()> {
        tracing::info!("listening on http://{}", self.addr);
        self.server.await
    }
}

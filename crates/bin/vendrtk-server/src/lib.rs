mod config;
pub mod middleware;
pub mod web;

pub use config::{Config, config};

use std::net::SocketAddr;

use axum::{Router, serve::Serve};
use tokio::net::TcpListener;

pub struct App {
    server: Serve<TcpListener, Router, Router>,
    addr: SocketAddr,
}

impl App {
    pub async fn build(addr: SocketAddr, public_dir: &str) -> std::io::Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        let addr = listener.local_addr()?;

        let router = Router::new()
            .nest("/api/files", web::routes_files::routes())
            .merge(web::routes_default::routes(public_dir))
            .layer(axum::middleware::from_fn(middleware::mw_logging));

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

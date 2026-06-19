use axum::response::{Html, IntoResponse};
use axum::routing::{Router, get};
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(addr).await.unwrap();

    let app = Router::new().route("/", get(root_handler));

    axum::serve(listener, app).await.unwrap();

    info!("Server started: http://{}", addr);
}

async fn root_handler() -> impl IntoResponse {
    Html("<h1>Hello World!</h1>")
}

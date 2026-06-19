use tokio::net::TcpListener;
use tracing::info;
use vendrtk_server::config;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_env_filter(config().log_level.as_str())
        .init();

    let addr = config().socket_addr();
    let listener = TcpListener::bind(addr).await.unwrap();

    info!("Server started: http://{}", addr);

    axum::serve(listener, vendrtk_server::app()).await.unwrap();
}

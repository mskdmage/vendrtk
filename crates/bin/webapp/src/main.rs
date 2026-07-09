use webapp::{App, config::config};

#[tokio::main]
async fn main() {

    tracing_subscriber::fmt()
        .with_target(true)
        .with_env_filter(config().log_level.as_str())
        .init();

    tracing::info!("tracing initialized (log_level={})", config().log_level);

    let app = App::new().await;

    app.run().await;

}
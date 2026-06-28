use webapp::{App, config::config};

#[tokio::main]
async fn main() {

    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_env_filter(config().log_level.as_str())
        .init();

    let app = App::new().await;

    app.run().await;

}
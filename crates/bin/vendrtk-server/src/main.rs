use vendrtk_server::{App, config};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_env_filter(config().log_level.as_str())
        .init();

    App::build(config().socket_addr(), &config().public_dir)
        .await
        .expect("failed to build server")
        .run()
        .await
        .expect("server failed");
}

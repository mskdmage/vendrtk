use vendrtk_server::{App, config};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_env_filter(config().log_level.as_str())
        .init();

    let app = match App::build(config().socket_addr(), &config().public_dir).await {
        Ok(app) => app,
        Err(err) => {
            eprintln!("failed to build server: {err}");
            std::process::exit(1);
        }
    };

    if let Err(err) = app.run().await {
        eprintln!("server failed: {err}");
        std::process::exit(1);
    }
}

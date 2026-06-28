use axum::{
    Router,
    handler::HandlerWithoutStateExt,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{MethodRouter, any_service},
};
use tower_http::services::ServeDir;

pub fn routes(public_dir: &str) -> Router {
    Router::new().fallback_service(serve_dir(public_dir))
}

fn serve_dir(public_dir: &str) -> MethodRouter {
    any_service(
        ServeDir::new(public_dir)
        .not_found_service(not_found_handler.into_service())
    )
}

async fn not_found_handler() -> impl IntoResponse {
    let payload = r#"
<h1>Not Found</h1>
    "#;

    (StatusCode::NOT_FOUND, Html(payload))
}
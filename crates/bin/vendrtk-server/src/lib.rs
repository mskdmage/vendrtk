mod config;
pub use config::config;
pub mod middleware;
pub mod web;

use axum::Router;

pub fn app() -> Router {
    Router::new()
        .nest("/api/files", web::routes_files::routes())
        .merge(web::routes_default::routes())
        .layer(axum::middleware::from_fn(middleware::mw_logging))
}

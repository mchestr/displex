use axum::{
    routing::get_service,
    Router,
};
use http::StatusCode;
use tower_http::services::{
    ServeDir,
    ServeFile,
};

use crate::config::AppConfig;

use super::DisplexState;

mod discord;
mod graphql;
mod plex;

pub fn configure(config: &AppConfig) -> Router<DisplexState> {
    let mut router = Router::new().merge(discord::routes()).merge(plex::routes());

    if config.api.enabled {
        router = router.nest("/gql", graphql::routes());
    }
    router
        .nest_service(
            "/assets",
            get_service(ServeDir::new("./dist/assets")).handle_error(|error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {}", error),
                )
            }),
        )
        .fallback_service(
            get_service(ServeFile::new("./dist/index.html")).handle_error(|_| async move {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error")
            }),
        )
}

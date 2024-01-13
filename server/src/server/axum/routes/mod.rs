use axum::{Router, routing::get_service};
use http::StatusCode;
use tower_http::services::ServeDir;

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
    router.fallback(
        get_service(ServeDir::new("./dist")).handle_error(|_| async move {
            (StatusCode::INTERNAL_SERVER_ERROR, "internal server error")
        }),
    )
}

use axum::Router;

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
}

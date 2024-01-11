use axum::Router;

use crate::config::AppConfig;

use super::DisplexState;

mod discord;
mod plex;

pub fn configure(_config: &AppConfig) -> Router<DisplexState> {
    Router::new().merge(discord::routes()).merge(plex::routes())
}

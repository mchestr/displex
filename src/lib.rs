pub mod bot;
pub mod config;
pub mod discord;
pub mod entities;
pub mod errors;
pub mod graphql;
pub mod metadata;
pub mod migrations;
pub mod overseerr;
pub mod plex;
pub mod server;
pub mod services;
pub mod tautulli;

pub mod discord_token;
pub mod discord_user;
pub mod plex_token;
pub mod plex_user;

pub static VERSION: &str = env!("CARGO_PKG_VERSION");
pub static AUTHOR: &str = "mchestr";
pub static PROJECT_NAME: &str = env!("CARGO_PKG_NAME");
pub static REPOSITORY_LINK: &str = "https://github.com/mchestr/displex";

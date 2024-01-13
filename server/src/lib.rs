pub mod bot;
pub mod config;
pub mod entities;
pub mod errors;
pub mod graphql;
pub mod migrations;
pub mod server;
pub mod services;
pub mod tasks;

pub static VERSION: &str = env!("CARGO_PKG_VERSION");
pub static AUTHOR: &str = "mchestr";
pub static PROJECT_NAME: &str = env!("CARGO_PKG_NAME");
pub static REPOSITORY_LINK: &str = "https://github.com/mchestr/displex";

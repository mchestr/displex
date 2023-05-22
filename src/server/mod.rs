use async_trait::async_trait;
use clap::ValueEnum;
use derive_more::Display;

use crate::config::ServerArgs;

pub mod axum;

#[derive(Debug, Copy, Clone, Default, Display, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Server {
    #[default]
    Axum,
}

#[async_trait]
pub trait DisplexHttpServer {
    async fn run(&self, config: ServerArgs);
}

#[async_trait]
impl DisplexHttpServer for Server {
    async fn run(&self, config: ServerArgs) {
        match self {
            Server::Axum => axum::run(config).await,
        }
    }
}

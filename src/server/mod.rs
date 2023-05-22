use async_trait::async_trait;
use clap::ValueEnum;
use derive_more::Display;

use crate::config::ServerArgs;

pub mod axum;

#[derive(Debug, Copy, Clone, Default, Display, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Server {
    #[default]
    Axum,
    Disabled,
}

#[async_trait]
pub trait DisplexHttpServer {
    async fn run(&self, rx: tokio::sync::broadcast::Receiver<()>, config: ServerArgs);
}

#[async_trait]
impl DisplexHttpServer for Server {
async fn run(&self, rx: tokio::sync::broadcast::Receiver<()>, config: ServerArgs) {
        match self {
            Server::Axum => axum::run(rx, config).await,
            Server::Disabled => tracing::info!("server disabled"),
        }
    }
}

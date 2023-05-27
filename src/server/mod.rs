use anyhow::Result;
use async_trait::async_trait;
use clap::ValueEnum;
use derive_more::Display;
use serde::{
    Deserialize,
    Serialize,
};

use crate::{config::DisplexConfig, utils::DisplexClients};

pub mod axum;

#[derive(
    Debug,
    Copy,
    Clone,
    Deserialize,
    Default,
    Display,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    ValueEnum,
    Serialize,
)]
pub enum Server {
    #[default]
    Axum,
    Disabled,
}

#[async_trait]
pub trait DisplexHttpServer {
    async fn run(
        &self,
        rx: tokio::sync::broadcast::Receiver<()>,
        config: DisplexConfig,
        clients: &DisplexClients,
    ) -> Result<()>;
}

#[async_trait]
impl DisplexHttpServer for Server {
    async fn run(
        &self,
        rx: tokio::sync::broadcast::Receiver<()>,
        config: DisplexConfig,
        clients: &DisplexClients,
    ) -> Result<()> {
        match self {
            Server::Axum => axum::run(rx, config, clients).await,
            Server::Disabled => tracing::info!("server disabled"),
        }
        Ok(())
    }
}

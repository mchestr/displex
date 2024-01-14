use anyhow::Result;
use async_trait::async_trait;
use clap::ValueEnum;
use derive_more::Display;
use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    config::AppConfig,
    graphql::GraphqlSchema,
    services::AppServices,
};

pub mod axum;
pub mod cookies;

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
        config: AppConfig,
        services: &AppServices,
        schema: &GraphqlSchema,
    ) -> Result<()>;
}

#[async_trait]
impl DisplexHttpServer for Server {
    async fn run(
        &self,
        rx: tokio::sync::broadcast::Receiver<()>,
        config: AppConfig,
        services: &AppServices,
        schema: &GraphqlSchema,
    ) -> Result<()> {
        match self {
            Server::Axum => axum::run(rx, config, services, schema).await,
            Server::Disabled => tracing::info!("server disabled"),
        }
        Ok(())
    }
}

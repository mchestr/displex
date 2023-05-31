use anyhow::Result;
use async_trait::async_trait;
use clap::ValueEnum;
use derive_more::Display;
use serde::{
    Deserialize,
    Serialize,
};
use tokio::sync::broadcast::Receiver;

use crate::{
    config::AppConfig,
    services::AppServices,
};

pub mod discord;

#[derive(
    Debug,
    Copy,
    Clone,
    Default,
    Deserialize,
    Display,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    ValueEnum,
    Serialize,
)]
#[display = "lowercase"]
pub enum DiscordBot {
    #[default]
    Serenity,
    Disabled,
}

#[async_trait]
pub trait DisplexBot {
    async fn run(
        &self,
        rx: Receiver<()>,
        config: AppConfig,
        serenity_client: serenity::Client,
        services: &AppServices,
    ) -> Result<()>;
}

#[async_trait]
impl DisplexBot for DiscordBot {
    async fn run(
        &self,
        rx: Receiver<()>,
        config: AppConfig,
        serenity_client: serenity::Client,
        services: &AppServices,
    ) -> Result<()> {
        match self {
            DiscordBot::Serenity => discord::run(rx, config, serenity_client, services).await,
            DiscordBot::Disabled => tracing::info!("bot disabled"),
        }
        Ok(())
    }
}

use async_trait::async_trait;
use clap::ValueEnum;
use derive_more::Display;
use tokio::sync::broadcast::Receiver;

use crate::config::{DiscordBotArgs};

mod serenity;

#[derive(Debug, Copy, Clone, Default, Display, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum DiscordBot {
    Serenity,
    #[default]
    Disabled,
}

#[async_trait]
pub trait DisplexBot {
    async fn run(&self, rx: Receiver<()>, config: DiscordBotArgs);
}

#[async_trait]
impl DisplexBot for DiscordBot {
    async fn run(&self, rx: Receiver<()>, config: DiscordBotArgs) {
        match self {
            DiscordBot::Serenity => serenity::run(rx, config).await,
            DiscordBot::Disabled => tracing::info!("bot disabled"),
        }
    }
}

use async_trait::async_trait;
use clap::ValueEnum;
use derive_more::Display;
use tokio::sync::broadcast::Receiver;

use crate::config::ServerArgs;

mod discord;


#[derive(Debug, Copy, Clone, Default, Display, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Bot {
    #[default]
    Discord,
    Disabled,
}


#[async_trait]
pub trait DisplexBot {
    async fn run(&self, rx: Receiver<()>, config: ServerArgs);
}

#[async_trait]
impl DisplexBot for Bot {
    async fn run(&self, rx: Receiver<()>, config: ServerArgs) {
        match self {
            Bot::Discord => discord::run(rx, config).await,
            Bot::Disabled => tracing::info!("bot disabled"),
        }
    }
}

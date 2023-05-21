use async_trait::async_trait;
use clap::ValueEnum;
use derive_more::Display;

use crate::config::ServerArgs;

pub mod actix_web;

#[derive(Debug, Display, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Server {
    ActixWeb,
}

#[async_trait]
pub trait DisplexHttpServer {
    async fn run(&self, config: ServerArgs);
}

#[async_trait]
impl DisplexHttpServer for Server {
    async fn run(&self, config: ServerArgs) {
        match self {
            Server::ActixWeb => actix_web::run(config).await.unwrap(),
        }
    }
}

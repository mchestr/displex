use async_trait::async_trait;
use clap::ValueEnum;
use derive_more::Display;

use crate::config::ServerArgs;
#[cfg(all(feature = "actix-web", feature = "axum"))]
compile_error!("features `displex/actix-web` and `displex/axum` are mutually exclusive");

#[cfg(feature = "actix-web")]
pub mod actix_web;

#[cfg(feature = "axum")]
pub mod axum;

#[derive(Debug, Copy, Clone, Default, Display, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Server {
    #[cfg(feature = "actix-web")]
    #[default]
    ActixWeb,
    #[cfg(feature = "axum")]
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
            #[cfg(feature = "actix-web")]
            Server::ActixWeb => actix_web::run(config).await.unwrap(),
            #[cfg(feature = "axum")]
            Server::Axum => axum::run(config).await,
        }
    }
}

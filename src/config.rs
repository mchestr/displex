use std::{
    fmt,
    path::Path,
    time::Duration,
};

use anyhow::{
    Context,
    Result,
};
use derivative::Derivative;
use figment::{
    providers::{
        Env,
        Format,
        Toml,
    },
    Figment,
};

use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    bot::DiscordBot,
    server::Server,
};

fn obfuscated_formatter(val: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{:?}", "*".repeat(val.len()))
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct DisplexConfig {
    #[serde(default = "default_application_name")]
    pub application_name: String,
    pub database: DatabaseConfig,
    #[serde(default = "default_debug")]
    pub debug: DebugConfig,
    pub discord: DiscordConfig,
    pub discord_bot: DiscordBotConfig,
    #[serde(default = "default_http")]
    pub http: HttpConfig,
    pub plex: PlexConfig,
    #[serde(default = "default_session")]
    pub session: SessionConfig,
    pub tautulli: TautulliConfig,
}

fn default_application_name() -> String {
    "displex".into()
}

fn default_debug() -> DebugConfig {
    DebugConfig {
        accept_invalid_certs: default_debug_accept_invalid_certs(),
    }
}

fn default_http() -> HttpConfig {
    HttpConfig {
        type_: default_http_server(),
        hostname: "localhost".into(),
        host: default_http_host(),
        port: default_http_port(),
    }
}

fn default_session() -> SessionConfig {
    SessionConfig {
        secret_key: default_session_secret(),
    }
}

#[derive(Derivative, Deserialize, Clone, Serialize)]
#[derivative(Debug)]
pub struct DatabaseConfig {
    #[derivative(Debug(format_with = "obfuscated_formatter"))]
    pub url: String,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct DebugConfig {
    #[serde(default = "default_debug_accept_invalid_certs")]
    pub accept_invalid_certs: bool,
}

fn default_debug_accept_invalid_certs() -> bool {
    false
}

#[derive(Derivative, Deserialize, Clone, Serialize)]
#[derivative(Debug)]
pub struct DiscordConfig {
    pub client_id: u64,
    #[derivative(Debug(format_with = "obfuscated_formatter"))]
    pub client_secret: String,
    pub server_id: u64,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct HttpConfig {
    #[serde(default = "default_http_server", rename = "type")]
    pub type_: Server,
    pub hostname: String,
    #[serde(default = "default_http_host")]
    pub host: String,
    #[serde(default = "default_http_port")]
    pub port: u16,
}

fn default_http_server() -> Server {
    Server::Axum
}

fn default_http_host() -> String {
    "127.0.0.1".into()
}

fn default_http_port() -> u16 {
    8080
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct PlexConfig {
    pub server_id: String,
}

#[derive(Derivative, Deserialize, Clone, Serialize)]
#[derivative(Debug)]
pub struct SessionConfig {
    #[derivative(Debug(format_with = "obfuscated_formatter"))]
    #[serde(default = "default_session_secret")]
    pub secret_key: String,
}

fn default_session_secret() -> String {
    "youshouldnotusethisinproductionandchangeme".into()
}

#[derive(Derivative, Deserialize, Clone, Serialize)]
#[derivative(Debug)]
pub struct TautulliConfig {
    #[serde(default = "default_tautulli_url")]
    pub url: String,
    #[derivative(Debug(format_with = "obfuscated_formatter"))]
    pub api_key: String,
}

fn default_tautulli_url() -> String {
    "http://localhost:8181".into()
}

#[derive(Derivative, Deserialize, Clone, Serialize)]
#[derivative(Debug)]
pub struct DiscordBotConfig {
    #[serde(default = "default_discord_bot", rename = "type")]
    pub type_: DiscordBot,
    #[derivative(Debug(format_with = "obfuscated_formatter"))]
    pub token: String,
    #[serde(default = "default_discord_bot_status_text")]
    pub status_text: String,
    pub stat_update: Option<StatUpdateConfig>,
    pub user_update: Option<UserUpdateConfig>,
}

fn default_discord_bot() -> DiscordBot {
    DiscordBot::Serenity
}

fn default_discord_bot_status_text() -> String {
    "DisPlex".into()
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct StatUpdateConfig {
    #[serde(with = "humantime_serde", default = "default_stat_update_job_interval")]
    pub interval: Duration,
    #[serde(default = "default_update_channel_bot_role_name")]
    pub bot_role_name: String,
    #[serde(default = "default_update_channel_subscriber_role_name")]
    pub subscriber_role_name: String,
    pub stats_category: Option<StatCategoryConfig>,
    pub library_category: Option<LibraryCategoryConfig>,
}

fn default_stat_update_job_interval() -> Duration {
    Duration::from_secs(60)
}

fn default_user_update_job_interval() -> Duration {
    Duration::from_secs(3600)
}

fn default_update_channel_bot_role_name() -> String {
    "Bot".into()
}

fn default_update_channel_subscriber_role_name() -> String {
    "Subscriber".into()
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct UserUpdateConfig {
    #[serde(with = "humantime_serde", default = "default_user_update_job_interval")]
    pub interval: Duration,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct StatCategoryConfig {
    #[serde(default = "default_stat_category_name")]
    pub name: String,
    pub stream_name: Option<String>,
    pub transcode_name: Option<String>,
    pub bandwidth_total_name: Option<String>,
    pub bandwidth_local_name: Option<String>,
    pub bandwidth_remote_name: Option<String>,
}

fn default_stat_category_name() -> String {
    "Plex Stats".into()
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct LibraryCategoryConfig {
    #[serde(default = "default_library_category_name")]
    pub name: String,
    pub movies_name: Option<String>,
    pub tv_shows_name: Option<String>,
    pub tv_episodes_name: Option<String>,
}

fn default_library_category_name() -> String {
    "Plex Library Stats".into()
}

pub fn load(config_file: &str) -> Result<DisplexConfig> {
    Figment::new()
        .merge(Toml::file(Path::new(config_file).join("config.toml")))
        .merge(Env::prefixed("DISPLEX_").split("__"))
        .extract()
        .context("Unable to construct application configuration")
}

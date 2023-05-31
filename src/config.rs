use std::{
    fmt,
    path::PathBuf,
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
        Json,
        Serialized,
        Toml,
        Yaml,
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
    PROJECT_NAME,
};

fn obfuscated_formatter(val: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{:?}", "*".repeat(val.len()))
}

#[derive(Deserialize, Debug, Clone, Serialize, Default)]
pub struct AppConfig {
    pub application_name: String,
    pub database: DatabaseConfig,
    pub debug: DebugConfig,
    pub discord: DiscordConfig,
    pub discord_bot: DiscordBotConfig,
    pub http: HttpConfig,
    pub plex: PlexConfig,
    pub session: SessionConfig,
    pub tautulli: TautulliConfig,
    pub web: WebConfig,
}

#[derive(Derivative, Deserialize, Clone, Serialize)]
#[derivative(Debug)]
pub struct DatabaseConfig {
    #[derivative(Debug(format_with = "obfuscated_formatter"))]
    pub url: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: format!("sqlite://{PROJECT_NAME}.db?mode=rwc"),
        }
    }
}

#[derive(Deserialize, Debug, Clone, Serialize, Default)]
pub struct DebugConfig {
    pub accept_invalid_certs: bool,
}

#[derive(Derivative, Deserialize, Clone, Serialize, Default)]
#[derivative(Debug)]
pub struct DiscordConfig {
    pub client_id: u64,
    #[derivative(Debug(format_with = "obfuscated_formatter"))]
    pub client_secret: String,
    pub server_id: u64,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct HttpConfig {
    #[serde(rename = "type")]
    pub type_: Server,
    pub hostname: String,
    pub host: String,
    pub port: u16,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 8080,
            hostname: "localhost".into(),
            type_: Default::default(),
        }
    }
}

#[derive(Deserialize, Debug, Clone, Serialize, Default)]
pub struct PlexConfig {
    pub server_id: String,
}

#[derive(Derivative, Deserialize, Clone, Serialize)]
#[derivative(Debug)]
pub struct SessionConfig {
    #[derivative(Debug(format_with = "obfuscated_formatter"))]
    pub secret_key: String,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            secret_key: "youshouldnotusethisinproductionandchangeme".into(),
        }
    }
}

#[derive(Derivative, Deserialize, Clone, Serialize)]
#[derivative(Debug)]
pub struct TautulliConfig {
    pub url: String,
    #[derivative(Debug(format_with = "obfuscated_formatter"))]
    pub api_key: String,
}

impl Default for TautulliConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:8181".into(),
            api_key: Default::default(),
        }
    }
}

#[derive(Derivative, Deserialize, Clone, Serialize)]
#[derivative(Debug)]
pub struct DiscordBotConfig {
    #[serde(rename = "type")]
    pub type_: DiscordBot,
    #[derivative(Debug(format_with = "obfuscated_formatter"))]
    pub token: String,
    pub status_text: String,
    pub stat_update: StatUpdateConfig,
    pub user_update: UserUpdateConfig,
}

impl Default for DiscordBotConfig {
    fn default() -> Self {
        Self {
            status_text: "DisPlex".into(),
            type_: Default::default(),
            token: Default::default(),
            stat_update: Default::default(),
            user_update: Default::default(),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct StatUpdateConfig {
    pub enabled: bool,
    #[serde(with = "humantime_serde")]
    pub interval: Duration,
    pub bot_role_name: String,
    pub subscriber_role_name: String,
    pub stats_category: Option<StatCategoryConfig>,
    pub library_category: Option<LibraryCategoryConfig>,
}

impl Default for StatUpdateConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: Duration::from_secs(60),
            bot_role_name: "Bot".into(),
            subscriber_role_name: "Subscriber".into(),
            stats_category: None,
            library_category: None,
        }
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct UserUpdateConfig {
    pub enabled: bool,
    #[serde(with = "humantime_serde")]
    pub interval: Duration,
}

impl Default for UserUpdateConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: Duration::from_secs(3600),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct StatCategoryConfig {
    pub name: String,
    pub stream_name: Option<String>,
    pub transcode_name: Option<String>,
    pub bandwidth_total_name: Option<String>,
    pub bandwidth_local_name: Option<String>,
    pub bandwidth_remote_name: Option<String>,
}

impl Default for StatCategoryConfig {
    fn default() -> Self {
        Self {
            name: "Plex Stats".into(),
            stream_name: None,
            transcode_name: None,
            bandwidth_local_name: None,
            bandwidth_remote_name: None,
            bandwidth_total_name: None,
        }
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct LibraryCategoryConfig {
    pub name: String,
    pub movies_name: Option<String>,
    pub tv_shows_name: Option<String>,
    pub tv_episodes_name: Option<String>,
}

impl Default for LibraryCategoryConfig {
    fn default() -> Self {
        Self {
            name: "Plex Library Stats".into(),
            movies_name: None,
            tv_episodes_name: None,
            tv_shows_name: None,
        }
    }
}

#[derive(Deserialize, Debug, Clone, Serialize, Default)]
pub struct WebConfig {
    pub cors_origins: Vec<String>,
    pub insecure_cookie: bool,
}

pub fn load(path: &str) -> Result<AppConfig> {
    Figment::new()
        .merge(Serialized::defaults(AppConfig::default()))
        .merge(Json::file(
            PathBuf::from(path).join(format!("{PROJECT_NAME}.json")),
        ))
        .merge(Toml::file(
            PathBuf::from(path).join(format!("{PROJECT_NAME}.toml")),
        ))
        .merge(Yaml::file(
            PathBuf::from(path).join(format!("{PROJECT_NAME}.yaml")),
        ))
        .merge(Env::raw().split("_").only(&["database.url"]))
        .merge(
            Env::prefixed(&format!("{PROJECT_NAME}_"))
                .split("__")
                .ignore(&["database.url"]),
        )
        .extract()
        .context("Unable to construct application configuration")
}

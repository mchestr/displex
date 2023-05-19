use std::env;

use derive_more::Display;

#[derive(Clone, Display)]
#[display(
    fmt = "Config(
        host: {}, 
        port: {}, 
        hostname: {}, 
        session_secret_key: *****,
        database_url: *****,
        application_name: {}, 
        accept_invalid_certs: {}, 
        plex_server_id: {}, 
        discord_client_id: {}, 
        discord_client_secret: *****,
        discord_bot_token: *****,
        tautulli_url: {},
        tautulli_api_key: *****,
    )",
    host,
    hostname,
    port,
    application_name,
    accept_invalid_certs,
    plex_server_id,
    discord_client_id,
    tautulli_url
)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub hostname: String,

    pub session_secret_key: String,
    pub database_url: String,
    pub application_name: String,
    pub accept_invalid_certs: bool,

    pub plex_server_id: String,

    pub discord_client_id: String,
    pub discord_client_secret: String,
    pub discord_bot_token: String,

    pub tautulli_url: String,
    pub tautulli_api_key: String,
}

impl Config {
    pub fn init() -> Config {
        Config {
            hostname: env::var("DISPLEX_HOSTNAME").expect("DISPLEX_HOSTNAME not set"),
            application_name: env::var("DISPLEX_APPLICATION_NAME")
                .expect("DISPLEX_APPLICATION_NAME not set"),
            host: env::var("DISPLEX_HOST").unwrap_or("127.0.0.1".into()),
            port: env::var("DISPLEX_PORT").map_or(8080, |v| {
                v.parse::<u16>()
                    .map_err(|e| format!("DISPLEX_PORT '{}' is invalid", e))
                    .unwrap()
            }),
            session_secret_key: env::var("DISPLEX_SESSION_SECRET_KEY")
                .expect("DISPLEX_SESSION_SECRET_KEY not set."),
            database_url: env::var("DISPLEX_DATABASE_URL").expect("DISPLEX_DATABASE_URL not set."),
            accept_invalid_certs: match env::var("DISPLEX_ACCEPT_INVALID_CERTS") {
                Ok(value) => match value.to_lowercase().as_str() {
                    "true" | "t" | "yes" | "y" => true,
                    _ => false,
                },
                Err(_) => false,
            },

            plex_server_id: env::var("DISPLEX_PLEX_SERVER_ID")
                .expect("DISPLEX_PLEX_SERVER_ID not set"),

            discord_client_id: env::var("DISPLEX_DISCORD_CLIENT_ID")
                .expect("DISPLEX_DISCORD_CLIENT_ID not set"),
            discord_client_secret: env::var("DISPLEX_DISCORD_CLIENT_SECRET")
                .expect("DISPLEX_DISCORD_CLIENT_SECRET not set"),
            discord_bot_token: env::var("DISPLEX_DISCORD_BOT_TOKEN")
                .expect("DISPLEX_DISCORD_BOT_TOKEN not set"),

            tautulli_api_key: env::var("DISPLEX_TAUTULLI_API_KEY")
                .expect("DISPLEX_TAUTULLI_API_KEY not set"),
            tautulli_url: env::var("DISPLEX_TAUTULLI_URL").expect("DISPLEX_TAUTULLI_URL not set"),
        }
    }
}

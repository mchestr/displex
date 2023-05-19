use std::env;

#[derive(Clone)]
pub struct Config {
    pub host: String,
    pub hostname: String,
    pub port: u16,
    pub session_secret_key: String,
    pub database_url: String,
    pub application_name: String,

    pub plex_server_id: String,

    pub discord_client_id: String,
    pub discord_client_secret: String,
    pub discord_bot_token: String,
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

            plex_server_id: env::var("DISPLEX_PLEX_SERVER_ID")
                .expect("DISPLEX_PLEX_SERVER_ID not set"),

            discord_client_id: env::var("DISPLEX_DISCORD_CLIENT_ID")
                .expect("DISPLEX_DISCORD_CLIENT_ID not set"),
            discord_client_secret: env::var("DISPLEX_DISCORD_CLIENT_SECRET")
                .expect("DISPLEX_DISCORD_CLIENT_SECRET not set"),
            discord_bot_token: env::var("DISPLEX_DISCORD_BOT_TOKEN")
                .expect("DISPLEX_DISCORD_BOT_TOKEN not set"),
        }
    }
}

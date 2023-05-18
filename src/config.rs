use std::env;

#[derive(Clone)]
pub struct Config {
    pub host: String,
    pub hostname: String,
    pub port: u16,
    pub session_secret_key: String,

    pub plex_client_id: String,
    pub plex_server_id: String,

    pub discord_client_id: String,
    pub discord_client_secret: String,
}

impl Config {
    pub fn init() -> Config {
        Config {
            hostname: env::var("TAUTBOT_HOSTNAME").expect("TAUTBOT_HOSTNAME not set"),
            host: env::var("TAUTBOT_HOST").unwrap_or("0.0.0.0".into()),
            port: env::var("TAUTBOT_PORT")
                .unwrap_or("8080".into())
                .parse::<u16>()
                .expect("TAUTBOT_PORT not set"),
            session_secret_key: env::var("TAUTBOT_SESSION_SECRET_KEY")
                .expect("TAUTBOT_SESSION_SECRET_KEY not set."),

            plex_client_id: env::var("TAUTBOT_PLEX_CLIENT_ID")
                .expect("TAUTBOT_PLEX_CLIENT_ID not set"),
            plex_server_id: env::var("TAUTBOT_PLEX_SERVER_ID")
                .expect("TAUTBOT_PLEX_SERVER_ID not set"),

            discord_client_id: env::var("TAUTBOT_DISCORD_CLIENT_ID")
                .expect("TAUTBOT_DISCORD_CLIENT_ID not set"),
            discord_client_secret: env::var("TAUTBOT_DISCORD_CLIENT_SECRET")
                .expect("TAUTBOT_DISCORD_CLIENT_SECRET not set"),
        }
    }
}

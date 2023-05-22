use clap::Args;
use derive_more::Display;

use crate::server::Server;

#[derive(Display, Clone)]
#[display(fmt = "********")]
pub struct Secret(String);

impl Secret {
    pub fn sensitive_string(&self) -> String {
        String::from(&self.0)
    }
}

impl From<String> for Secret {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Args, Clone, Display)]
#[display(fmt = "{{
    application_name: {application_name},
    hostname: {hostname},
    host: {host},
    port: {port},
    accept_valid_certs: {accept_invalid_certs},
    session: {session},
    database: {database},
    discord: {discord},
    plex: {plex},
    tautulli: {tautulli},
}}")]
pub struct ServerArgs {
    #[arg(long, env = "DISPLEX_APPLICATION_NAME", default_value = "DisPlex")]
    pub application_name: String,

    #[arg(long, env = "DISPLEX_HOSTNAME", required = true)]
    pub hostname: String,

    #[arg(long, env = "DISPLEX_HTTP_HOST", default_value = "127.0.0.1")]
    pub host: String,

    #[arg(long, env = "DISPLEX_HTTP_PORT", default_value = "8080")]
    pub port: u16,

    #[arg(long, env = "DISPLEX_ACCEPT_INVALID_CERTS", default_value = "false")]
    pub accept_invalid_certs: bool,

    #[arg(long, env = "DISPLEX_HTTP_SERVER", value_enum, default_value_t)]
    pub http_server: Server,

    #[command(flatten)]
    pub session: SessionArgs,

    #[clap(flatten)]
    pub database: DatabaseArgs,

    #[command(flatten)]
    pub discord: DiscordArgs,

    #[clap(flatten)]
    pub plex: PlexArgs,

    #[clap(flatten)]
    pub tautulli: TautulliArgs,
}

#[derive(Args, Clone, Display)]
#[display(fmt = "{{
        discord_client_id: {discord_client_id},
        discord_client_secret: {discord_client_secret},
        discord_bot_token: {discord_bot_token},
        discord_server_id: {discord_server_id},
    }}")]
pub struct DiscordArgs {
    #[arg(
        long,
        env = "DISPLEX_DISCORD_CLIENT_ID",
        required = true,
        hide_env_values = true
    )]
    pub discord_client_id: Secret,
    #[arg(
        long,
        env = "DISPLEX_DISCORD_CLIENT_SECRET",
        required = true,
        hide_env_values = true
    )]
    pub discord_client_secret: Secret,
    #[arg(
        long,
        env = "DISPLEX_DISCORD_BOT_TOKEN",
        required = true,
        hide_env_values = true
    )]
    pub discord_bot_token: Secret,
    #[arg(long, env = "DISPLEX_DISCORD_SERVER_ID", required = true)]
    pub discord_server_id: String,
}

#[derive(Args, Clone, Display)]
#[display(fmt = "{{
        plex_server_id: {plex_server_id},
    }}")]
pub struct PlexArgs {
    #[arg(long, env = "DISPLEX_PLEX_SERVER_ID", required = true)]
    pub plex_server_id: String,
}

#[derive(Args, Clone, Display)]
#[display(fmt = "{{
        session_secret_key: {session_secret_key},
    }}")]
pub struct SessionArgs {
    #[arg(
        long,
        env = "DISPLEX_SESSION_SECRET_KEY",
        required = true,
        hide_env_values = true
    )]
    pub session_secret_key: Secret,
}

#[derive(Args, Clone, Display)]
#[display(fmt = "{{
        database_url: {database_url},
    }}")]
pub struct DatabaseArgs {
    #[arg(
        long,
        env = "DISPLEX_DATABASE_URL",
        required = true,
        hide_env_values = true
    )]
    pub database_url: Secret,
}

#[derive(Args, Clone, Display)]
#[display(fmt = "{{
        tautulli_url: {tautulli_url},
        tautulli_api_key: {tautulli_api_key},
    }}")]
pub struct TautulliArgs {
    #[arg(long, env = "DISPLEX_TAUTULLI_URL", required = true)]
    pub tautulli_url: String,
    #[arg(
        long,
        env = "DISPLEX_TAUTULLI_API_KEY",
        required = true,
        hide_env_values = true
    )]
    pub tautulli_api_key: Secret,
}

#[derive(Args, Clone, Display)]
#[display(fmt = "{{
        application_name: {application_name},
        hostname: {hostname},
        accept_invalid_certs: {accept_invalid_certs},
        discord: {discord},
        tautulli: {tautulli},
        database: {database},
    }}")]
pub struct RefreshArgs {
    #[arg(long, env = "DISPLEX_APPLICATION_NAME", default_value = "Displex")]
    pub application_name: String,

    #[arg(long, env = "DISPLEX_HOSTNAME", required = true)]
    pub hostname: String,

    #[arg(long, env = "DISPLEX_ACCEPT_INVALID_CERTS", default_value = "false")]
    pub accept_invalid_certs: bool,

    #[command(flatten)]
    pub discord: DiscordArgs,

    #[clap(flatten)]
    pub tautulli: TautulliArgs,

    #[clap(flatten)]
    pub database: DatabaseArgs,
}

#[derive(Args, Clone, Display)]
#[display(fmt = "{{
        discord_bot_token: {discord_bot_token},
        discord_client_id: {discord_client_id},
    }}")]
pub struct SetMetadataArgs {
    #[arg(
        long,
        env = "DISPLEX_DISCORD_CLIENT_ID",
        required = true,
        hide_env_values = true
    )]
    pub discord_client_id: Secret,
    #[arg(
        long,
        env = "DISPLEX_DISCORD_BOT_TOKEN",
        required = true,
        hide_env_values = true
    )]
    pub discord_bot_token: Secret,
    #[arg(long, env = "DISPLEX_ACCEPT_INVALID_CERTS", default_value = "false")]
    pub accept_invalid_certs: bool,
}

use clap::Args;

#[derive(Debug, Args, Clone)]
pub struct ServerArgs {
    #[arg(long, env = "DISPLEX_APPLICATION_NAME", default_value = "Displex")]
    pub application_name: String,

    #[arg(long, env = "DISPLEX_HOSTNAME", required = true)]
    pub hostname: String,

    #[arg(long, env = "DISPLEX_HOST", default_value = "127.0.0.1")]
    pub host: String,

    #[arg(long, env = "DISPLEX_PORT", default_value = "8080")]
    pub port: u16,

    #[arg(long, env = "DISPLEX_ACCEPT_INVALID_CERTS", default_value = "false")]
    pub accept_invalid_certs: bool,

    #[command(flatten)]
    pub session: SessionArgs,

    #[command(flatten)]
    pub discord: DiscordArgs,

    #[clap(flatten)]
    pub plex: PlexArgs,

    #[clap(flatten)]
    pub database: DatabaseArgs,

    #[clap(flatten)]
    pub tautulli: TautulliArgs,
}

#[derive(Debug, Args, Clone)]
pub struct DiscordArgs {
    #[arg(long, env = "DISPLEX_DISCORD_CLIENT_ID", required = true)]
    pub discord_client_id: String,
    #[arg(long, env = "DISPLEX_DISCORD_CLIENT_SECRET", required = true)]
    pub discord_client_secret: String,
    #[arg(long, env = "DISPLEX_DISCORD_BOT_TOKEN", required = true)]
    pub discord_bot_token: String,
    #[arg(long, env = "DISPLEX_DISCORD_SERVER_ID", required = true)]
    pub discord_server_id: String,
    #[arg(long, env = "DISPLEX_DISCORD_CHANNEL_ID", required = true)]
    pub discord_channel_id: String,
}

#[derive(Debug, Args, Clone)]
pub struct PlexArgs {
    #[arg(long, env = "DISPLEX_PLEX_SERVER_ID", required = true)]
    pub plex_server_id: String,
}

#[derive(Debug, Args, Clone)]
pub struct SessionArgs {
    #[arg(long, env = "DISPLEX_SESSION_SECRET_KEY", required = true)]
    pub session_secret_key: String,
}

#[derive(Debug, Args, Clone)]
pub struct DatabaseArgs {
    #[arg(long, env = "DISPLEX_DATABASE_URL", required = true)]
    pub database_url: String,
}

#[derive(Debug, Args, Clone)]
pub struct TautulliArgs {
    #[arg(long, env = "DISPLEX_TAUTULLI_URL", required = true)]
    pub tautulli_url: String,
    #[arg(long, env = "DISPLEX_TAUTULLI_API_KEY", required = true)]
    pub tautulli_api_key: String,
}

#[derive(Debug, Args, Clone)]
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
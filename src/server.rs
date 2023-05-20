use std::time::Duration;

use actix_session::{
    storage::CookieSessionStore,
    SessionMiddleware,
};
use actix_web::{
    cookie::Key,
    middleware::Logger,
    rt,
    web::{self,},
    App,
    HttpServer,
};

use crate::{
    config::ServerArgs,
    db::{
        self,
        run_migrations,
    },
    discord::{
        client::DiscordClient,
        metadata::register_metadata,
    },
    handlers,
    plex::client::PlexClient,
    tautulli::client::TautulliClient,
};
use db::initialize_db_pool;
use reqwest::header::HeaderValue;

async fn run_app(config: ServerArgs) -> std::io::Result<()> {
    let mut default_headers = reqwest::header::HeaderMap::new();
    default_headers.append("Accept", HeaderValue::from_static("application/json"));

    let reqwest_client = reqwest::ClientBuilder::new()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .pool_idle_timeout(Duration::from_secs(90))
        .default_headers(default_headers)
        .danger_accept_invalid_certs(config.accept_invalid_certs)
        .build()
        .unwrap();

    let discord_client = DiscordClient::new(
        &reqwest_client,
        &config.discord.discord_client_id.sensitive_string(),
        &config.discord.discord_client_secret.sensitive_string(),
        &format!("https://{}/discord/callback", &config.hostname),
        &config.discord.discord_bot_token.sensitive_string(),
        &config.discord.discord_server_id,
        &config.discord.discord_channel_id,
    );

    let plex_client = PlexClient::new(
        &reqwest_client,
        &config.application_name,
        &format!("https://{}/plex/callback", &config.hostname),
    );

    let tautlli_client = TautulliClient::new(
        &reqwest_client,
        &config.tautulli.tautulli_url,
        &config.tautulli.tautulli_api_key.sensitive_string(),
    );

    let pool = initialize_db_pool(&config.database.database_url.sensitive_string());
    let mut conn = pool.get().unwrap();
    run_migrations(&mut conn).unwrap();
    register_metadata(&config, &discord_client).await.unwrap();

    let host = String::from(&config.host);
    let port = config.port;

    // srv is server controller type, `dev::Server`
    let server = HttpServer::new(move || {
        App::new()
            .configure(handlers::configure)
            .app_data(web::Data::new(plex_client.clone()))
            .app_data(web::Data::new(discord_client.clone()))
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(tautlli_client.clone()))
            .wrap(Logger::default())
            .wrap(
                // create cookie based session middleware
                SessionMiddleware::builder(
                    CookieSessionStore::default(),
                    Key::from(
                        config
                            .session
                            .session_secret_key
                            .sensitive_string()
                            .as_bytes(),
                    ),
                )
                .cookie_secure(true)
                .build(),
            )
    })
    .bind((host.clone(), port))?
    .run();

    log::info!("starting HTTP server at http://{}:{}", host, port);
    server.await
}

pub fn run(config: ServerArgs) {
    let server_future = run_app(config);
    rt::System::new()
        .block_on(server_future)
        .expect("failed to gracefully shutdown");
}

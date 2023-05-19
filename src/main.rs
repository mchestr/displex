use std::time::Duration;

use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::Key,
    middleware::Logger,
    web::{self},
    App, HttpServer,
};


use db::{initialize_db_pool};
use displex::{
    config,
    db::{
        self,
        run_migrations,
    },
    discord::{
        client::DiscordClient,
        metadata::register_metadata,
    },
    plex::client::PlexClient,
    tautulli::{client::TautulliClient},
};
use reqwest::header::HeaderValue;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().unwrap();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let config = config::Config::init();
    log::info!("Loaded config: {}", config);
    let host = String::from(&config.host);
    let port = config.port;

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
        &config.discord_client_id,
        &config.discord_client_secret,
        &format!("https://{}/discord/callback", &config.hostname),
        &config.discord_bot_token,
        &config.discord_server_id,
        &config.discord_channel_id,
    );

    let plex_client = PlexClient::new(
        &reqwest_client,
        &config.application_name,
        &format!("https://{}/plex/callback", &config.hostname),
    );

    let tautlli_client = TautulliClient::new(
        &reqwest_client.clone(),
        &config.tautulli_url,
        &config.tautulli_api_key,
    );

    let pool = initialize_db_pool(&config.database_url);
    let mut conn = pool.get().unwrap();
    run_migrations(&mut conn).unwrap();
    register_metadata(&discord_client).await.unwrap();

    log::info!("Starting listener on {}:{}", &host, &port);
    HttpServer::new(move || {
        App::new()
            .configure(config::config_app)
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
                    Key::from(config.session_secret_key.as_bytes()),
                )
                .cookie_secure(true)
                .build(),
            )
    })
    .bind((host, port))?
    .run()
    .await
}

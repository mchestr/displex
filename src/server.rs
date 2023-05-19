use std::time::Duration;

use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    body::MessageBody,
    cookie::Key,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    middleware::Logger,
    rt,
    web::{self},
    App, Error, HttpServer,
};

use crate::{
    config::ServerArgs,
    db::{self, run_migrations},
    discord::client::DiscordClient,
    handlers,
    plex::client::PlexClient,
    tautulli::client::TautulliClient,
};
use db::initialize_db_pool;
use reqwest::header::HeaderValue;

pub fn new(
    config: ServerArgs,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<impl MessageBody>,
        Config = (),
        InitError = (),
        Error = Error,
    >,
> {
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
    // register_metadata(&discord_client).await.unwrap();

    log::info!("Starting listener on {}:{}", &host, &port);

    App::new()
        .configure(handlers::configure)
        .app_data(web::Data::new(plex_client))
        .app_data(web::Data::new(discord_client))
        .app_data(web::Data::new(config.clone()))
        .app_data(web::Data::new(pool))
        .app_data(web::Data::new(tautlli_client))
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
}

pub fn run(config: ServerArgs) {
    rt::System::new()
        .block_on(
            HttpServer::new(move || new(config.clone()))
                .bind(("127.0.0.1", 8080))
                .unwrap()
                .workers(1)
                .run(),
        )
        .unwrap()
}

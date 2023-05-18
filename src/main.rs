use std::{env, time::Duration};

use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{
    cookie::Key,
    error::{ErrorBadRequest, ErrorInternalServerError, ErrorUnauthorized},
    get,
    middleware::Logger,
    web::{self, Redirect},
    App, HttpResponse, HttpServer, Responder, Result,
};

use config::Config;
use db::initialize_db_pool;
use dotenv::dotenv;
use oauth2::TokenResponse;
use reqwest::header::HeaderValue;
use serde::Deserialize;

use crate::{
    db::{discord_tokens::DiscordToken, plex_tokens::PlexToken, DbPool},
    discord::client::DiscordClient,
    plex::client::PlexClient,
};

mod config;
mod db;
mod discord;
mod plex;
mod schema;
mod session;

// 1. Initial route that will ask user to authorize bot for their discord account
#[get("/discord/linked-role")]
async fn discord_linked_role(
    discord_client: web::Data<DiscordClient>,
    session: Session,
) -> Result<impl Responder> {
    let (url, state) = discord_client.authorize_url();
    session.insert(session::DISCORD_STATE, state.secret())?;

    Ok(Redirect::to(url.to_string()))
}

#[derive(Debug, Deserialize)]
pub struct DiscordRedirectQueryParams {
    pub code: String,
    pub state: String,
}

// 2. URL Discord will redirect user to after granting bot access
#[get("/discord/callback")]
async fn discord_callback(
    plex_client: web::Data<PlexClient>,
    qs: web::Query<DiscordRedirectQueryParams>,
    session: Session,
) -> Result<impl Responder> {
    let session_token = session
        .get::<String>(session::DISCORD_STATE)?
        .expect("invalid state");
    if session_token != qs.state {
        log::info!("session state does not match query parameters");
        return Err(ErrorBadRequest("invalid state"));
    }
    session.insert(session::DISCORD_CODE, &qs.code)?;

    let pin = plex_client.get_pin().await.map_err(|err| {
        log::error!("{}", err);
        ErrorInternalServerError("something bad happened")
    })?;
    let url = plex_client
        .generate_auth_url(pin.id, &pin.code)
        .await
        .map_err(|err| {
            log::error!("{}", err);
            ErrorInternalServerError("something bad happened")
        })?;

    Ok(Redirect::to(String::from(url)))
}

#[derive(Debug, Deserialize)]
pub struct PlexRedirectQueryParams {
    pub id: u64,
    pub code: String,
}

// 3. Callback plex will redirect to after user grants access
#[get("/plex/callback")]
async fn plex_callback(
    config: web::Data<Config>,
    discord_client: web::Data<DiscordClient>,
    plex_client: web::Data<PlexClient>,
    pool: web::Data<DbPool>,
    qs: web::Query<PlexRedirectQueryParams>,
    session: Session,
) -> Result<impl Responder> {
    let resp = plex_client
        .pin_claim(qs.id, &qs.code)
        .await
        .map_err(|err| {
            log::error!("{}", err);
            ErrorInternalServerError("something bad happened")
        })?;

    let discord_token = session
        .get::<String>(session::DISCORD_CODE)?
        .expect("invalid discord token");

    match plex_client
        .get_devices(&resp.auth_token)
        .await
        .map_err(|err| {
            log::error!("{}", err);
            ErrorInternalServerError("something bad happened")
        })?
        .iter()
        .find(|&d| d.client_identifier == config.plex_server_id)
    {
        Some(_) => {
            let token = discord_client.token(&discord_token).await.map_err(|err| {
                log::error!("discord_client.token: {}", err);
                ErrorInternalServerError("something bad happened")
            })?;

            let d_access_token = token.access_token().secret();
            let d_access_token_clone = d_access_token.clone();
            let t = token.clone();

            let discord_user = discord_client.user(&d_access_token).await.map_err(|err| {
                log::error!("discord_client.user: {}", err);
                ErrorInternalServerError("something bad happened")
            })?;

            let plex_user = plex_client.user(&resp.auth_token).await.map_err(|err| {
                log::error!("plex_client.user {}", err);
                ErrorInternalServerError("something bad happened")
            })?;

            let _ = web::block(move || {
                // note that obtaining a connection from the pool is also potentially blocking
                let mut conn = pool.get()?;

                conn.build_transaction().run(|conn| {
                    db::plex_tokens::insert_new_token(
                        conn,
                        PlexToken {
                            id: plex_user.id.to_string(),
                            username: plex_user.username,
                            access_token: resp.auth_token,
                        },
                    )?;

                    db::discord_tokens::insert_new_token(
                        conn,
                        DiscordToken {
                            id: discord_user.id,
                            username: discord_user.username,
                            access_token: d_access_token_clone.into(),
                            token_type: "bearer".into(),
                            refresh_token: t
                                .refresh_token()
                                .expect("expecting refresh token")
                                .secret()
                                .into(),
                            scopes: t.scopes().map_or("".into(), |d| {
                                d.iter().map(|i| i.to_string() + ",").collect()
                            }),
                        },
                    )
                })
            })
            .await?
            // map diesel query errors to a 500 error response
            .map_err(|err| {
                log::error!("db save: {}", err);
                ErrorInternalServerError("something bad happened")
            })?;

            discord_client
                .link_application(&d_access_token)
                .await
                .map_err(|err| {
                    log::error!("discord_client.link_application: {}", err);
                    ErrorInternalServerError("something bad happened")
                })?;
            Ok(HttpResponse::Ok()
                .body("Successfully linked! You can go back to Discord now and close this tab."))
        }
        None => Err(ErrorUnauthorized("unauthorized user")),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let config = config::Config::init();
    let cfg = config.clone();
    let port = config.port;

    let mut default_headers = reqwest::header::HeaderMap::new();
    default_headers.append("Accept", HeaderValue::from_static("application/json"));

    let reqwest_client = reqwest::ClientBuilder::new()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .pool_idle_timeout(Duration::from_secs(90))
        .default_headers(default_headers)
        .build()
        .unwrap();

    let pool = initialize_db_pool();

    env::set_var("RUST_LOG", "info,actix_web=debug,actix_server=info");
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    HttpServer::new(move || {
        App::new()
            .service(discord_callback)
            .service(discord_linked_role)
            .service(plex_callback)
            .app_data(web::Data::new(plex::client::PlexClient::new_with_client(
                reqwest_client.clone(),
                &config.plex_client_id,
                &format!("https://{}/plex/callback", &config.hostname),
            )))
            .app_data(web::Data::new(DiscordClient::new(
                reqwest_client.clone(),
                &config.discord_client_id,
                &config.discord_client_secret,
                &format!("https://{}/discord/callback", &config.hostname),
            )))
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(pool.clone()))
            .wrap(Logger::default())
            .wrap(
                // create cookie based session middleware
                SessionMiddleware::builder(
                    CookieSessionStore::default(),
                    Key::from(&config.session_secret_key.as_bytes()),
                )
                .cookie_secure(true)
                .build(),
            )
    })
    .bind((cfg.host, port))?
    .run()
    .await
}

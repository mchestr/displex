use anyhow::anyhow;
use std::time::Duration;

use axum::{
    extract::{
        Query,
        State,
    },
    response::{
        IntoResponse,
        Redirect,
    },
    routing::get,
    Router,
};

use oauth2::TokenResponse;
use sea_orm::TransactionTrait;
use serde::Deserialize;
use tower_cookies::Cookies;

use crate::{
    errors::DisplexError,
    server::{
        axum::DisplexState,
        cookies::{
            get_cookie_data,
            set_cookie_data,
            CookieData,
            Role,
        },
    },
    services::{
        discord_token::resolver::CreateDiscordTokenResult,
        discord_user::resolver::{
            CreateDiscordUserErrorVariant,
            CreateDiscordUserResult,
        },
    },
};

#[derive(Deserialize)]
struct DiscordAuthQueryParams {
    pub next: Option<String>,
}

async fn discord_auth(
    cookies: Cookies,
    State(state): State<DisplexState>,
    query_string: Query<DiscordAuthQueryParams>,
) -> Result<impl IntoResponse, DisplexError> {
    let query_param = match &query_string.next {
        Some(next) => format!("?next={}", next),
        None => String::new(),
    };

    let (url, persist_state) = state.services.discord_service.authorize_url(&format!(
        "https://{}/auth/discord/callback{}",
        &state.config.http.hostname, query_param
    ));

    let persist_state = String::from(persist_state.secret());
    set_cookie_data(
        &state.config.session.secret_key,
        &cookies,
        &CookieData {
            discord_state: Some(persist_state),
            ..Default::default()
        },
    )?;

    Ok(Redirect::to(url.as_str()))
}

#[derive(Deserialize)]
struct CallbackQueryParams {
    pub code: String,
    pub state: String,
    pub next: Option<String>,
}

async fn callback(
    cookies: Cookies,
    State(state): State<DisplexState>,
    query_string: Query<CallbackQueryParams>,
) -> Result<impl IntoResponse, DisplexError> {
    let mut cookie_data = get_cookie_data(&state.config.session.secret_key, &cookies)?;

    let discord_state = cookie_data.discord_state.as_mut().unwrap();
    verify_state(discord_state, &query_string.state)?;

    let params = match &query_string.next {
        Some(next) => format!("?next={}", next),
        None => String::new(),
    };
    let token = state
        .services
        .discord_service
        .token(
            &query_string.code,
            &format!(
                "https://{}/auth/discord/callback{}",
                &state.config.http.hostname, params
            ),
        )
        .await?;
    let access_token = String::from(token.access_token().secret());
    let discord_user = state.services.discord_service.user(&access_token).await?;

    state
        .services
        .db
        .transaction::<_, (), DisplexError>(|txn| {
            let discord_user = discord_user.clone();
            Box::pin(async move {
                let result = state
                    .services
                    .discord_users_service
                    .create_with_conn(&discord_user.id, &discord_user.username, txn)
                    .await?;
                match result {
                    CreateDiscordUserResult::Error(err) => match err.error {
                        CreateDiscordUserErrorVariant::InternalError => {
                            Err(DisplexError(anyhow!("internal error")))
                        }
                    },
                    _ => Ok(()),
                }?;

                let scopes: String = token.scopes().map_or("".into(), |d| {
                    d.iter().map(|i| i.to_string() + ",").collect()
                });
                let result = state
                    .services
                    .discord_tokens_service
                    .create_with_conn(
                        token.access_token().secret(),
                        token
                            .refresh_token()
                            .expect("expecting refresh token")
                            .secret(),
                        &(chrono::Utc::now()
                            + chrono::Duration::seconds(
                                token
                                    .expires_in()
                                    .unwrap_or(Duration::from_secs(1800))
                                    .as_secs() as i64,
                            )),
                        &scopes,
                        &discord_user.id,
                        txn,
                    )
                    .await?;
                match result {
                    CreateDiscordTokenResult::Error(_) => {
                        Err(DisplexError(anyhow!("internal error")))
                    }
                    _ => Ok(()),
                }?;
                Ok(())
            })
        })
        .await
        .expect("failed to save discord user info to database");

    let discord_user_id = discord_user.id;
    cookie_data.discord_user = Some(discord_user_id.clone());
    if state
        .config
        .api
        .admin_discord_ids
        .contains(&discord_user_id.clone())
    {
        cookie_data.role = Role::Admin;
    } else {
        cookie_data.role = Role::User;
    }
    set_cookie_data(&state.config.session.secret_key, &cookies, &cookie_data)?;

    let mut url = String::from("/");
    if query_string.next.is_some() {
        let pin = state.services.plex_service.get_pin().await?;
        url = state
            .services
            .plex_service
            .generate_auth_url(pin.id, &pin.code)
            .await?;
    }

    Ok(Redirect::to(url.as_str()))
}

#[tracing::instrument]
fn verify_state(session_state: &str, query_string_state: &str) -> Result<(), anyhow::Error> {
    if session_state != query_string_state {
        tracing::info!("session state does not match query parameters");
        anyhow::bail!("invalid state")
    }
    Ok(())
}

pub fn routes() -> Router<DisplexState> {
    Router::new()
        .route("/auth/discord", get(discord_auth))
        .route("/auth/discord/callback", get(callback))
}

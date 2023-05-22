use std::time::Duration;

use anyhow::anyhow;
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
use axum_sessions::extractors::ReadableSession;
use oauth2::TokenResponse;
use serde::Deserialize;

use crate::{
    db::{
        self,
        discord::{
            NewDiscordToken,
            NewDiscordUser,
        },
        plex::{
            NewPlexToken,
            NewPlexUser,
        },
    },
    discord::models::{
        ApplicationMetadata,
        ApplicationMetadataUpdate,
    },
    errors::DisplexError,
    server::axum::DisplexState,
    session::DISCORD_CODE,
    tautulli::models::QueryDays,
};

#[derive(Deserialize)]
struct CallbackQueryParams {
    pub id: u64,
    pub code: String,
}

async fn callback(
    session: ReadableSession,
    State(state): State<DisplexState>,
    query_string: Query<CallbackQueryParams>,
) -> Result<impl IntoResponse, DisplexError> {
    let resp = state
        .plex_client
        .pin_claim(query_string.id, &query_string.code)
        .await?;

    let discord_token = session
        .get::<String>(DISCORD_CODE)
        .ok_or_else(|| anyhow!("no code found for session"))?;

    let is_subscriber = state
        .plex_client
        .get_devices(&resp.auth_token)
        .await?
        .iter()
        .any(|d| d.client_identifier == state.config.plex.plex_server_id);

    let token = state.discord_oauth_client.token(&discord_token).await?;

    let d_access_token = String::from(token.access_token().secret());
    let discord_user = state.discord_client.user(&d_access_token).await?;

    let plex_user = state.plex_client.user(&resp.auth_token).await?;

    tracing::info!(
        "{} is a subscriber: {}",
        discord_user.username,
        is_subscriber
    );

    let mut transaction = state.db.begin().await?;
    let discord_user = db::discord::insert_user(
        &mut transaction,
        NewDiscordUser {
            id: discord_user.id,
            username: discord_user.username,
        },
    )
    .await?;
    tracing::debug!("inserted discord user: {:?}", discord_user);

    let discord_token = db::discord::insert_token(
        &mut transaction,
        NewDiscordToken {
            access_token: token.access_token().secret().into(),
            refresh_token: token
                .refresh_token()
                .expect("expecting refresh token")
                .secret()
                .into(),
            scopes: token.scopes().map_or("".into(), |d| {
                d.iter().map(|i| i.to_string() + ",").collect()
            }),
            expires_at: chrono::Utc::now()
                + chrono::Duration::seconds(
                    token
                        .expires_in()
                        .unwrap_or(Duration::from_secs(1800))
                        .as_secs() as i64,
                ),
            discord_user_id: String::from(&discord_user.id),
        },
    )
    .await?;
    tracing::debug!("inserted discord token: {:?}", discord_token);

    let plex_user = db::plex::insert_user(
        &mut transaction,
        NewPlexUser {
            id: plex_user.id,
            username: plex_user.username,
            discord_user_id: String::from(&discord_user.id),
            is_subscriber,
        },
    )
    .await?;
    tracing::debug!("inserted plex user: {:?}", plex_user);

    let plex_token = db::plex::insert_token(
        &mut transaction,
        NewPlexToken {
            access_token: resp.auth_token,
            plex_user_id: plex_user.id,
        },
    )
    .await?;
    tracing::debug!("inserted plex token: {:?}", plex_token);

    transaction.commit().await?;

    let mut data = ApplicationMetadataUpdate {
        platform_name: String::from(&state.config.application_name),
        metadata: ApplicationMetadata {
            is_subscriber,
            ..Default::default()
        },
        ..Default::default()
    };
    if is_subscriber {
        let watch_stats = state
            .tautulli_client
            .get_user_watch_time_stats(plex_user.id, Some(true), Some(QueryDays::Total))
            .await?;

        if let Some(latest) = watch_stats.get(0) {
            data.metadata.total_watches = latest.total_plays;
            data.metadata.hours_watched = latest.total_time / 3600;
        };
    };

    state
        .discord_client
        .link_application(&d_access_token, data)
        .await?;
    Ok(Redirect::to(&format!(
        "discord://-/channels/{}/@home",
        state.config.discord.discord_server_id
    )))
}

pub fn routes() -> Router<DisplexState> {
    Router::new().route("/plex/callback", get(callback))
}

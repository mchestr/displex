use actix_session::Session;
use actix_web::{
    error::{ErrorBadRequest, ErrorInternalServerError},
    web::{self, Redirect},
    Responder, Result,
};
use serde::Deserialize;

use crate::{discord::client::DiscordClient, plex::client::PlexClient, session};

pub async fn linked_role(
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

pub async fn callback(
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

    Ok(Redirect::to(url))
}

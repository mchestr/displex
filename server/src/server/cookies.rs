use anyhow::anyhow;
use async_graphql::{
    Context,
    Enum,
};
use std::time::Duration;

use cookie::{
    time::OffsetDateTime,
    Cookie,
    Key,
    SameSite,
};
use serde::{
    Deserialize,
    Serialize,
};
use tower_cookies::Cookies;

pub const DISPLEX_COOKIE: &str = "displex";

#[derive(Enum, Debug, Deserialize, Serialize, Default, Copy, Clone, Eq, PartialEq)]
pub enum Role {
    Admin,
    User,
    #[default]
    Anonymous,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct CookieData {
    #[serde(rename = "ds")]
    pub discord_state: Option<String>,
    #[serde(rename = "du")]
    pub discord_user: Option<String>,
    #[serde(rename = "pu")]
    pub plex_user: Option<i64>,
    #[serde(rename = "r", default)]
    pub role: Role,
}

impl From<&CookieData> for Cookie<'_> {
    fn from(value: &CookieData) -> Self {
        Cookie::build((DISPLEX_COOKIE, serde_json::to_string(value).unwrap()))
            .same_site(SameSite::Lax)
            .http_only(true)
            .secure(true)
            .path("/")
            // 6 months
            .expires(OffsetDateTime::now_utc() + Duration::from_secs(60 * 60 * 24 * 30 * 6))
            .build()
    }
}

pub fn get_cookie_data(secret_key: &str, cookies: &Cookies) -> anyhow::Result<CookieData> {
    let key = Key::from(secret_key.as_bytes());
    let signed = cookies.signed(&key);
    let cookie_data = signed
        .get(DISPLEX_COOKIE)
        .ok_or_else(|| anyhow!("session state is invalid"))?;
    serde_json::from_str(cookie_data.value_trimmed()).map_err(|err| anyhow!(err))
}

pub fn set_cookie_data(
    secret_key: &str,
    cookies: &Cookies,
    cookie_data: &CookieData,
) -> anyhow::Result<()> {
    let key = Key::from(secret_key.as_bytes());
    let signed = cookies.signed(&key);
    signed.add(cookie_data.into());
    Ok(())
}

pub fn verify_role(ctx: &Context<'_>, expected_role: Role) -> anyhow::Result<()> {
    let role = match ctx.data::<CookieData>() {
        Ok(cookie) => &cookie.role,
        Err(_) => &Role::Anonymous,
    };
    tracing::info!("verifying role: {role:?} >= {expected_role:?}");
    match expected_role {
        Role::Admin => {
            if [Role::Admin].contains(role) {
                Ok(())
            } else {
                Err(anyhow!("unauthorized"))
            }
        }
        Role::User => {
            if [Role::Admin, Role::User].contains(role) {
                Ok(())
            } else {
                Err(anyhow!("unauthorized"))
            }
        }
        _ => Err(anyhow!("unauthorized")),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn serde_works() {
        let json = "{\"ds\":\"ds\",\"du\":\"du\",\"pu\":1}";
        let data: CookieData = serde_json::from_str(json).unwrap();
        assert_eq!(data.discord_state, Some(String::from("ds")));
        assert_eq!(data.discord_user, Some(String::from("du")));
        assert_eq!(data.plex_user, Some(1));
        assert_eq!(data.role, Role::Anonymous);
    }

    #[test]
    fn serde_works_role() {
        let json = "{\"ds\":\"ds\",\"du\":\"du\",\"pu\":1,\"r\":\"Admin\"}";
        let data: CookieData = serde_json::from_str(json).unwrap();
        assert_eq!(data.discord_state, Some(String::from("ds")));
        assert_eq!(data.discord_user, Some(String::from("du")));
        assert_eq!(data.plex_user, Some(1));
        assert_eq!(data.role, Role::Admin);
    }
}

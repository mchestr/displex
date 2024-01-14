use anyhow::anyhow;
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

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct CookieData {
    #[serde(rename = "ds")]
    pub discord_state: Option<String>,
    #[serde(rename = "du")]
    pub discord_user: Option<String>,
    #[serde(rename = "pu")]
    pub plex_user: Option<String>,
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

#[cfg(test)]
mod test {
    use super::CookieData;

    #[test]
    fn serde_works() {
        let json = "{\"ds\":\"ds\",\"du\":\"du\",\"pu\":\"pu\"}";
        let data: CookieData = serde_json::from_str(json).unwrap();
        assert_eq!(data.discord_state, Some(String::from("ds")));
        assert_eq!(data.discord_user, Some(String::from("du")));
        assert_eq!(data.plex_user, Some(String::from("pu")));
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct DiscordMetaDataPush {
    pub platform_name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub global_name: Option<String>,
    pub avatar: Option<String>,
    pub discriminator: String,
    pub public_flags: i64,
    pub flags: i64,
    pub banner: Option<String>,
    pub banner_color: Option<i64>,
    pub accent_color: Option<i64>,
    pub locale: String,
    pub mfa_enabled: bool,
    pub verified: Option<bool>,
    pub premium_type: i64,
}

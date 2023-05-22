use chrono::Utc;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewDiscordToken {
    pub access_token: String,
    pub refresh_token: String,
    pub scopes: String,
    pub expires_at: chrono::DateTime<Utc>,
    pub discord_user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordToken {
    pub access_token: String,
    pub refresh_token: String,
    pub scopes: String,
    pub expires_at: chrono::DateTime<Utc>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub discord_user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewDiscordUser {
    pub id: String,
    pub username: String,
}

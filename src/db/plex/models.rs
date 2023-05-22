use chrono::Utc;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlexToken {
    pub access_token: String,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub plex_user_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPlexToken {
    pub access_token: String,
    pub plex_user_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlexUser {
    pub id: i64,
    pub username: String,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub discord_user_id: String,
    pub is_subscriber: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPlexUser {
    pub id: i64,
    pub username: String,
    pub discord_user_id: String,
    pub is_subscriber: bool,
}

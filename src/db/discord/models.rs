use chrono::Utc;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::{discord_tokens, discord_users};

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = discord_tokens)]
pub struct NewDiscordToken {
    pub access_token: String,
    pub refresh_token: String,
    pub scopes: String,
    pub expires_at: chrono::DateTime<Utc>,
    pub discord_user_id: String,
}

#[derive(Associations, Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(belongs_to(DiscordUser))]
#[diesel(table_name = discord_tokens)]
pub struct DiscordToken {
    pub access_token: String,
    pub refresh_token: String,
    pub scopes: String,
    pub expires_at: chrono::DateTime<Utc>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub discord_user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Identifiable, Queryable, Selectable)]
#[diesel(table_name = discord_users)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = discord_users)]
pub struct NewDiscordUser {
    pub id: String,
    pub username: String,
}

use diesel::prelude::*;
use serde::{
    Deserialize,
    Serialize,
};

use crate::db::discord::DiscordUser;

use crate::schema::{
    plex_tokens,
    plex_users,
};

#[derive(Associations, Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(belongs_to(PlexUser))]
#[diesel(table_name = plex_tokens)]
pub struct PlexToken {
    pub access_token: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub plex_user_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = plex_tokens)]
pub struct NewPlexToken {
    pub access_token: String,
    pub plex_user_id: i64,
}

#[derive(
    Associations, Debug, Clone, Serialize, Deserialize, Identifiable, Queryable, Selectable,
)]
#[diesel(belongs_to(DiscordUser))]
#[diesel(table_name = plex_users)]
pub struct PlexUser {
    pub id: i64,
    pub username: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub discord_user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = plex_users)]
pub struct NewPlexUser {
    pub id: i64,
    pub username: String,
    pub discord_user_id: String,
}

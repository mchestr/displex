use anyhow::Result;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::discord_tokens;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = discord_tokens)]
pub struct DiscordToken {
    pub id: String,
    pub username: String,
    pub access_token: String,
    pub token_type: String,
    pub refresh_token: String,
    pub scopes: String,
}

/// Run query using Diesel to insert a new database row and return the result.
pub fn insert_new_token(conn: &mut PgConnection, token: DiscordToken) -> Result<DiscordToken> {
    // It is common when using Diesel with Actix Web to import schema-related
    // modules inside a function's scope (rather than the normal module's scope)
    // to prevent import collisions and namespace pollution.
    use crate::schema::discord_tokens::dsl::*;

    diesel::insert_into(discord_tokens)
        .values(&token)
        .execute(conn)?;

    Ok(token)
}

use anyhow::Result;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::plex_tokens;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = plex_tokens)]
pub struct PlexToken {
    pub id: String,
    pub username: String,
    pub access_token: String,
}

/// Run query using Diesel to insert a new database row and return the result.
pub fn insert_new_token(conn: &mut PgConnection, token: PlexToken) -> Result<PlexToken> {
    // It is common when using Diesel with Actix Web to import schema-related
    // modules inside a function's scope (rather than the normal module's scope)
    // to prevent import collisions and namespace pollution.
    use crate::schema::plex_tokens::dsl::*;

    diesel::insert_into(plex_tokens)
        .values(&token)
        .execute(conn)?;

    Ok(token)
}

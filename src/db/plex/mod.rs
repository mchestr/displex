use anyhow::Result;
use diesel::{
    prelude::*,
    PgConnection,
};

pub use self::models::*;

mod models;

pub fn insert_token(conn: &mut PgConnection, new: NewPlexToken) -> Result<PlexToken> {
    use crate::schema::plex_tokens::dsl::*;
    log::debug!("inserting record: {:?}", new);

    let token = diesel::insert_into(plex_tokens)
        .values(&new)
        .on_conflict(access_token)
        // do_nothing will result in no data returned from database. Do a dummy update instead.
        .do_update()
        .set(access_token.eq(&new.access_token))
        .get_result(conn)?;

    Ok(token)
}

pub fn insert_user(conn: &mut PgConnection, new: NewPlexUser) -> Result<PlexUser> {
    use crate::schema::plex_users::dsl::*;
    log::debug!("inserting record: {:?}", new);

    let token = diesel::insert_into(plex_users)
        .values(&new)
        .on_conflict(id)
        .do_update()
        .set(username.eq(&new.username))
        .get_result(conn)?;

    Ok(token)
}

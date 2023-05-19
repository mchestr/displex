use anyhow::Result;
use diesel::{prelude::*, PgConnection};

mod models;

pub use models::*;

pub fn insert_token(conn: &mut PgConnection, new: NewDiscordToken) -> Result<DiscordToken> {
    use crate::schema::discord_tokens::dsl::*;
    log::debug!("inserting record: {:?}", new);

    let token = diesel::insert_into(discord_tokens)
        .values(&new)
        .on_conflict(access_token)
        .do_update()
        .set(refresh_token.eq(&new.refresh_token))
        .get_result(conn)?;

    Ok(token)
}

pub fn insert_user(conn: &mut PgConnection, new: NewDiscordUser) -> Result<DiscordUser> {
    use crate::schema::discord_users::dsl::*;
    log::debug!("inserting record: {:?}", new);

    let token = diesel::insert_into(discord_users)
        .values(&new)
        .on_conflict(id)
        .do_update()
        .set(username.eq(&new.username))
        .get_result(conn)?;

    Ok(token)
}

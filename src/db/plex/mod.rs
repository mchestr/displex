use anyhow::Result;
use sqlx::PgConnection;

pub use self::models::*;

mod models;

pub async fn insert_token(conn: &mut PgConnection, new: NewPlexToken) -> Result<PlexToken> {
    Ok(sqlx::query_as!(
        PlexToken,
        // language=PostgresSQL
        r#"insert into "plex_tokens" (access_token, plex_user_id) values ($1, $2) 
           on conflict (access_token) do nothing
           returning *"#,
        new.access_token,
        new.plex_user_id
    )
    .fetch_one(conn)
    .await?)
}

pub async fn insert_user(conn: &mut PgConnection, new: NewPlexUser) -> Result<PlexUser> {
    Ok(sqlx::query_as!(
        PlexUser,
        // language=PostgresSQL
        r#"insert into "plex_users" (id, username, discord_user_id, is_subscriber) values ($1, $2, $3, $4) 
           on conflict (id) do update 
           set username = excluded.username 
           returning *"#,
        new.id, new.username, new.discord_user_id, new.is_subscriber
    ).fetch_one(conn)
    .await?)
}

use anyhow::Result;

mod models;

pub use models::*;
use sqlx::PgConnection;

pub async fn insert_token(conn: &mut PgConnection, new: NewDiscordToken) -> Result<DiscordToken> {
    Ok(sqlx::query_as!(
        DiscordToken,
        // language=PostgresSQL
        r#"insert into "discord_tokens" (access_token, refresh_token, scopes, expires_at, discord_user_id) values ($1, $2, $3, $4, $5) 
           on conflict (access_token) do update
           set refresh_token = excluded.refresh_token 
           returning *"#,
        new.access_token,
        new.refresh_token,
        new.scopes,
        new.expires_at,
        new.discord_user_id,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn insert_user(conn: &mut PgConnection, new: NewDiscordUser) -> Result<DiscordUser> {
    Ok(sqlx::query_as!(
        DiscordUser,
        // language=PostgresSQL
        r#"insert into "discord_users" (id, username) values ($1, $2) 
           on conflict (id) do update 
           set username = excluded.username 
           returning *"#,
        new.id,
        new.username,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn get_latest_token(conn: &mut PgConnection, user_id: &str) -> Result<DiscordToken> {
    Ok(sqlx::query_as!(
        DiscordToken,
        r#"select * from discord_tokens where discord_user_id = $1 order by expires_at desc limit 1"#,
        user_id,
    )
    .fetch_one(conn)
    .await?)
}

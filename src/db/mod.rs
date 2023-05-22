use anyhow::Result;
use sqlx::{
    postgres::PgPoolOptions,
    PgPool,
};

use self::{
    discord::DiscordUser,
    plex::PlexUser,
};

pub mod discord;
pub mod plex;

pub async fn initialize_db_pool(database_url: &str) -> Result<PgPool> {
    Ok(PgPoolOptions::new()
        // The default connection limit for a Postgres server is 100 connections, minus 3 for
        // superusers. Since we're using the default superuser we don't have to worry about
        // this too much, although we should leave some connections available for manual
        // access.
        //
        // If you're deploying your application with multiple replicas, then the total
        // across all replicas should not exceed the Postgres connection limit.
        .max_connections(50)
        .connect(&database_url)
        .await
        .unwrap())
}

pub async fn run_migrations(db: &PgPool) -> Result<()> {
    Ok(sqlx::migrate!().run(db).await?)
}

//    pub id: i64,
// pub username: String,
// pub created_at: chrono::DateTime<Utc>,
// pub updated_at: chrono::DateTime<Utc>,
// pub discord_user_id: String,
// pub is_subscriber: bool,

pub async fn list_users<'e, E>(conn: E) -> Result<Vec<(DiscordUser, PlexUser)>>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    let users = sqlx::query!(
        r#"select d.id as did, d.username as du, d.created_at as dca, d.updated_at as dua, 
           p.id, p.username, p.created_at, p.updated_at, p.discord_user_id, p.is_subscriber 
           from discord_users as d 
           inner join plex_users as p on d.id = p.discord_user_id 
           where p.is_subscriber"#
    )
    .fetch_all(conn)
    .await?;

    Ok(users
        .into_iter()
        .map(|r| {
            (
                DiscordUser {
                    id: r.did.unwrap(),
                    username: r.du.unwrap(),
                    created_at: r.created_at.unwrap(),
                    updated_at: r.updated_at.unwrap(),
                },
                PlexUser {
                    id: r.id.unwrap(),
                    username: r.username.unwrap(),
                    created_at: r.created_at.unwrap(),
                    updated_at: r.updated_at.unwrap(),
                    discord_user_id: r.discord_user_id.unwrap(),
                    is_subscriber: r.is_subscriber.unwrap(),
                },
            )
        })
        .collect())
}

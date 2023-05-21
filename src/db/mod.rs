

use anyhow::Result;
use diesel::{
    prelude::*,
    PgConnection,
    QueryDsl,
};

pub mod discord;
pub mod plex;

#[cfg(feature = "actix-web")]
pub type DbPool = r2d2::Pool<r2d2::ConnectionManager<PgConnection>>;
#[cfg(feature = "axum")]
pub type DbPool = deadpool_diesel::postgres::Pool;

use diesel_migrations::{
    embed_migrations,
    EmbeddedMigrations,
    MigrationHarness,
};

use crate::schema::{
    discord_users,
    plex_users::{
        self,
        is_subscriber,
    },
};

use self::{
    discord::DiscordUser,
    plex::PlexUser,
};
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[cfg(feature = "actix-web")]
pub fn initialize_db_pool(database_url: &str) -> DbPool {
    let manager = diesel::r2d2::ConnectionManager::<PgConnection>::new(database_url);
    diesel::r2d2::Pool::builder()
        .build(manager)
        .expect("unable to connect to postgres")
}

#[cfg(feature = "axum")]
pub fn initialize_db_pool(database_url: &str) -> Result<DbPool> {
    let manager =
        deadpool_diesel::postgres::Manager::new(database_url, deadpool_diesel::Runtime::Tokio1);
    let pool = deadpool_diesel::postgres::Pool::builder(manager).build()?;
    Ok(pool)
}

#[cfg(features = "actix-web")]
pub fn run_migrations(
    connection: &mut impl MigrationHarness<Pg>,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    // This will run the necessary migrations.
    //
    // See the documentation for `MigrationHarness` for
    // all available methods.
    connection.run_pending_migrations(MIGRATIONS)?;

    Ok(())
}

#[cfg(feature = "axum")]
pub async fn run_migrations(pool: &DbPool) -> Result<()> {
    use anyhow::anyhow;

    let conn = pool.get().await.unwrap();
    conn.interact(|conn| conn.run_pending_migrations(MIGRATIONS).map(|_| ()))
        .await
        .map_err(|_| anyhow!("failed to get conection"))?
        .map_err(|_| anyhow!("failed to migrate"))?;
    Ok(())
}

pub fn list_users(conn: &mut PgConnection) -> Result<Vec<(DiscordUser, PlexUser)>> {
    let users = discord_users::table
        .inner_join(plex_users::table)
        .select((DiscordUser::as_select(), PlexUser::as_select()))
        .filter(is_subscriber)
        .load::<(DiscordUser, PlexUser)>(conn)?;

    Ok(users)
}

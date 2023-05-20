use std::error::Error;

use anyhow::Result;
use diesel::{
    pg::Pg,
    prelude::*,
    r2d2,
    PgConnection,
    QueryDsl,
};

pub mod discord;
pub mod plex;

pub type DbPool = r2d2::Pool<r2d2::ConnectionManager<PgConnection>>;

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

pub fn initialize_db_pool(database_url: &str) -> DbPool {
    let manager = r2d2::ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("unable to connect to postgres")
}

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

pub fn list_users(conn: &mut PgConnection) -> Result<Vec<(DiscordUser, PlexUser)>> {
    let users = discord_users::table
        .inner_join(plex_users::table)
        .select((DiscordUser::as_select(), PlexUser::as_select()))
        .filter(is_subscriber)
        .load::<(DiscordUser, PlexUser)>(conn)?;

    Ok(users)
}

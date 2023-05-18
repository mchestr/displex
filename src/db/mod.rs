use diesel::{r2d2, PgConnection};

pub mod discord_tokens;
pub mod plex_tokens;

pub type DbPool = r2d2::Pool<r2d2::ConnectionManager<PgConnection>>;

/// Initialize database connection pool based on `DATABASE_URL` environment variable.
///
/// See more: <https://docs.rs/diesel/latest/diesel/r2d2/index.html>.
pub fn initialize_db_pool() -> DbPool {
    let conn_spec = std::env::var("DISPLEX_DATABASE_URL").expect("DATABASE_URL should be set");
    let manager = r2d2::ConnectionManager::<PgConnection>::new(conn_spec);
    r2d2::Pool::builder()
        .build(manager)
        .expect("unable to connect to postgres")
}

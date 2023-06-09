pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_discord_user;
mod m20230528_193818_create_discord_token;
mod m20230528_193838_create_plex_user;
mod m20230528_193841_create_plex_token;

pub use m20220101_000001_create_discord_user::DiscordUser;
pub use m20230528_193818_create_discord_token::DiscordToken;
pub use m20230528_193838_create_plex_user::PlexUser;
pub use m20230528_193841_create_plex_token::PlexToken;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_discord_user::Migration),
            Box::new(m20230528_193818_create_discord_token::Migration),
            Box::new(m20230528_193838_create_plex_user::Migration),
            Box::new(m20230528_193841_create_plex_token::Migration),
        ]
    }
}

pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_discord_user;
mod m20230528_193818_create_discord_token;
mod m20230528_193838_create_plex_user;
mod m20230528_193841_create_plex_token;
mod m20230930_035233_discord_user_is_active;
mod m20231007_195159_add_token_enum;
mod m20231007_222508_add_token_enum;

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
            Box::new(m20230930_035233_discord_user_is_active::Migration),
            Box::new(m20231007_195159_add_token_enum::Migration),
            Box::new(m20231007_222508_add_token_enum::Migration),
        ]
    }
}

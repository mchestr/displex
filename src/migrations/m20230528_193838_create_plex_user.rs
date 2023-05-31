use sea_orm_migration::prelude::*;

use super::DiscordUser;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PlexUser::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PlexUser::Id)
                            .integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PlexUser::Username).string().not_null())
                    .col(
                        ColumnDef::new(PlexUser::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PlexUser::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(ColumnDef::new(PlexUser::DiscordUserId).string().not_null())
                    .col(
                        ColumnDef::new(PlexUser::IsSubscriber)
                            .boolean()
                            .default(false),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-plex-user-discord_user_id")
                            .from(PlexUser::Table, PlexUser::DiscordUserId)
                            .to(DiscordUser::Table, DiscordUser::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PlexUser::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum PlexUser {
    Table,
    Id,
    Username,
    CreatedAt,
    UpdatedAt,
    DiscordUserId,
    IsSubscriber,
}

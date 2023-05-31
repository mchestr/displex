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
                    .table(DiscordToken::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DiscordToken::AccessToken)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(DiscordToken::RefreshToken)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DiscordToken::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(DiscordToken::Scopes).string().not_null())
                    .col(
                        ColumnDef::new(DiscordToken::DiscordUserId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DiscordToken::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DiscordToken::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-discord-token-discord_user_id")
                            .from(DiscordToken::Table, DiscordToken::DiscordUserId)
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
            .drop_table(Table::drop().table(DiscordToken::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden

#[derive(Iden)]
pub enum DiscordToken {
    Table,
    AccessToken,
    RefreshToken,
    ExpiresAt,
    DiscordUserId,
    Scopes,
    CreatedAt,
    UpdatedAt,
}

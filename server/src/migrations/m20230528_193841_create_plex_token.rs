use sea_orm_migration::prelude::*;

use super::PlexUser;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PlexToken::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PlexToken::AccessToken)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(PlexToken::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PlexToken::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(ColumnDef::new(PlexToken::PlexUserId).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-plex-token-plex_user_id")
                            .from(PlexToken::Table, PlexToken::PlexUserId)
                            .to(PlexUser::Table, PlexUser::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PlexToken::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum PlexToken {
    Table,
    AccessToken,
    CreatedAt,
    UpdatedAt,
    PlexUserId,
}

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DiscordUser::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DiscordUser::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(DiscordUser::Username).string().not_null())
                    .col(
                        ColumnDef::new(DiscordUser::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DiscordUser::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DiscordUser::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum DiscordUser {
    Table,
    Id,
    Username,
    CreatedAt,
    UpdatedAt,
}

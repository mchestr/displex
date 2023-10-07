use sea_orm_migration::prelude::*;

use super::DiscordToken;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(DiscordToken::Table)
                    .drop_column(Alias::new("status"))
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(DiscordToken::Table)
                    .add_column(
                        ColumnDef::new(Alias::new("status"))
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(DiscordToken::Table)
                    .drop_column(Alias::new("status"))
                    .to_owned(),
            )
            .await
    }
}

use async_graphql::{
    Enum,
    SimpleObject,
};
use sea_orm::entity::prelude::*;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(
    Debug, Enum, Copy, Eq, Deserialize, Clone, PartialEq, EnumIter, Serialize, DeriveActiveEnum,
)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
#[derive(Default)]
pub enum TokenStatus {
    #[sea_orm(num_value = 0)]
    #[default]
    Active,
    #[sea_orm(num_value = 1)]
    Revoked,
    #[sea_orm(num_value = 2)]
    Renewed,
    #[sea_orm(num_value = 3)]
    Expired,
}

#[derive(
    Clone, Debug, Default, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, SimpleObject,
)]
#[graphql(name = "DiscordToken")]
#[sea_orm(table_name = "discord_token")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub access_token: String,
    pub refresh_token: String,
    pub scopes: String,
    pub expires_at: DateTimeUtc,
    pub discord_user_id: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    pub status: TokenStatus,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::discord_user::Entity",
        from = "Column::DiscordUserId",
        to = "super::discord_user::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    DiscordUserId,
}

impl Related<super::discord_user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DiscordUserId.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

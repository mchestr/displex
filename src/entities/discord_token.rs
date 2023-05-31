use async_graphql::SimpleObject;
use sea_orm::entity::prelude::*;
use serde::{
    Deserialize,
    Serialize,
};

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

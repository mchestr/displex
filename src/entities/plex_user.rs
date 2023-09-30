use async_graphql::SimpleObject;
use sea_orm::entity::prelude::*;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, SimpleObject)]
#[graphql(name = "PlexUser")]
#[sea_orm(table_name = "plex_user")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub username: String,
    pub discord_user_id: String,
    pub is_subscriber: bool,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::plex_token::Entity")]
    PlexToken,
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

use async_graphql::SimpleObject;
use sea_orm::entity::prelude::*;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, SimpleObject)]
#[graphql(name = "DiscordUser")]
#[sea_orm(table_name = "discord_user")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub username: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::discord_token::Entity")]
    DiscordToken,
    #[sea_orm(has_many = "super::plex_user::Entity")]
    PlexUser,
}

impl Related<super::discord_token::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DiscordToken.def()
    }
}

impl Related<super::plex_user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PlexUser.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

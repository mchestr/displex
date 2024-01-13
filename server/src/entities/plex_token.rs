use async_graphql::SimpleObject;
use sea_orm::entity::prelude::*;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, SimpleObject)]
#[graphql(name = "PlexToken")]
#[sea_orm(table_name = "plex_token")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub access_token: String,
    pub plex_user_id: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::plex_user::Entity",
        from = "Column::PlexUserId",
        to = "super::plex_user::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    PlexUserId,
}

impl Related<super::plex_user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PlexUserId.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

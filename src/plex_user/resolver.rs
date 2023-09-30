use async_graphql::{
    Context,
    Enum,
    InputObject,
    Object,
    Result,
    SimpleObject,
    Union,
};

use chrono::Utc;
use sea_orm::{
    prelude::*,
    ActiveValue,
    QueryTrait,
};

use crate::entities::plex_user;

#[derive(Default)]
pub struct PlexUsersQuery;

#[Object]
impl PlexUsersQuery {
    async fn get_plex_user(
        &self,
        gql_ctx: &Context<'_>,
        input: GetPlexUserInput,
    ) -> Result<GetPlexUserResult> {
        gql_ctx
            .data_unchecked::<PlexUsersService>()
            .get(input.id)
            .await
    }
    async fn list_plex_users(
        &self,
        gql_ctx: &Context<'_>,
        input: ListPlexUserInput,
    ) -> Result<Vec<plex_user::Model>> {
        gql_ctx
            .data_unchecked::<PlexUsersService>()
            .list(input.discord_user_id)
            .await
    }
}

#[derive(Default)]
pub struct PlexUsersMutation;

#[Object]
impl PlexUsersMutation {
    async fn create_plex_user(
        &self,
        gql_ctx: &Context<'_>,
        input: CreatePlexUserInput,
    ) -> Result<CreatePlexUserResult> {
        gql_ctx
            .data_unchecked::<PlexUsersService>()
            .create(
                input.id,
                &input.username,
                input.is_subscriber,
                &input.discord_user_id,
            )
            .await
    }

    async fn update_plex_user(
        &self,
        gql_ctx: &Context<'_>,
        input: UpdatePlexUserInput,
    ) -> Result<UpdatePlexUserResult> {
        gql_ctx
            .data_unchecked::<PlexUsersService>()
            .update(input.id, &input.username, input.is_subscriber)
            .await
    }

    async fn delete_plex_user(
        &self,
        gql_ctx: &Context<'_>,
        input: DeletePlexUserInput,
    ) -> Result<DeletePlexUserResult> {
        gql_ctx
            .data_unchecked::<PlexUsersService>()
            .delete(input.id)
            .await
    }
}

#[derive(Debug, InputObject)]
pub struct CreatePlexUserInput {
    pub id: i64,
    pub username: String,
    pub discord_user_id: String,
    pub is_subscriber: bool,
}

#[derive(Debug, InputObject)]
pub struct UpdatePlexUserInput {
    pub id: i64,
    pub username: String,
    pub is_subscriber: bool,
}

#[derive(Debug, InputObject)]
pub struct GetPlexUserInput {
    pub id: i64,
}

#[derive(Debug, InputObject)]
struct DeletePlexUserInput {
    id: i64,
}

#[derive(Debug, InputObject)]
struct ListPlexUserInput {
    discord_user_id: Option<String>,
}

#[derive(Enum, Clone, Debug, Copy, PartialEq, Eq)]
pub enum CreatePlexUserErrorVariant {
    TokenAlreadyExists,
    InternalError,
}

#[derive(Debug, SimpleObject)]
pub struct CreatePlexUserError {
    pub error: CreatePlexUserErrorVariant,
}

#[derive(Union)]
pub enum CreatePlexUserResult {
    Ok(PlexUserId),
    Error(CreatePlexUserError),
}

#[derive(Enum, Clone, Debug, Copy, PartialEq, Eq)]
pub enum UpdatePlexUserErrorVariant {
    UserDoesNotExist,
    InternalError,
}

#[derive(Debug, SimpleObject)]
pub struct UpdatePlexUserError {
    pub error: UpdatePlexUserErrorVariant,
}

#[derive(Union)]
pub enum UpdatePlexUserResult {
    Ok(plex_user::Model),
    Err(UpdatePlexUserError),
}

#[derive(Enum, Clone, Debug, Copy, PartialEq, Eq)]
pub enum GetPlexUserVariant {
    UserDoesNotExist,
    InternalError,
}

#[derive(Debug, SimpleObject)]
pub struct GetPlexUserError {
    pub error: GetPlexUserVariant,
}

#[derive(Union)]
pub enum GetPlexUserResult {
    Ok(plex_user::Model),
    Err(GetPlexUserError),
}

#[derive(Enum, Clone, Debug, Copy, PartialEq, Eq)]
pub enum DeletePlexUserErrorVariant {
    UserDoesNotExist,
    InternalError,
}

#[derive(Debug, SimpleObject)]
pub struct DeletePlexUserError {
    pub error: DeletePlexUserErrorVariant,
}

#[derive(Debug, SimpleObject)]
pub struct DeletePlexUserSuccess {
    pub message: String,
}

#[derive(Union)]
pub enum DeletePlexUserResult {
    Ok(DeletePlexUserSuccess),
    Err(DeletePlexUserError),
}

#[derive(Debug, SimpleObject)]
pub struct PlexUserId {
    pub id: i64,
}

#[derive(Debug, Clone)]
pub struct PlexUsersService {
    db: DatabaseConnection,
}

impl PlexUsersService {
    pub fn new(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }

    pub async fn create(
        &self,
        id: i64,
        username: &str,
        is_subscriber: bool,
        discord_user_id: &str,
    ) -> Result<CreatePlexUserResult> {
        let data = plex_user::ActiveModel {
            id: ActiveValue::Set(id),
            username: ActiveValue::Set(username.to_owned()),
            is_subscriber: ActiveValue::Set(is_subscriber),
            discord_user_id: ActiveValue::Set(discord_user_id.to_owned()),
            ..Default::default()
        };

        let result = match plex_user::Entity::insert(data).exec(&self.db).await {
            Ok(result) => result,
            Err(DbErr::Query(err)) => {
                tracing::warn!("create DbErr::Query: {:?}", err);
                return Ok(CreatePlexUserResult::Error(CreatePlexUserError {
                    error: CreatePlexUserErrorVariant::TokenAlreadyExists,
                }));
            }
            Err(DbErr::Exec(err)) => {
                tracing::warn!("create DbErr::Exec: {:?}", err);
                return Ok(CreatePlexUserResult::Error(CreatePlexUserError {
                    error: CreatePlexUserErrorVariant::TokenAlreadyExists,
                }));
            }
            Err(err) => {
                tracing::warn!("create Unknown: {:?}", err);
                return Ok(CreatePlexUserResult::Error(CreatePlexUserError {
                    error: CreatePlexUserErrorVariant::InternalError,
                }));
            }
        };

        Ok(CreatePlexUserResult::Ok(PlexUserId {
            id: result.last_insert_id,
        }))
    }

    pub async fn get(&self, id: i64) -> Result<GetPlexUserResult> {
        Ok(
            match plex_user::Entity::find_by_id(id).one(&self.db).await {
                Ok(Some(result)) => GetPlexUserResult::Ok(result),
                Ok(None) => GetPlexUserResult::Err(GetPlexUserError {
                    error: GetPlexUserVariant::UserDoesNotExist,
                }),
                Err(err) => {
                    tracing::warn!("get db error: {:?}", err);
                    GetPlexUserResult::Err(GetPlexUserError {
                        error: GetPlexUserVariant::InternalError,
                    })
                }
            },
        )
    }

    pub async fn list(&self, discord_user_id: Option<String>) -> Result<Vec<plex_user::Model>> {
        Ok(plex_user::Entity::find()
            .apply_if(discord_user_id, |query, value| {
                query.filter(plex_user::Column::DiscordUserId.eq(value))
            })
            .all(&self.db)
            .await?)
    }

    pub async fn update(
        &self,
        id: i64,
        username: &str,
        is_subscriber: bool,
    ) -> Result<UpdatePlexUserResult> {
        let user = plex_user::ActiveModel {
            id: ActiveValue::Set(id.to_owned()),
            username: ActiveValue::Set(username.to_owned()),
            is_subscriber: ActiveValue::Set(is_subscriber),
            updated_at: ActiveValue::Set(Utc::now()),
            ..Default::default()
        };
        Ok(match plex_user::Entity::update(user).exec(&self.db).await {
            Ok(user) => UpdatePlexUserResult::Ok(user),
            Err(DbErr::RecordNotUpdated) => UpdatePlexUserResult::Err(UpdatePlexUserError {
                error: UpdatePlexUserErrorVariant::UserDoesNotExist,
            }),
            Err(err) => {
                tracing::warn!("update db error: {:?}", err);
                UpdatePlexUserResult::Err(UpdatePlexUserError {
                    error: UpdatePlexUserErrorVariant::InternalError,
                })
            }
        })
    }

    pub async fn delete(&self, id: i64) -> Result<DeletePlexUserResult> {
        Ok(
            match plex_user::Entity::delete_by_id(id).exec(&self.db).await {
                Ok(res) => match res.rows_affected {
                    0 => DeletePlexUserResult::Err(DeletePlexUserError {
                        error: DeletePlexUserErrorVariant::UserDoesNotExist,
                    }),
                    _ => DeletePlexUserResult::Ok(DeletePlexUserSuccess {
                        message: "ok".into(),
                    }),
                },
                Err(err) => {
                    tracing::warn!("delete db error: {:?}", err);
                    DeletePlexUserResult::Err(DeletePlexUserError {
                        error: DeletePlexUserErrorVariant::InternalError,
                    })
                }
            },
        )
    }
}

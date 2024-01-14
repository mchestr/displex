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
use sea_query::OnConflict;
use tracing::instrument;

use crate::{
    entities::{
        plex_user,
        prelude::*,
    },
    server::cookies::{
        verify_role,
        Role,
    },
};

#[derive(Default)]
pub struct PlexUsersQuery;

#[Object]
impl PlexUsersQuery {
    async fn get_plex_user(
        &self,
        gql_ctx: &Context<'_>,
        input: GetPlexUserInput,
    ) -> Result<GetPlexUserResult> {
        verify_role(gql_ctx, Role::Admin)?;
        gql_ctx
            .data_unchecked::<PlexUsersService>()
            .get(&input.id)
            .await
    }
    async fn list_plex_users(
        &self,
        gql_ctx: &Context<'_>,
        input: ListPlexUserInput,
    ) -> Result<Vec<plex_user::Model>> {
        verify_role(gql_ctx, Role::Admin)?;
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
        verify_role(gql_ctx, Role::Admin)?;
        gql_ctx
            .data_unchecked::<PlexUsersService>()
            .create(
                &input.id,
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
        verify_role(gql_ctx, Role::Admin)?;
        gql_ctx
            .data_unchecked::<PlexUsersService>()
            .update(&input.id, &input.username, input.is_subscriber)
            .await
    }

    async fn delete_plex_user(
        &self,
        gql_ctx: &Context<'_>,
        input: DeletePlexUserInput,
    ) -> Result<DeletePlexUserResult> {
        verify_role(gql_ctx, Role::Admin)?;
        gql_ctx
            .data_unchecked::<PlexUsersService>()
            .delete(&input.id)
            .await
    }
}

#[derive(Debug, InputObject)]
pub struct CreatePlexUserInput {
    pub id: String,
    pub username: String,
    pub discord_user_id: String,
    pub is_subscriber: bool,
}

#[derive(Debug, InputObject)]
pub struct UpdatePlexUserInput {
    pub id: String,
    pub username: String,
    pub is_subscriber: bool,
}

#[derive(Debug, InputObject)]
pub struct GetPlexUserInput {
    pub id: String,
}

#[derive(Debug, InputObject)]
struct DeletePlexUserInput {
    id: String,
}

#[derive(Debug, InputObject)]
struct ListPlexUserInput {
    discord_user_id: Option<String>,
}

#[derive(Enum, Clone, Debug, Copy, PartialEq, Eq)]
pub enum CreatePlexUserErrorVariant {
    InternalError,
}

#[derive(Debug, SimpleObject)]
pub struct CreatePlexUserError {
    pub error: CreatePlexUserErrorVariant,
}

#[derive(Debug, Union)]
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

#[derive(Debug, Union)]
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

#[derive(Debug, Union)]
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

#[derive(Debug, Union)]
pub enum DeletePlexUserResult {
    Ok(DeletePlexUserSuccess),
    Err(DeletePlexUserError),
}

#[derive(Debug, SimpleObject)]
pub struct PlexUserId {
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct PlexUsersService {
    db: DatabaseConnection,
}

impl PlexUsersService {
    pub fn new(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }

    #[instrument(skip(self), ret)]
    pub async fn create(
        &self,
        id: &str,
        username: &str,
        is_subscriber: bool,
        discord_user_id: &str,
    ) -> Result<CreatePlexUserResult> {
        self.create_with_conn(id, username, is_subscriber, discord_user_id, &self.db)
            .await
    }

    #[instrument(skip(self, conn), ret)]
    pub async fn create_with_conn<'a, C>(
        &self,
        id: &str,
        username: &str,
        is_subscriber: bool,
        discord_user_id: &str,
        conn: &'a C,
    ) -> Result<CreatePlexUserResult>
    where
        C: ConnectionTrait,
    {
        let data = plex_user::ActiveModel {
            id: ActiveValue::Set(id.to_owned()),
            username: ActiveValue::Set(username.to_owned()),
            is_subscriber: ActiveValue::Set(is_subscriber),
            discord_user_id: ActiveValue::Set(discord_user_id.to_owned()),
            ..Default::default()
        };

        let result = match PlexUser::insert(data)
            .on_conflict(
                OnConflict::column(plex_user::Column::Id)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(conn)
            .await
        {
            Ok(result) => result,
            Err(DbErr::UnpackInsertId) | Err(DbErr::RecordNotInserted) => {
                return Ok(CreatePlexUserResult::Ok(PlexUserId { id: id.to_owned() }))
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

    #[instrument(skip(self), ret)]
    pub async fn get(&self, id: &str) -> Result<GetPlexUserResult> {
        Ok(match PlexUser::find_by_id(id).one(&self.db).await {
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
        })
    }

    #[instrument(skip(self), ret)]
    pub async fn list(&self, discord_user_id: Option<String>) -> Result<Vec<plex_user::Model>> {
        Ok(PlexUser::find()
            .apply_if(discord_user_id, |query, value| {
                query.filter(plex_user::Column::DiscordUserId.eq(value))
            })
            .all(&self.db)
            .await?)
    }

    #[instrument(skip(self), ret)]
    pub async fn update(
        &self,
        id: &str,
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
        Ok(match PlexUser::update(user).exec(&self.db).await {
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

    #[instrument(skip(self), ret)]
    pub async fn delete(&self, id: &str) -> Result<DeletePlexUserResult> {
        Ok(match PlexUser::delete_by_id(id).exec(&self.db).await {
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
        })
    }
}

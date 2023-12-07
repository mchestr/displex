use async_graphql::{
    Context,
    Enum,
    InputObject,
    Object,
    OneofObject,
    Result,
    SimpleObject,
    Union,
};
use chrono::Utc;
use sea_orm::{
    prelude::*,
    ActiveValue,
    FromJsonQueryResult,
};
use sea_query::OnConflict;
use serde::{
    Deserialize,
    Serialize,
};
use tracing::instrument;

use crate::{
    entities::{
        discord_token,
        discord_user,
        plex_token,
        plex_user,
        prelude::*,
    },
    services::{
        discord_token::resolver::DiscordTokensService,
        plex_token::resolver::PlexTokensService,
        plex_user::resolver::PlexUsersService,
    },
};

pub static COOKIE_NAME: &str = "auth";

#[derive(
    SimpleObject, Debug, PartialEq, Eq, Clone, Serialize, Deserialize, FromJsonQueryResult,
)]
pub struct DiscordUserSummary {
    discord_user: discord_user::Model,
    discord_tokens: Vec<discord_token::Model>,
    plex_users: Vec<plex_user::Model>,
    plex_tokens: Vec<plex_token::Model>,
}

#[derive(Default)]
pub struct DiscordUsersQuery;

#[Object]
impl DiscordUsersQuery {
    async fn get_discord_user(
        &self,
        gql_ctx: &Context<'_>,
        input: GetDiscordUserInput,
    ) -> Result<GetDiscordUserResult> {
        gql_ctx
            .data_unchecked::<DiscordUsersService>()
            .get(&input.id)
            .await
    }

    async fn list_discord_users(&self, gql_ctx: &Context<'_>) -> Result<Vec<discord_user::Model>> {
        gql_ctx.data_unchecked::<DiscordUsersService>().list().await
    }

    async fn user_summary(
        &self,
        gql_ctx: &Context<'_>,
        input: UserSummaryBy,
    ) -> Result<SummaryDiscordUserResult> {
        gql_ctx
            .data_unchecked::<DiscordUsersService>()
            .summary(&input)
            .await
    }
}

#[derive(Default)]
pub struct DiscordUsersMutation;

#[Object]
impl DiscordUsersMutation {
    async fn create_discord_user(
        &self,
        gql_ctx: &Context<'_>,
        input: CreateDiscordUserInput,
    ) -> Result<CreateDiscordUserResult> {
        gql_ctx
            .data_unchecked::<DiscordUsersService>()
            .create(&input.id, &input.username)
            .await
    }

    async fn update_discord_user(
        &self,
        gql_ctx: &Context<'_>,
        input: UpdateDiscordUserInput,
    ) -> Result<UpdateDiscordUserResult> {
        gql_ctx
            .data_unchecked::<DiscordUsersService>()
            .update(&input.id, &input.username)
            .await
    }

    async fn delete_discord_user(
        &self,
        gql_ctx: &Context<'_>,
        input: DeleteDiscordUserInput,
    ) -> Result<DeleteDiscordUserResult> {
        gql_ctx
            .data_unchecked::<DiscordUsersService>()
            .delete(&input.id)
            .await
    }
}

#[derive(Debug, InputObject)]
pub struct CreateDiscordUserInput {
    pub id: String,
    pub username: String,
}

#[derive(Debug, InputObject)]
pub struct UpdateDiscordUserInput {
    pub id: String,
    pub username: String,
}

#[derive(Debug, InputObject)]
pub struct GetDiscordUserInput {
    pub id: String,
}

#[derive(Debug, InputObject)]
pub struct DeleteDiscordUserInput {
    pub id: String,
}

#[derive(Debug, OneofObject)]
pub enum UserSummaryBy {
    Username(String),
    Id(String),
}

#[derive(Enum, Clone, Debug, Copy, PartialEq, Eq)]
pub enum CreateDiscordUserErrorVariant {
    InternalError,
}

#[derive(Debug, SimpleObject)]
pub struct CreateDiscordUserError {
    pub error: CreateDiscordUserErrorVariant,
}

#[derive(Debug, Union)]
pub enum CreateDiscordUserResult {
    Ok(DiscordUserId),
    Error(CreateDiscordUserError),
}

#[derive(Enum, Clone, Debug, Copy, PartialEq, Eq)]
pub enum UpdateDiscordUserErrorVariant {
    UserDoesNotExist,
    InternalError,
}

#[derive(Debug, SimpleObject)]
pub struct UpdateDiscordUserError {
    pub error: UpdateDiscordUserErrorVariant,
}

#[derive(Debug, Union)]
pub enum UpdateDiscordUserResult {
    Ok(discord_user::Model),
    Err(UpdateDiscordUserError),
}

#[derive(Enum, Clone, Debug, Copy, PartialEq, Eq)]
pub enum GetDiscordUserVariant {
    UserDoesNotExist,
    InternalError,
}

#[derive(Debug, SimpleObject)]
pub struct GetDiscordUserError {
    pub error: GetDiscordUserVariant,
}

#[derive(Debug, Union)]
pub enum GetDiscordUserResult {
    Ok(discord_user::Model),
    Err(GetDiscordUserError),
}

#[derive(Enum, Clone, Debug, Copy, PartialEq, Eq)]
pub enum DeleteDiscordUserErrorVariant {
    UserDoesNotExist,
    InternalError,
}

#[derive(Debug, SimpleObject)]
pub struct DeleteDiscordUserError {
    pub error: DeleteDiscordUserErrorVariant,
}

#[derive(Debug, SimpleObject)]
pub struct DeleteDiscordUserSuccess {
    pub message: String,
}

#[derive(Debug, Union)]
pub enum DeleteDiscordUserResult {
    Ok(DeleteDiscordUserSuccess),
    Err(DeleteDiscordUserError),
}

#[derive(Debug, SimpleObject)]
pub struct DiscordUserId {
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct DiscordUsersService {
    db: DatabaseConnection,
    discord_tokens_service: DiscordTokensService,
    plex_tokens_service: PlexTokensService,
    plex_users_service: PlexUsersService,
}

#[derive(Enum, Clone, Debug, Copy, PartialEq, Eq)]
pub enum SummaryDiscordUserErrorVariant {
    UserDoesNotExist,
    InternalError,
}

#[derive(Debug, SimpleObject)]
pub struct SummaryDiscordUserError {
    pub error: DeleteDiscordUserErrorVariant,
}

#[derive(Debug, SimpleObject)]
pub struct SummaryDiscordUserSuccess {
    pub summary: DiscordUserSummary,
}

#[derive(Debug, Union)]
pub enum SummaryDiscordUserResult {
    Ok(SummaryDiscordUserSuccess),
    Err(SummaryDiscordUserError),
}

impl DiscordUsersService {
    pub fn new(
        db: &DatabaseConnection,
        discord_tokens_service: &DiscordTokensService,
        plex_tokens_service: &PlexTokensService,
        plex_users_service: &PlexUsersService,
    ) -> Self {
        Self {
            db: db.clone(),
            discord_tokens_service: discord_tokens_service.clone(),
            plex_tokens_service: plex_tokens_service.clone(),
            plex_users_service: plex_users_service.clone(),
        }
    }

    #[instrument(skip(self), ret)]
    pub async fn create(&self, id: &str, username: &str) -> Result<CreateDiscordUserResult> {
        self.create_with_conn(id, username, &self.db).await
    }

    #[instrument(skip(self, conn), ret)]
    pub async fn create_with_conn<'a, C>(
        &self,
        id: &str,
        username: &str,
        conn: &'a C,
    ) -> Result<CreateDiscordUserResult>
    where
        C: ConnectionTrait,
    {
        let user = discord_user::ActiveModel {
            id: ActiveValue::Set(id.to_owned()),
            username: ActiveValue::Set(username.to_owned()),
            ..Default::default()
        };

        let user = match DiscordUser::insert(user)
            .on_conflict(
                OnConflict::column(discord_user::Column::Id)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(conn)
            .await
        {
            Ok(user) => user,
            Err(DbErr::UnpackInsertId) | Err(DbErr::RecordNotInserted) => {
                return Ok(CreateDiscordUserResult::Ok(DiscordUserId { id: id.into() }))
            }
            Err(err) => {
                tracing::warn!("create Unknown: {:?}", err);
                return Ok(CreateDiscordUserResult::Error(CreateDiscordUserError {
                    error: CreateDiscordUserErrorVariant::InternalError,
                }));
            }
        };

        Ok(CreateDiscordUserResult::Ok(DiscordUserId {
            id: user.last_insert_id,
        }))
    }

    #[instrument(skip(self), ret)]
    pub async fn list(&self) -> Result<Vec<discord_user::Model>> {
        Ok(DiscordUser::find().all(&self.db).await?)
    }

    #[instrument(skip(self), ret)]
    pub async fn deactivate(&self, id: &str) -> Result<UpdateDiscordUserResult> {
        let user = discord_user::ActiveModel {
            id: ActiveValue::Set(id.to_owned()),
            is_active: ActiveValue::Set(false),
            updated_at: ActiveValue::Set(Utc::now()),
            ..Default::default()
        };
        Ok(match DiscordUser::update(user).exec(&self.db).await {
            Ok(user) => UpdateDiscordUserResult::Ok(user),
            Err(DbErr::RecordNotUpdated) => UpdateDiscordUserResult::Err(UpdateDiscordUserError {
                error: UpdateDiscordUserErrorVariant::UserDoesNotExist,
            }),
            Err(err) => {
                tracing::warn!("update db error: {:?}", err);
                UpdateDiscordUserResult::Err(UpdateDiscordUserError {
                    error: UpdateDiscordUserErrorVariant::InternalError,
                })
            }
        })
    }

    #[instrument(skip(self), ret)]
    pub async fn update(&self, id: &str, username: &str) -> Result<UpdateDiscordUserResult> {
        let user = discord_user::ActiveModel {
            id: ActiveValue::Set(id.to_owned()),
            username: ActiveValue::Set(username.to_owned()),
            updated_at: ActiveValue::Set(Utc::now()),
            ..Default::default()
        };
        Ok(match DiscordUser::update(user).exec(&self.db).await {
            Ok(user) => UpdateDiscordUserResult::Ok(user),
            Err(DbErr::RecordNotUpdated) => UpdateDiscordUserResult::Err(UpdateDiscordUserError {
                error: UpdateDiscordUserErrorVariant::UserDoesNotExist,
            }),
            Err(err) => {
                tracing::warn!("update db error: {:?}", err);
                UpdateDiscordUserResult::Err(UpdateDiscordUserError {
                    error: UpdateDiscordUserErrorVariant::InternalError,
                })
            }
        })
    }

    #[instrument(skip(self), ret)]
    pub async fn get(&self, id: &str) -> Result<GetDiscordUserResult> {
        Ok(match DiscordUser::find_by_id(id).one(&self.db).await {
            Ok(Some(result)) => GetDiscordUserResult::Ok(result),
            Ok(None) => GetDiscordUserResult::Err(GetDiscordUserError {
                error: GetDiscordUserVariant::UserDoesNotExist,
            }),
            Err(err) => {
                tracing::warn!("get db error: {:?}", err);
                GetDiscordUserResult::Err(GetDiscordUserError {
                    error: GetDiscordUserVariant::InternalError,
                })
            }
        })
    }

    #[instrument(skip(self), ret)]
    pub async fn get_by_username(&self, username: &str) -> Result<GetDiscordUserResult> {
        Ok(
            match DiscordUser::find()
                .filter(discord_user::Column::Username.eq(username))
                .one(&self.db)
                .await
            {
                Ok(Some(result)) => GetDiscordUserResult::Ok(result),
                Ok(None) => GetDiscordUserResult::Err(GetDiscordUserError {
                    error: GetDiscordUserVariant::UserDoesNotExist,
                }),
                Err(err) => {
                    tracing::warn!("get_by_username db error: {:?}", err);
                    GetDiscordUserResult::Err(GetDiscordUserError {
                        error: GetDiscordUserVariant::InternalError,
                    })
                }
            },
        )
    }

    #[instrument(skip(self), ret)]
    pub async fn delete(&self, id: &str) -> Result<DeleteDiscordUserResult> {
        Ok(match DiscordUser::delete_by_id(id).exec(&self.db).await {
            Ok(res) => match res.rows_affected {
                0 => DeleteDiscordUserResult::Err(DeleteDiscordUserError {
                    error: DeleteDiscordUserErrorVariant::UserDoesNotExist,
                }),
                _ => DeleteDiscordUserResult::Ok(DeleteDiscordUserSuccess {
                    message: "ok".into(),
                }),
            },
            Err(err) => {
                tracing::warn!("delete db error: {:?}", err);
                DeleteDiscordUserResult::Err(DeleteDiscordUserError {
                    error: DeleteDiscordUserErrorVariant::InternalError,
                })
            }
        })
    }

    #[instrument(skip(self), ret)]
    pub async fn summary(&self, user_by: &UserSummaryBy) -> Result<SummaryDiscordUserResult> {
        let discord_user = match user_by {
            UserSummaryBy::Username(username) => match self.get_by_username(username).await? {
                GetDiscordUserResult::Ok(result) => result,
                GetDiscordUserResult::Err(_) => {
                    return Ok(SummaryDiscordUserResult::Err(SummaryDiscordUserError {
                        error: DeleteDiscordUserErrorVariant::UserDoesNotExist,
                    }))
                }
            },
            UserSummaryBy::Id(id) => match self.get(id).await? {
                GetDiscordUserResult::Ok(result) => result,
                GetDiscordUserResult::Err(_) => {
                    return Ok(SummaryDiscordUserResult::Err(SummaryDiscordUserError {
                        error: DeleteDiscordUserErrorVariant::UserDoesNotExist,
                    }))
                }
            },
        };

        let discord_tokens = self
            .discord_tokens_service
            .list(Some(discord_user.id.clone()), None, None)
            .await?;
        let plex_users = self
            .plex_users_service
            .list(Some(discord_user.id.clone()))
            .await?;
        let plex_user_ids: Vec<String> = plex_users.iter().map(|u| String::from(&u.id)).collect();
        let plex_tokens = self
            .plex_tokens_service
            .list(None, Some(plex_user_ids))
            .await?;
        Ok(SummaryDiscordUserResult::Ok(SummaryDiscordUserSuccess {
            summary: DiscordUserSummary {
                discord_user,
                discord_tokens,
                plex_tokens,
                plex_users,
            },
        }))
    }

    #[instrument(skip(self), ret)]
    pub async fn list_users_for_refresh(
        &self,
    ) -> Result<Vec<(discord_user::Model, Option<plex_user::Model>)>> {
        Ok(DiscordUser::find()
            .filter(discord_user::Column::IsActive.eq(true))
            .find_also_related(plex_user::Entity)
            .all(&self.db)
            .await?)
    }

    #[instrument(skip(self))]
    pub async fn list_subscribers(
        &self,
    ) -> Result<Vec<(discord_user::Model, Option<plex_user::Model>)>> {
        Ok(DiscordUser::find()
            .find_also_related(plex_user::Entity)
            .all(&self.db)
            .await?)
    }

    pub async fn list_subscriber_tokens(
        &self,
    ) -> Result<Vec<(discord_user::Model, Option<discord_token::Model>)>> {
        Ok(DiscordUser::find()
            .find_also_related(discord_token::Entity)
            .filter(discord_token::Column::Status.eq(discord_token::TokenStatus::Active))
            .all(&self.db)
            .await?)
    }
}

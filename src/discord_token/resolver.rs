use async_graphql::{
    Context,
    Enum,
    InputObject,
    Object,
    Result,
    SimpleObject,
    Union,
};

use sea_orm::{
    prelude::*,
    ActiveValue,
    QueryOrder,
    QueryTrait,
};

use crate::entities::discord_token;

pub static COOKIE_NAME: &str = "auth";

#[derive(Default)]
pub struct DiscordTokensQuery;

#[Object]
impl DiscordTokensQuery {
    async fn get_discord_token(
        &self,
        gql_ctx: &Context<'_>,
        input: GetDiscordTokenInput,
    ) -> Result<GetDiscordTokenResult> {
        gql_ctx
            .data_unchecked::<DiscordTokensService>()
            .get(&input.access_token)
            .await
    }
    async fn list_discord_tokens(
        &self,
        gql_ctx: &Context<'_>,
        input: ListDiscordTokenInput,
    ) -> Result<Vec<discord_token::Model>> {
        gql_ctx
            .data_unchecked::<DiscordTokensService>()
            .list(input.discord_user_id)
            .await
    }
}

#[derive(Default)]
pub struct DiscordTokensMutation;

#[Object]
impl DiscordTokensMutation {
    async fn create_discord_token(
        &self,
        gql_ctx: &Context<'_>,
        input: CreateDiscordTokenInput,
    ) -> Result<CreateDiscordTokenResult> {
        gql_ctx
            .data_unchecked::<DiscordTokensService>()
            .create(
                &input.access_token,
                &input.refresh_token,
                &input.expires_at,
                &input.scopes,
                &input.discord_user_id,
            )
            .await
    }

    async fn delete_discord_token(
        &self,
        gql_ctx: &Context<'_>,
        input: DeleteDiscordTokenInput,
    ) -> Result<DeleteDiscordTokenResult> {
        gql_ctx
            .data_unchecked::<DiscordTokensService>()
            .delete(&input.access_token)
            .await
    }
}

#[derive(Debug, InputObject)]
pub struct CreateDiscordTokenInput {
    pub access_token: String,
    pub refresh_token: String,
    pub scopes: String,
    pub expires_at: DateTimeUtc,
    pub discord_user_id: String,
}

#[derive(Debug, InputObject)]
pub struct GetDiscordTokenInput {
    pub access_token: String,
}

#[derive(Debug, InputObject)]
pub struct DeleteDiscordTokenInput {
    pub access_token: String,
}

#[derive(Debug, InputObject)]
pub struct ListDiscordTokenInput {
    pub discord_user_id: Option<String>,
}

#[derive(Enum, Clone, Debug, Copy, PartialEq, Eq)]
pub enum CreateDiscordTokenErrorVariant {
    TokenAlreadyExists,
    InternalError,
}

#[derive(Debug, SimpleObject)]
pub struct CreateDiscordTokenError {
    pub error: CreateDiscordTokenErrorVariant,
}

#[derive(Union)]
pub enum CreateDiscordTokenResult {
    Ok(DiscordTokenId),
    Error(CreateDiscordTokenError),
}

#[derive(Enum, Clone, Debug, Copy, PartialEq, Eq)]
pub enum GetDiscordTokenVariant {
    TokenDoesNotExist,
    InternalError,
}

#[derive(Debug, SimpleObject)]
pub struct GetDiscordTokenError {
    pub error: GetDiscordTokenVariant,
}

#[derive(Union)]
pub enum GetDiscordTokenResult {
    Ok(discord_token::Model),
    Err(GetDiscordTokenError),
}

#[derive(Enum, Clone, Debug, Copy, PartialEq, Eq)]
pub enum DeleteDiscordTokenErrorVariant {
    UserDoesNotExist,
    InternalError,
}

#[derive(Debug, SimpleObject)]
pub struct DeleteDiscordTokenError {
    pub error: DeleteDiscordTokenErrorVariant,
}

#[derive(Debug, SimpleObject)]
pub struct DeleteDiscordTokenSuccess {
    pub message: String,
}

#[derive(Union)]
pub enum DeleteDiscordTokenResult {
    Ok(DeleteDiscordTokenSuccess),
    Err(DeleteDiscordTokenError),
}

#[derive(Debug, SimpleObject)]
pub struct DiscordTokenId {
    pub access_token: String,
}

#[derive(Debug, Clone)]
pub struct DiscordTokensService {
    db: DatabaseConnection,
}

impl DiscordTokensService {
    pub fn new(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }

    pub async fn create(
        &self,
        access_token: &str,
        refresh_token: &str,
        expires_at: &DateTimeUtc,
        scopes: &str,
        discord_user_id: &str,
    ) -> Result<CreateDiscordTokenResult> {
        let data = discord_token::ActiveModel {
            access_token: ActiveValue::Set(access_token.to_owned()),
            refresh_token: ActiveValue::Set(refresh_token.to_owned()),
            expires_at: ActiveValue::Set(expires_at.to_owned()),
            scopes: ActiveValue::Set(scopes.to_owned()),
            discord_user_id: ActiveValue::Set(discord_user_id.to_owned()),
            ..Default::default()
        };

        match discord_token::Entity::insert(data).exec(&self.db).await {
            Ok(result) => result,
            Err(DbErr::UnpackInsertId) => {
                return Ok(CreateDiscordTokenResult::Ok(DiscordTokenId {
                    access_token: access_token.into(),
                }))
            }
            Err(DbErr::Query(err)) => {
                tracing::warn!("create DbErr::Query: {:?}", err);
                return Ok(CreateDiscordTokenResult::Ok(DiscordTokenId {
                    access_token: access_token.to_owned(),
                }));
            }
            Err(DbErr::Exec(err)) => {
                tracing::warn!("create DbErr::Exec: {:?}", err);
                return Ok(CreateDiscordTokenResult::Error(CreateDiscordTokenError {
                    error: CreateDiscordTokenErrorVariant::TokenAlreadyExists,
                }));
            }
            Err(err) => {
                tracing::warn!("create Unknown: {:?}", err);
                return Ok(CreateDiscordTokenResult::Error(CreateDiscordTokenError {
                    error: CreateDiscordTokenErrorVariant::InternalError,
                }));
            }
        };

        Ok(CreateDiscordTokenResult::Ok(DiscordTokenId {
            access_token: access_token.to_owned(),
        }))
    }

    pub async fn get(&self, access_token: &str) -> Result<GetDiscordTokenResult> {
        Ok(
            match discord_token::Entity::find_by_id(access_token)
                .one(&self.db)
                .await
            {
                Ok(Some(result)) => GetDiscordTokenResult::Ok(result),
                Ok(None) => GetDiscordTokenResult::Err(GetDiscordTokenError {
                    error: GetDiscordTokenVariant::TokenDoesNotExist,
                }),
                Err(err) => {
                    tracing::warn!("get db error: {:?}", err);
                    GetDiscordTokenResult::Err(GetDiscordTokenError {
                        error: GetDiscordTokenVariant::InternalError,
                    })
                }
            },
        )
    }

    pub async fn list(&self, discord_user_id: Option<String>) -> Result<Vec<discord_token::Model>> {
        Ok(discord_token::Entity::find()
            .apply_if(discord_user_id, |query, value| {
                query.filter(discord_token::Column::DiscordUserId.eq(value))
            })
            .all(&self.db)
            .await?)
    }

    pub async fn delete(&self, access_token: &str) -> Result<DeleteDiscordTokenResult> {
        Ok(
            match discord_token::Entity::delete_by_id(access_token)
                .exec(&self.db)
                .await
            {
                Ok(res) => match res.rows_affected {
                    0 => DeleteDiscordTokenResult::Err(DeleteDiscordTokenError {
                        error: DeleteDiscordTokenErrorVariant::UserDoesNotExist,
                    }),
                    _ => DeleteDiscordTokenResult::Ok(DeleteDiscordTokenSuccess {
                        message: "ok".into(),
                    }),
                },
                Err(err) => {
                    tracing::warn!("got db error: {:?}", err);
                    DeleteDiscordTokenResult::Err(DeleteDiscordTokenError {
                        error: DeleteDiscordTokenErrorVariant::InternalError,
                    })
                }
            },
        )
    }

    pub async fn latest_token(
        &self,
        discord_user_id: &str,
    ) -> Result<Option<discord_token::Model>> {
        Ok(discord_token::Entity::find()
            .filter(discord_token::Column::DiscordUserId.eq(discord_user_id))
            .order_by_desc(discord_token::Column::ExpiresAt)
            .one(&self.db)
            .await?)
    }
}

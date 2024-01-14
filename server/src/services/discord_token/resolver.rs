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
    QueryOrder,
    QueryTrait,
};
use sea_query::OnConflict;
use tracing::instrument;

use crate::entities::discord_token::{
    self,
    TokenStatus,
};

use crate::{
    entities::prelude::*,
    server::cookies::{
        verify_role,
        Role,
    },
};

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
        verify_role(gql_ctx, Role::Admin)?;
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
        verify_role(gql_ctx, Role::Admin)?;
        gql_ctx
            .data_unchecked::<DiscordTokensService>()
            .list(input.discord_user_id, input.before_expires_at, input.status)
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
        verify_role(gql_ctx, Role::Admin)?;
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
        verify_role(gql_ctx, Role::Admin)?;
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
    pub before_expires_at: Option<chrono::DateTime<Utc>>,
    pub status: Option<TokenStatus>,
}

#[derive(Enum, Clone, Debug, Copy, PartialEq, Eq)]
pub enum CreateDiscordTokenErrorVariant {
    InternalError,
    Unauthorized,
}

#[derive(Debug, SimpleObject)]
pub struct CreateDiscordTokenError {
    pub error: CreateDiscordTokenErrorVariant,
}

#[derive(Debug, Union)]
pub enum CreateDiscordTokenResult {
    Ok(DiscordTokenId),
    Error(CreateDiscordTokenError),
}

#[derive(Debug, Enum, Clone, Copy, PartialEq, Eq)]
pub enum GetDiscordTokenVariant {
    TokenDoesNotExist,
    InternalError,
}

#[derive(Debug, SimpleObject)]
pub struct GetDiscordTokenError {
    pub error: GetDiscordTokenVariant,
}

#[derive(Debug, Union)]
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

#[derive(Debug, Union)]
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

    #[instrument(skip(self), ret)]
    pub async fn create(
        &self,
        access_token: &str,
        refresh_token: &str,
        expires_at: &DateTimeUtc,
        scopes: &str,
        discord_user_id: &str,
    ) -> Result<CreateDiscordTokenResult> {
        self.create_with_conn(
            access_token,
            refresh_token,
            expires_at,
            scopes,
            discord_user_id,
            &self.db,
        )
        .await
    }

    #[instrument(skip(self, conn), ret)]
    pub async fn create_with_conn<'a, C>(
        &self,
        access_token: &str,
        refresh_token: &str,
        expires_at: &DateTimeUtc,
        scopes: &str,
        discord_user_id: &str,
        conn: &'a C,
    ) -> Result<CreateDiscordTokenResult>
    where
        C: ConnectionTrait,
    {
        let data = discord_token::ActiveModel {
            access_token: ActiveValue::Set(access_token.to_owned()),
            refresh_token: ActiveValue::Set(refresh_token.to_owned()),
            expires_at: ActiveValue::Set(expires_at.to_owned()),
            scopes: ActiveValue::Set(scopes.to_owned()),
            discord_user_id: ActiveValue::Set(discord_user_id.to_owned()),
            ..Default::default()
        };

        match DiscordToken::insert(data)
            .on_conflict(
                OnConflict::column(discord_token::Column::AccessToken)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(conn)
            .await
        {
            Ok(result) => result,
            Err(DbErr::UnpackInsertId) | Err(DbErr::RecordNotInserted) => {
                return Ok(CreateDiscordTokenResult::Ok(DiscordTokenId {
                    access_token: access_token.into(),
                }))
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

    #[instrument(skip(self))]
    pub async fn get(&self, access_token: &str) -> Result<GetDiscordTokenResult> {
        Ok(
            match DiscordToken::find_by_id(access_token).one(&self.db).await {
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

    #[instrument(skip(self), ret)]
    pub async fn list(
        &self,
        discord_user_id: Option<String>,
        before_expires: Option<chrono::DateTime<Utc>>,
        status: Option<TokenStatus>,
    ) -> Result<Vec<discord_token::Model>> {
        Ok(DiscordToken::find()
            .apply_if(discord_user_id, |query, value| {
                query.filter(discord_token::Column::DiscordUserId.eq(value))
            })
            .apply_if(before_expires, |query, value| {
                query.filter(discord_token::Column::ExpiresAt.lt(value))
            })
            .apply_if(status, |query, value| {
                query.filter(discord_token::Column::Status.eq(value))
            })
            .order_by_desc(discord_token::Column::ExpiresAt)
            .all(&self.db)
            .await?)
    }

    #[instrument(skip(self), ret)]
    pub async fn delete(&self, access_token: &str) -> Result<DeleteDiscordTokenResult> {
        Ok(
            match DiscordToken::delete_by_id(access_token)
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

    #[instrument(skip(self), ret)]
    pub async fn set_status(
        &self,
        discord_token: &str,
        status: TokenStatus,
    ) -> Result<discord_token::Model> {
        Ok(DiscordToken::update(discord_token::ActiveModel {
            access_token: ActiveValue::Set(discord_token.into()),
            status: ActiveValue::Set(status),
            ..Default::default()
        })
        .exec(&self.db)
        .await?)
    }

    #[instrument(skip(self), ret)]
    pub async fn latest_token(
        &self,
        discord_user_id: &str,
    ) -> Result<Option<discord_token::Model>> {
        Ok(DiscordToken::find()
            .filter(discord_token::Column::DiscordUserId.eq(discord_user_id))
            .order_by_desc(discord_token::Column::ExpiresAt)
            .one(&self.db)
            .await?)
    }
}

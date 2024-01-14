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
    QueryTrait,
};
use sea_query::OnConflict;
use tracing::instrument;

use crate::entities::plex_token;

use crate::{
    entities::prelude::*,
    server::cookies::{
        verify_role,
        Role,
    },
};

#[derive(Default)]
pub struct PlexTokensQuery;

#[Object]
impl PlexTokensQuery {
    async fn get_plex_token(
        &self,
        gql_ctx: &Context<'_>,
        input: GetPlexTokenInput,
    ) -> Result<GetPlexTokenResult> {
        verify_role(gql_ctx, Role::Admin)?;
        gql_ctx
            .data_unchecked::<PlexTokensService>()
            .get(&input.access_token)
            .await
    }
    async fn list_plex_tokens(
        &self,
        gql_ctx: &Context<'_>,
        input: ListPlexTokenInput,
    ) -> Result<Vec<plex_token::Model>> {
        verify_role(gql_ctx, Role::Admin)?;
        gql_ctx
            .data_unchecked::<PlexTokensService>()
            .list(input.plex_user_id, input.plex_user_ids)
            .await
    }
}

#[derive(Default)]
pub struct PlexTokensMutation;

#[Object]
impl PlexTokensMutation {
    async fn create_plex_token(
        &self,
        gql_ctx: &Context<'_>,
        input: CreatePlexTokenInput,
    ) -> Result<CreatePlexTokenResult> {
        verify_role(gql_ctx, Role::Admin)?;
        gql_ctx
            .data_unchecked::<PlexTokensService>()
            .create(&input.access_token, &input.plex_user_id)
            .await
    }

    async fn delete_plex_token(
        &self,
        gql_ctx: &Context<'_>,
        input: DeletePlexTokenInput,
    ) -> Result<DeletePlexTokenResult> {
        verify_role(gql_ctx, Role::Admin)?;
        gql_ctx
            .data_unchecked::<PlexTokensService>()
            .delete(&input.access_token)
            .await
    }
}

#[derive(Debug, InputObject)]
pub struct CreatePlexTokenInput {
    pub access_token: String,
    pub plex_user_id: String,
}

#[derive(Debug, InputObject)]
pub struct GetPlexTokenInput {
    pub access_token: String,
}

#[derive(Debug, InputObject)]
pub struct DeletePlexTokenInput {
    pub access_token: String,
}

#[derive(Debug, InputObject)]
pub struct ListPlexTokenInput {
    pub plex_user_id: Option<String>,
    pub plex_user_ids: Option<Vec<String>>,
}

#[derive(Enum, Clone, Debug, Copy, PartialEq, Eq)]
pub enum CreatePlexTokenErrorVariant {
    InternalError,
}

#[derive(Debug, SimpleObject)]
pub struct CreatePlexTokenError {
    pub error: CreatePlexTokenErrorVariant,
}

#[derive(Debug, Union)]
pub enum CreatePlexTokenResult {
    Ok(PlexTokenId),
    Error(CreatePlexTokenError),
}

#[derive(Enum, Clone, Debug, Copy, PartialEq, Eq)]
pub enum GetPlexTokenVariant {
    TokenDoesNotExist,
    InternalError,
}

#[derive(Debug, SimpleObject)]
pub struct GetPlexTokenError {
    pub error: GetPlexTokenVariant,
}

#[derive(Debug, Union)]
pub enum GetPlexTokenResult {
    Ok(plex_token::Model),
    Err(GetPlexTokenError),
}

#[derive(Enum, Clone, Debug, Copy, PartialEq, Eq)]
pub enum DeletePlexTokenErrorVariant {
    TokenDoesNotExist,
    InternalError,
}

#[derive(Debug, SimpleObject)]
pub struct DeletePlexTokenError {
    pub error: DeletePlexTokenErrorVariant,
}

#[derive(Debug, SimpleObject)]
pub struct DeletePlexTokenSuccess {
    pub message: String,
}

#[derive(Debug, Union)]
pub enum DeletePlexTokenResult {
    Ok(DeletePlexTokenSuccess),
    Err(DeletePlexTokenError),
}

#[derive(Debug, SimpleObject)]
pub struct PlexTokenId {
    pub access_token: String,
}

#[derive(Debug, Clone)]
pub struct PlexTokensService {
    db: DatabaseConnection,
}

impl PlexTokensService {
    pub fn new(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }

    #[instrument(skip(self), ret)]
    pub async fn create(
        &self,
        access_token: &str,
        plex_user_id: &str,
    ) -> Result<CreatePlexTokenResult> {
        self.create_with_conn(access_token, plex_user_id, &self.db)
            .await
    }

    #[instrument(skip(self, conn), ret)]
    pub async fn create_with_conn<'a, C>(
        &self,
        access_token: &str,
        plex_user_id: &str,
        conn: &'a C,
    ) -> Result<CreatePlexTokenResult>
    where
        C: ConnectionTrait,
    {
        let data = plex_token::ActiveModel {
            access_token: ActiveValue::Set(access_token.to_owned()),
            plex_user_id: ActiveValue::Set(plex_user_id.to_owned()),
            ..Default::default()
        };

        let result = match PlexToken::insert(data)
            .on_conflict(
                OnConflict::column(plex_token::Column::AccessToken)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(conn)
            .await
        {
            Ok(result) => result,
            Err(DbErr::UnpackInsertId) | Err(DbErr::RecordNotInserted) => {
                return Ok(CreatePlexTokenResult::Ok(PlexTokenId {
                    access_token: access_token.into(),
                }))
            }
            Err(err) => {
                tracing::warn!("create Unknown: {:?}", err);
                return Ok(CreatePlexTokenResult::Error(CreatePlexTokenError {
                    error: CreatePlexTokenErrorVariant::InternalError,
                }));
            }
        };

        Ok(CreatePlexTokenResult::Ok(PlexTokenId {
            access_token: result.last_insert_id,
        }))
    }

    #[instrument(skip(self), ret)]
    pub async fn get(&self, access_token: &str) -> Result<GetPlexTokenResult> {
        Ok(
            match PlexToken::find_by_id(access_token).one(&self.db).await {
                Ok(Some(result)) => GetPlexTokenResult::Ok(result),
                Ok(None) => GetPlexTokenResult::Err(GetPlexTokenError {
                    error: GetPlexTokenVariant::TokenDoesNotExist,
                }),
                Err(err) => {
                    tracing::warn!("get db error: {:?}", err);
                    GetPlexTokenResult::Err(GetPlexTokenError {
                        error: GetPlexTokenVariant::InternalError,
                    })
                }
            },
        )
    }

    #[instrument(skip(self), ret)]
    pub async fn list(
        &self,
        plex_user_id: Option<String>,
        plex_user_ids: Option<Vec<String>>,
    ) -> Result<Vec<plex_token::Model>> {
        Ok(PlexToken::find()
            .apply_if(plex_user_id, |query, value| {
                query.filter(plex_token::Column::PlexUserId.eq(value))
            })
            .apply_if(plex_user_ids, |query, value| {
                query.filter(plex_token::Column::PlexUserId.is_in(value))
            })
            .all(&self.db)
            .await?)
    }

    #[instrument(skip(self), ret)]
    pub async fn delete(&self, access_token: &str) -> Result<DeletePlexTokenResult> {
        Ok(
            match PlexToken::delete_by_id(access_token).exec(&self.db).await {
                Ok(res) => match res.rows_affected {
                    0 => DeletePlexTokenResult::Err(DeletePlexTokenError {
                        error: DeletePlexTokenErrorVariant::TokenDoesNotExist,
                    }),
                    _ => DeletePlexTokenResult::Ok(DeletePlexTokenSuccess {
                        message: "ok".into(),
                    }),
                },
                Err(err) => {
                    tracing::warn!("delete db error: {:?}", err);
                    DeletePlexTokenResult::Err(DeletePlexTokenError {
                        error: DeletePlexTokenErrorVariant::InternalError,
                    })
                }
            },
        )
    }
}

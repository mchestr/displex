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

use crate::entities::plex_token;

#[derive(Default)]
pub struct PlexTokensQuery;

#[Object]
impl PlexTokensQuery {
    async fn get_plex_token(
        &self,
        gql_ctx: &Context<'_>,
        input: GetPlexTokenInput,
    ) -> Result<GetPlexTokenResult> {
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
    TokenAlreadyExists,
    InternalError,
}

#[derive(Debug, SimpleObject)]
pub struct CreatePlexTokenError {
    pub error: CreatePlexTokenErrorVariant,
}

#[derive(Union)]
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

#[derive(Union)]
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

#[derive(Union)]
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

    pub async fn create(
        &self,
        access_token: &str,
        plex_user_id: &str,
    ) -> Result<CreatePlexTokenResult> {
        let data = plex_token::ActiveModel {
            access_token: ActiveValue::Set(access_token.to_owned()),
            plex_user_id: ActiveValue::Set(plex_user_id.to_owned()),
            ..Default::default()
        };

        let result = match plex_token::Entity::insert(data).exec(&self.db).await {
            Ok(result) => result,
            Err(DbErr::UnpackInsertId) => {
                return Ok(CreatePlexTokenResult::Ok(PlexTokenId {
                    access_token: access_token.into(),
                }))
            }
            Err(DbErr::Query(err)) => {
                tracing::warn!("create DbErr::Query: {:?}", err);
                return Ok(CreatePlexTokenResult::Error(CreatePlexTokenError {
                    error: CreatePlexTokenErrorVariant::TokenAlreadyExists,
                }));
            }
            Err(DbErr::Exec(err)) => {
                tracing::warn!("create DbErr::Exec: {:?}", err);
                return Ok(CreatePlexTokenResult::Error(CreatePlexTokenError {
                    error: CreatePlexTokenErrorVariant::TokenAlreadyExists,
                }));
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

    pub async fn get(&self, access_token: &str) -> Result<GetPlexTokenResult> {
        Ok(
            match plex_token::Entity::find_by_id(access_token)
                .one(&self.db)
                .await
            {
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

    pub async fn list(
        &self,
        plex_user_id: Option<String>,
        plex_user_ids: Option<Vec<String>>,
    ) -> Result<Vec<plex_token::Model>> {
        Ok(plex_token::Entity::find()
            .apply_if(plex_user_id, |query, value| {
                query.filter(plex_token::Column::PlexUserId.eq(value))
            })
            .apply_if(plex_user_ids, |query, value| {
                query.filter(plex_token::Column::PlexUserId.is_in(value))
            })
            .all(&self.db)
            .await?)
    }

    pub async fn delete(&self, access_token: &str) -> Result<DeletePlexTokenResult> {
        Ok(
            match plex_token::Entity::delete_by_id(access_token)
                .exec(&self.db)
                .await
            {
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

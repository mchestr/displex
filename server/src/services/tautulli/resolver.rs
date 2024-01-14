use async_graphql::{
    Context,
    Enum,
    Object,
    SimpleObject,
    Union,
};

use reqwest::Url;

use tracing::instrument;

use crate::{
    server::cookies::{
        verify_role,
        Role,
    },
    services::tautulli::models::{
        ApiResponse,
        GetActivity,
        GetLibrary,
        ServerStatus,
        UserWatchStat,
    },
};
use anyhow::Result;

use super::models::QueryDays;

#[derive(Default)]
pub struct TautulliQuery;

#[Object]
impl TautulliQuery {
    async fn get_plex_status(
        &self,
        gql_ctx: &Context<'_>,
    ) -> async_graphql::Result<GetPlexStatusResult> {
        verify_role(gql_ctx, Role::Admin)?;
        Ok(GetPlexStatusResult::Ok(
            gql_ctx
                .data_unchecked::<TautulliService>()
                .server_status()
                .await?,
        ))
    }

    async fn get_plex_activity(
        &self,
        gql_ctx: &Context<'_>,
    ) -> async_graphql::Result<GetPlexActivityResult> {
        verify_role(gql_ctx, Role::Admin)?;
        Ok(GetPlexActivityResult::Ok(
            gql_ctx
                .data_unchecked::<TautulliService>()
                .get_activity()
                .await?,
        ))
    }
}

#[derive(Debug, Union)]
pub enum GetPlexStatusResult {
    Ok(ServerStatus),
    Err(GetPlexStatusError),
}

#[derive(Debug, SimpleObject)]
pub struct GetPlexStatusError {
    pub error: GetPlexStatusVariant,
}

#[derive(Enum, Clone, Debug, Copy, PartialEq, Eq)]
pub enum GetPlexStatusVariant {
    InternalError,
}

#[derive(Debug, Union)]
pub enum GetPlexActivityResult {
    Ok(GetActivity),
    Err(GetPlexActivityError),
}

#[derive(Debug, SimpleObject)]
pub struct GetPlexActivityError {
    pub error: GetPlexActivityVariant,
}

#[derive(Enum, Clone, Debug, Copy, PartialEq, Eq)]
pub enum GetPlexActivityVariant {
    InternalError,
}

#[derive(Debug, Clone)]
pub struct TautulliService {
    client: reqwest::Client,
    api_key: String,
    url: String,
}

impl TautulliService {
    pub fn new(client: &reqwest::Client, url: &str, api_key: &str) -> Self {
        Self {
            client: client.clone(),
            api_key: String::from(api_key),
            url: String::from(url),
        }
    }

    #[instrument(skip(self), ret, level = "debug")]
    pub async fn get_user_watch_time_stats(
        &self,
        user_id: &str,
        grouping: Option<bool>,
        query_days: Option<QueryDays>,
    ) -> Result<Vec<UserWatchStat>> {
        let user_id = user_id.to_string();
        let mut params = vec![
            ("apikey", self.api_key.clone()),
            ("cmd", "get_user_watch_time_stats".into()),
            ("user_id", user_id),
        ];
        if let Some(grouping) = grouping {
            params.push((
                "grouping",
                match grouping {
                    true => "1".into(),
                    false => "0".into(),
                },
            ));
        }
        if let Some(query_days) = query_days {
            params.push(("query_days", query_days.to_string()));
        }

        let url = Url::parse_with_params(&format!("{}/api/v2", self.url), &params)?;
        let response: ApiResponse<Vec<UserWatchStat>> =
            self.client.get(url).send().await?.json().await?;

        Ok(response.response.data)
    }

    #[instrument(skip(self), ret, level = "debug")]
    pub async fn server_status(&self) -> Result<ServerStatus> {
        let params = vec![
            ("apikey", self.api_key.clone()),
            ("cmd", "server_status".into()),
        ];

        let url = Url::parse_with_params(&format!("{}/api/v2", self.url), &params)?;
        let response: ApiResponse<ServerStatus> = self.client.get(url).send().await?.json().await?;

        Ok(response.response.data)
    }

    #[instrument(skip(self), ret, level = "debug")]
    pub async fn get_activity(&self) -> Result<GetActivity> {
        let params = vec![
            ("apikey", self.api_key.clone()),
            ("cmd", "get_activity".into()),
        ];

        let url = Url::parse_with_params(&format!("{}/api/v2", self.url), &params)?;
        let response: ApiResponse<GetActivity> = self.client.get(url).send().await?.json().await?;

        Ok(response.response.data)
    }

    #[instrument(skip(self), ret, level = "debug")]
    pub async fn get_libraries(&self) -> Result<Vec<GetLibrary>> {
        let params = vec![
            ("apikey", self.api_key.clone()),
            ("cmd", "get_libraries".into()),
        ];

        let url = Url::parse_with_params(&format!("{}/api/v2", self.url), &params)?;
        let response: ApiResponse<Vec<GetLibrary>> =
            self.client.get(url).send().await?.json().await?;

        Ok(response.response.data)
    }
}

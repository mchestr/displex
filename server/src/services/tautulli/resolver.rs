use async_graphql::{
    Context,
    Enum,
    Object,
    SimpleObject,
    Union,
};

use reqwest::Url;

use serde::{
    Deserialize,
    Serialize,
};
use tracing::instrument;

use crate::{
    server::cookies::{
        get_plex_id,
        verify_role,
        Role,
    },
    services::tautulli::models::{
        ApiResponse,
        GetActivity,
        GetLibrary,
        HomeStats,
        ServerStatus,
        StatId,
        UserTable,
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

    async fn top_media(&self, gql_ctx: &Context<'_>) -> async_graphql::Result<GetTopMediaResult> {
        let plex_user = get_plex_id(gql_ctx)?;
        let user_stats = gql_ctx
            .data_unchecked::<TautulliService>()
            .get_home_stats(Some(&plex_user))
            .await?;

        let mut top_media = TopMedia::default();
        for stat in user_stats {
            match stat.stat_id {
                StatId::TopMovies => {
                    top_media.top_movie = stat
                        .rows
                        .into_iter()
                        .map(|m| m.title)
                        .next()
                        .unwrap_or_default();
                }
                StatId::TopTv => {
                    top_media.top_show = stat
                        .rows
                        .into_iter()
                        .map(|m| m.title)
                        .next()
                        .unwrap_or_default();
                }
                _ => continue,
            }
        }
        Ok(GetTopMediaResult::Ok(top_media))
    }

    async fn leaderboard(
        &self,
        gql_ctx: &Context<'_>,
    ) -> async_graphql::Result<GetLeaderboardResult> {
        let plex_user = get_plex_id(gql_ctx)?;

        let users_table = gql_ctx
            .data_unchecked::<TautulliService>()
            .get_users_table(Some("duration"), Some("desc"))
            .await?;
        let mut position = 1;
        let mut leaderboard = Leaderboard::default();
        for user in users_table.data {
            let user_id = user.user_id.to_string();
            if user_id.eq(&plex_user) {
                leaderboard.watch_duration = user.duration;
                leaderboard.watch_count = user.plays;
                leaderboard.watch_position = position;
                break;
            }
            position = position + 1;
        }
        Ok(GetLeaderboardResult::Ok(leaderboard))
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

#[derive(Debug, Default, PartialEq, Deserialize, Serialize, SimpleObject)]
pub struct Leaderboard {
    pub watch_position: i64,
    pub watch_duration: i64,
    pub watch_count: i64,
}

#[derive(Debug, Union)]
pub enum GetLeaderboardResult {
    Ok(Leaderboard),
    Err(GetLeaderboardError),
}

#[derive(Debug, SimpleObject)]
pub struct GetLeaderboardError {
    pub error: GetLeaderboardVariant,
}

#[derive(Enum, Clone, Debug, Copy, PartialEq, Eq)]
pub enum GetLeaderboardVariant {
    InternalError,
}

#[derive(Debug, Default, PartialEq, Deserialize, Serialize, SimpleObject)]
pub struct TopMedia {
    pub show_count: i64,
    pub movie_count: i64,
    pub total_minutes: i64,
    pub top_movie: String,
    pub top_show: String,
}

#[derive(Debug, Union)]
pub enum GetTopMediaResult {
    Ok(TopMedia),
    Err(GetTopMediaError),
}

#[derive(Debug, SimpleObject)]
pub struct GetTopMediaError {
    pub error: GetTopMediaVariant,
}

#[derive(Enum, Clone, Debug, Copy, PartialEq, Eq)]
pub enum GetTopMediaVariant {
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

    #[instrument(skip(self), ret, level = "debug")]
    pub async fn get_home_stats(&self, user_id: Option<&str>) -> Result<Vec<HomeStats>> {
        let mut params = vec![
            ("apikey", self.api_key.clone()),
            ("cmd", "get_home_stats".into()),
        ];
        if let Some(user_id) = user_id {
            params.append(&mut vec![("user_id", String::from(user_id))]);
        };
        let url = Url::parse_with_params(&format!("{}/api/v2", self.url), &params)?;
        let response: ApiResponse<Vec<HomeStats>> =
            self.client.get(url).send().await?.json().await?;

        Ok(response.response.data)
    }

    #[instrument(skip(self), ret, level = "debug")]
    pub async fn get_users_table(
        &self,
        order_column: Option<&str>,
        order_dir: Option<&str>,
    ) -> Result<UserTable> {
        let mut params = vec![
            ("apikey", self.api_key.clone()),
            ("cmd", "get_users_table".into()),
            ("length", "100".into()),
        ];
        if let Some(order_column) = order_column {
            params.append(&mut vec![("order_column", String::from(order_column))]);
        };
        if let Some(order_dir) = order_dir {
            params.append(&mut vec![("order_dir", String::from(order_dir))]);
        };
        let url = Url::parse_with_params(&format!("{}/api/v2", self.url), &params)?;
        let response: ApiResponse<UserTable> = self.client.get(url).send().await?.json().await?;

        Ok(response.response.data)
    }
}

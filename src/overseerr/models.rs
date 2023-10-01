use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: i64,
    pub display_name: String,
    pub plex_username: String,
    pub plex_id: i64,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct ApiResponse<T> {
    pub results: Vec<T>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct VerifiedUserRequest {
    pub plex_user_id: String,
    pub discord_user_id: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserSettingsRequest {
    pub discord_id: Option<String>,
    pub movie_quota_limit: Option<i64>,
    pub movie_quota_days: Option<i64>,
    pub tv_quota_limit: Option<i64>,
    pub tv_quota_days: Option<i64>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserRequestSettings {
    pub movie_quota_limit: Option<i64>,
    pub movie_quota_days: Option<i64>,
    pub tv_quota_limit: Option<i64>,
    pub tv_quota_days: Option<i64>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserSettingsResponse {
    pub discord_id: Option<String>,
    pub original_language: Option<String>,
    pub region: Option<String>,
    pub watchlist_sync_movies: Option<String>,
    pub watchlist_sync_tv: Option<String>,
}

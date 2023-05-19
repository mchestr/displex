use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct ApiResponse<T> {
    pub response: ApiResult<T>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct ApiResult<T> {
    pub result: String,
    pub message: Option<String>,
    pub data: T,
}

#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct UserWatchStat {
    pub query_days: i32,
    pub total_plays: i32,
    pub total_time: i32,
}

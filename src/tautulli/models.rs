use derive_more::Display;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Display)]
pub enum QueryDays {
    #[display(fmt = "1")]
    Day,
    #[display(fmt = "7")]
    Week,
    #[display(fmt = "30")]
    Month,
    #[display(fmt = "0")]
    Total,
}

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

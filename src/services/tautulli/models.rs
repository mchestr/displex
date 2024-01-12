use async_graphql::SimpleObject;
use derive_more::Display;
use serde::{
    de::{
        self,
        Unexpected,
    },
    Deserialize,
    Deserializer,
    Serialize,
};

#[derive(Debug, Display)]
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

#[derive(Debug, Default, PartialEq, Deserialize, Serialize, SimpleObject)]
pub struct ServerStatus {
    pub connected: bool,
}

#[derive(Debug, Default, PartialEq, Deserialize, Serialize, SimpleObject)]
pub struct GetActivity {
    pub stream_count: String,
    pub stream_count_direct_play: u32,
    pub stream_count_direct_stream: u32,
    pub stream_count_transcode: u32,
    pub total_bandwidth: u32,
    pub lan_bandwidth: u32,
    pub wan_bandwidth: u32,
}

#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct GetLibrary {
    pub section_id: String,
    pub section_name: String,
    pub section_type: String,
    pub agent: String,
    pub thumb: String,
    pub count: String,
    pub child_count: Option<String>,
    pub parent_count: Option<String>,
    #[serde(deserialize_with = "bool_from_int")]
    pub is_active: bool,
}

fn bool_from_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match u8::deserialize(deserializer)? {
        0 => Ok(false),
        1 => Ok(true),
        other => Err(de::Error::invalid_value(
            Unexpected::Unsigned(other as u64),
            &"zero or one",
        )),
    }
}

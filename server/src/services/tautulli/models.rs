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

#[derive(Debug, Display)]
pub enum MediaType {
    #[display(fmt = "movie")]
    Movie,
    #[display(fmt = "episode")]
    Episode,
    #[display(fmt = "track")]
    Track,
    #[display(fmt = "live")]
    Live,
    #[display(fmt = "collection")]
    Collection,
    #[display(fmt = "playlist")]
    Playlist,
}

#[derive(Debug, Display)]
pub enum TranscodeDecision {
    #[display(fmt = "direct play")]
    DirectPlay,
    #[display(fmt = "copy")]
    Copy,
    #[display(fmt = "transcode")]
    Transcode,
}

#[derive(Debug, Display)]
pub enum OrderColumn {
    #[display(fmt = "date")]
    Date,
    #[display(fmt = "friendly_name")]
    FriendlyName,
    #[display(fmt = "ip_address")]
    IpAddress,
    #[display(fmt = "platform")]
    Platform,
    #[display(fmt = "player")]
    Player,
    #[display(fmt = "full_title")]
    FullTitle,
    #[display(fmt = "started")]
    Started,
    #[display(fmt = "paused_counter")]
    PausedCounter,
    #[display(fmt = "stopped")]
    Stopped,
    #[display(fmt = "duration")]
    Duration,
}

#[derive(Debug, Display)]
pub enum OrderDir {
    #[display(fmt = "desc")]
    Desc,
    #[display(fmt = "asc")]
    Asc,
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

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StatId {
    TopMovies,
    PopularMovies,
    TopTv,
    PopularTv,
    TopMusic,
    PopularMusic,
    LastWatched,
    TopLibraries,
    TopUsers,
    TopPlatforms,
    MostConcurrent,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct HomeStats {
    pub stat_id: StatId,
    pub rows: Vec<StatRow>,
}

#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct StatRow {
    pub title: String,
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

#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct UserTable {
    pub records_filtered: Option<i64>,
    pub records_total: Option<i64>,
    pub data: Vec<User>,
}

#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct User {
    pub user_id: i64,
    pub username: String,
    pub friendly_name: String,
    pub plays: i64,
    pub duration: i64,
}

#[derive(Debug, Default, PartialEq, Deserialize, Serialize, SimpleObject)]
pub struct GetHistory {
    pub draw: i32,
    pub records_total: Option<i32>,
    pub records_filtered: Option<i32>,
    pub total_duration: String,
    pub filter_duration: String,
    pub data: Vec<HistoryItem>,
}

#[derive(Debug, Default, PartialEq, Deserialize, Serialize, SimpleObject)]
pub struct HistoryItem {
    pub date: i64,
    pub friendly_name: String,
    pub full_title: String,
    pub grandparent_rating_key: Option<String>,
    pub grandparent_title: Option<String>,
    pub original_title: Option<String>,
    pub group_count: i32,
    pub group_ids: String,
    pub guid: String,
    pub ip_address: String,
    #[serde(deserialize_with = "bool_from_int")]
    pub live: bool,
    pub location: String,
    pub machine_id: String,
    pub media_index: Option<String>,
    pub media_type: String,
    pub originally_available_at: Option<String>,
    pub parent_media_index: Option<String>,
    pub parent_rating_key: Option<String>,
    pub parent_title: Option<String>,
    pub paused_counter: i32,
    pub percent_complete: i32,
    pub platform: String,
    pub play_duration: i64,
    pub product: String,
    pub player: String,
    pub rating_key: i64,
    pub reference_id: i64,
    #[serde(deserialize_with = "bool_from_int")]
    pub relayed: bool,
    pub row_id: i64,
    #[serde(deserialize_with = "bool_from_int")]
    pub secure: bool,
    pub session_key: Option<String>,
    pub started: i64,
    pub state: Option<String>,
    pub stopped: i64,
    pub thumb: String,
    pub title: String,
    pub transcode_decision: String,
    pub user: String,
    pub user_id: i64,
    pub watched_status: f32,
    pub year: Option<i32>,
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

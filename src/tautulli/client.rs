use anyhow::Result;
use reqwest::Url;

use super::models::{ApiResponse, UserWatchStat, QueryDays};

#[derive(Clone, Debug)]
pub struct TautulliClient {
    client: reqwest::Client,
    api_key: String,
    url: String,
}

impl TautulliClient {
    pub fn new(client: &reqwest::Client, url: &str, api_key: &str) -> TautulliClient {
        TautulliClient {
            client: client.clone(),
            api_key: String::from(api_key),
            url: String::from(url),
        }
    }

    pub async fn get_user_watch_time_stats(
        &self,
        user_id: i64,
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
}

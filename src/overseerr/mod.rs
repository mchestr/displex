pub mod models;

use anyhow::Result;
use tracing::info;

use self::models::{
    ApiResponse,
    UpdateUserSettingsRequest,
    UpdateUserSettingsResponse,
    User,
};

#[derive(Clone, Debug)]
pub struct OverseerrService {
    client: reqwest::Client,
    url: String,
    api_key: String,
}

impl OverseerrService {
    pub fn new(client: &reqwest::Client, url: &str, api_key: &str) -> OverseerrService {
        OverseerrService {
            client: client.clone(),
            url: String::from(url),
            api_key: String::from(api_key),
        }
    }

    pub async fn get_users(&self) -> Result<Vec<User>> {
        let result: ApiResponse<User> = self
            .client
            .get(format!("{}/api/v1/user", self.url))
            .header("X-Api-Key", &self.api_key)
            .query(&[("take", "100")])
            .send()
            .await?
            .json()
            .await?;
        Ok(result.results)
    }

    pub async fn set_discord_user_settings(
        &self,
        user_id: &str,
        discord_user_id: &str,
    ) -> Result<UpdateUserSettingsResponse> {
        Ok(self
            .client
            .post(format!(
                "{}/api/v1/user/{}/settings/main",
                self.url, user_id
            ))
            .header("X-Api-Key", &self.api_key)
            .json(&UpdateUserSettingsRequest {
                discord_id: Some(discord_user_id.to_owned()),
                tv_quota_limit: Some(0),
                movie_quota_limit: Some(0),
            })
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn verified_user(&self, discord_user_id: &str, plex_user_id: &str) -> Result<()> {
        info!(
            "Setting Overseerr settings... discord: {}, plex: {}",
            discord_user_id, plex_user_id
        );
        let overseerr_user = self
            .get_users()
            .await?
            .into_iter()
            .find(|u| u.plex_id.to_string() == plex_user_id);
        if let Some(user) = overseerr_user {
            info!("Found Overseerr user: {:#?}", user);
            let response = self
                .set_discord_user_settings(&user.id.to_string(), discord_user_id)
                .await?;
            info!("Successfully updated Overseerr User: {:#?}", response);
        } else {
            info!("No Overseerr user found!");
        }
        Ok(())
    }
}

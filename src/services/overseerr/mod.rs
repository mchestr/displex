pub mod models;

use anyhow::Result;
use tracing::{
    info,
    instrument,
};

use crate::{
    config::{
        AppConfig,
        RequestLimitTier,
    },
    services::tautulli::{
        models::QueryDays,
        TautulliService,
    },
};

use self::models::{
    ApiResponse,
    User,
    UserRequestSettings,
};

#[derive(Clone, Debug)]
pub struct OverseerrService {
    client: reqwest::Client,
    url: String,
    api_key: String,
    config: AppConfig,
    tautulli_service: TautulliService,
}

impl OverseerrService {
    pub fn new(
        config: &AppConfig,
        client: &reqwest::Client,
        url: &str,
        api_key: &str,
        tautulli_service: &TautulliService,
    ) -> OverseerrService {
        OverseerrService {
            client: client.clone(),
            url: String::from(url),
            api_key: String::from(api_key),
            config: config.clone(),
            tautulli_service: tautulli_service.clone(),
        }
    }

    #[instrument(skip(self), ret, level = "debug")]
    pub async fn get_users(&self) -> Result<Vec<User>> {
        let result: ApiResponse<User> = self
            .client
            .get(format!("{}/api/v1/user", self.url))
            .header("X-Api-Key", &self.api_key)
            .query(&[("take", "100")])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(result.results)
    }

    #[instrument(skip(self), ret)]
    pub async fn set_request_tier(&self, user: &User) -> Result<()> {
        let watch_stats = self
            .tautulli_service
            .get_user_watch_time_stats(
                &user.plex_id.to_string(),
                Some(true),
                Some(QueryDays::Total),
            )
            .await?;

        let latest_stat = watch_stats
            .first()
            .ok_or_else(|| anyhow::anyhow!("failed to fetch stats"))?;

        let watch_hours = latest_stat.total_time / 3600;
        let mut request_tier: Option<RequestLimitTier> = None;
        for tier in &self.config.requests_config.tiers {
            if tier.watch_hours < watch_hours.into() {
                request_tier = Some(tier.clone());
            }
        }

        if let Some(tier) = request_tier {
            tracing::info!(
                "Setting user ({}:{}) to tier {}",
                user.display_name,
                watch_hours,
                tier.name
            );
            self.set_user_request_settings(
                &user.id.to_string(),
                &UserRequestSettings {
                    movie_quota_limit: Some(tier.movie.quota_limit),
                    movie_quota_days: Some(tier.movie.quota_days),
                    tv_quota_limit: Some(tier.tv.quota_limit),
                    tv_quota_days: Some(tier.tv.quota_days),
                },
            )
            .await?;
        } else {
            tracing::info!("Setting user {} to default tier", user.display_name);
            self.set_default_request_settings(user).await?;
        }
        Ok(())
    }

    #[instrument(skip(self), ret)]
    pub async fn set_user_request_settings(
        &self,
        user_id: &str,
        request_settings: &UserRequestSettings,
    ) -> Result<()> {
        self.client
            .post(format!(
                "{}/api/v1/user/{}/settings/main",
                self.url, user_id
            ))
            .header("X-Api-Key", &self.api_key)
            .json(&request_settings)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    #[instrument(skip(self), ret)]
    pub async fn set_default_request_settings(&self, user: &User) -> Result<()> {
        self.set_user_request_settings(
            &user.id.to_string(),
            &UserRequestSettings {
                movie_quota_limit: None,
                movie_quota_days: None,
                tv_quota_limit: None,
                tv_quota_days: None,
            },
        )
        .await?;
        Ok(())
    }

    #[instrument(skip(self), ret)]
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
            self.set_request_tier(&user).await?;
        } else {
            info!("No Overseerr user found!");
        }
        Ok(())
    }
}

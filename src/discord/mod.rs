use ::oauth2::CsrfToken;
use anyhow::Result;
use reqwest::Url;
use serenity::http::Http;
use std::sync::Arc;

use self::{
    models::{
        ApplicationMetadataDefinition,
        ApplicationMetadataUpdate,
        User,
    },
    oauth2::{
        DiscordOAuth2Client,
        DiscordOAuth2Token,
    },
};

pub mod models;
pub mod oauth2;

#[derive(Clone, Debug)]
pub struct DiscordService {
    client: reqwest::Client,
    oauth2_client: DiscordOAuth2Client,
    discord_http_client: Arc<Http>,
    bot_token: String,
}

impl DiscordService {
    pub fn new(
        client: &reqwest::Client,
        discord_http_client: &Arc<Http>,
        bot_token: &str,
        client_id: u64,
        client_secret: &str,
        redirect_url: &str,
    ) -> DiscordService {
        DiscordService {
            client: client.clone(),
            discord_http_client: discord_http_client.clone(),
            oauth2_client: DiscordOAuth2Client::new(
                client.clone(),
                client_id,
                client_secret,
                Some(redirect_url),
            ),
            bot_token: bot_token.to_owned(),
        }
    }

    pub async fn application_metadata(
        &self,
        application_id: u64,
    ) -> Result<Vec<ApplicationMetadataDefinition>> {
        Ok(self
            .client
            .get(format_url(&format!(
                "/applications/{application_id}/role-connections/metadata"
            )))
            .header("Authorization", format!("Bot {}", self.bot_token))
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn register_application_metadata(
        &self,
        application_id: u64,
        metadata: Vec<ApplicationMetadataDefinition>,
    ) -> Result<Vec<ApplicationMetadataDefinition>> {
        Ok(self
            .client
            .put(format_url(&format!(
                "/applications/{application_id}/role-connections/metadata"
            )))
            .header("Authorization", format!("Bot {}", self.bot_token))
            .json(&metadata)
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn link_application(
        &self,
        application_id: u64,
        metadata: ApplicationMetadataUpdate,
        token: &str,
    ) -> Result<()> {
        self.client
            .put(format_url(&format!(
                "/users/@me/applications/{application_id}/role-connection"
            )))
            .bearer_auth(token)
            .json(&metadata)
            .send()
            .await?;
        Ok(())
    }

    pub async fn user(&self, token: &str) -> Result<User> {
        Ok(self
            .client
            .get(format_url("/users/@me"))
            .bearer_auth(token)
            .send()
            .await?
            .json()
            .await?)
    }
    pub fn authorize_url(&self) -> (Url, CsrfToken) {
        self.oauth2_client.authorize_url()
    }

    pub async fn token(&self, code: &str) -> Result<DiscordOAuth2Token> {
        self.oauth2_client.token(code).await
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<DiscordOAuth2Token> {
        self.oauth2_client.refresh_token(refresh_token).await
    }
}

fn format_url(path: &str) -> String {
    format!("https://discord.com/api/v10{path}")
}

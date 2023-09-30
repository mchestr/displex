use ::oauth2::CsrfToken;
use anyhow::Result;
use reqwest::Url;
use serenity::{
    http::Http,
    json::JsonMap,
    model::prelude::{
        GuildChannel,
        Role,
    },
};
use std::sync::Arc;

use self::{
    models::{
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
}

impl DiscordService {
    pub fn new(
        client: &reqwest::Client,
        discord_http_client: &Arc<Http>,
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
        }
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

    pub async fn unlink_application(
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

    pub async fn get_guild_roles(&self, guild_id: u64) -> Result<Vec<Role>> {
        Ok(self.discord_http_client.get_guild_roles(guild_id).await?)
    }

    pub async fn get_channels(&self, guild_id: u64) -> Result<Vec<GuildChannel>> {
        Ok(self.discord_http_client.get_channels(guild_id).await?)
    }

    pub async fn create_channel(
        &self,
        guild_id: u64,
        map: &JsonMap,
        audit_log_reason: Option<&str>,
    ) -> Result<GuildChannel> {
        Ok(self
            .discord_http_client
            .create_channel(guild_id, map, audit_log_reason)
            .await?)
    }

    pub async fn edit_channel(
        &self,
        channel_id: u64,
        map: &JsonMap,
        audit_log_reason: Option<&str>,
    ) -> Result<GuildChannel> {
        Ok(self
            .discord_http_client
            .edit_channel(channel_id, map, audit_log_reason)
            .await?)
    }
}

fn format_url(path: &str) -> String {
    format!("https://discord.com/api/v10{path}")
}

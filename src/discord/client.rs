use anyhow::Result;
use oauth2::{
    basic::{BasicClient, BasicTokenType},
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EmptyExtraTokenFields,
    RedirectUrl, Scope, TokenUrl,
};
use reqwest::Url;

use super::models::{DiscordMetaDataPush, User};

type OAuth2DiscordClient = oauth2::Client<
    oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
    oauth2::StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>,
    oauth2::basic::BasicTokenType,
    oauth2::StandardTokenIntrospectionResponse<
        oauth2::EmptyExtraTokenFields,
        oauth2::basic::BasicTokenType,
    >,
    oauth2::StandardRevocableToken,
    oauth2::StandardErrorResponse<oauth2::RevocationErrorResponseType>,
>;

pub type DiscordOAuth2Token = oauth2::StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>;

pub struct DiscordClient {
    oauth_client: OAuth2DiscordClient,
    client: reqwest::Client,

    client_id: String,
}

impl DiscordClient {
    pub fn new(
        client: reqwest::Client,
        client_id: &str,
        client_secret: &str,
        redirect_url: &str,
    ) -> DiscordClient {
        let cid = ClientId::new(String::from(client_id));
        let client_secret = ClientSecret::new(String::from(client_secret));

        // Create an OAuth2 client
        let auth_url = AuthUrl::new("https://discord.com/api/oauth2/authorize".to_string())
            .expect("Invalid authorization endpoint URL");
        let token_url = TokenUrl::new("https://discord.com/api/oauth2/token".to_string())
            .expect("Invalid token endpoint URL");
        let redirect_url = String::from(redirect_url);
        let oauth_client = BasicClient::new(cid, Some(client_secret), auth_url, Some(token_url))
            .set_redirect_uri(RedirectUrl::new(redirect_url).expect("Invalid redirect URL"));
        DiscordClient {
            client: client,
            oauth_client: oauth_client,
            client_id: String::from(client_id),
        }
    }

    pub fn authorize_url(&self) -> (Url, CsrfToken) {
        self.oauth_client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new(String::from("identify")))
            .add_scope(Scope::new(String::from("role_connections.write")))
            .url()
    }

    pub async fn token(&self, code: &str) -> Result<DiscordOAuth2Token> {
        let resp = self
            .oauth_client
            .exchange_code(AuthorizationCode::new(String::from(code)))
            .request_async(oauth2::reqwest::async_http_client)
            .await?;
        Ok(resp)
    }

    pub async fn link_application(&self, token: &str) -> Result<()> {
        self.client
            .put(format!(
                "https://discord.com/api/v10/users/@me/applications/{}/role-connection",
                self.client_id
            ))
            .bearer_auth(&token)
            .json(&DiscordMetaDataPush {
                platform_name: String::from("mikeflix"),
            })
            .send()
            .await?;
        Ok(())
    }

    pub async fn user(&self, token: &str) -> Result<User> {
        log::info!(
            "{}",
            self.client
                .get("https://discord.com/api/v10/users/@me")
                .bearer_auth(&token)
                .send()
                .await?
                .text()
                .await?
        );
        Ok(self
            .client
            .get("https://discord.com/api/v10/users/@me")
            .bearer_auth(&token)
            .send()
            .await?
            .json()
            .await?)
    }
}

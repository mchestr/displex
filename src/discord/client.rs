use anyhow::Result;
use oauth2::{
    basic::{
        BasicClient,
        BasicTokenType,
    },
    AuthUrl,
    AuthorizationCode,
    ClientId,
    ClientSecret,
    CsrfToken,
    EmptyExtraTokenFields,
    HttpRequest,
    HttpResponse,
    RedirectUrl,
    RefreshToken,
    Scope,
    TokenUrl,
};
use reqwest::Url;

use super::models::{
    ApplicationMetadataDefinition,
    ApplicationMetadataUpdate,
    User,
};

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

#[derive(Clone, Debug)]
pub struct DiscordClient {
    oauth_client: OAuth2DiscordClient,
    client: reqwest::Client,
    client_id: String,
    bot_token: String,
    server_id: String,
    channel_id: String,
}

impl DiscordClient {
    pub fn new(
        client: &reqwest::Client,
        client_id: &str,
        client_secret: &str,
        redirect_url: &str,
        bot_token: &str,
        server_id: &str,
        channel_id: &str,
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
            client: client.clone(),
            oauth_client,
            client_id: String::from(client_id),
            bot_token: String::from(bot_token),
            server_id: String::from(server_id),
            channel_id: String::from(channel_id),
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
            .request_async(|request| self.send(request))
            .await?;
        Ok(resp)
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<DiscordOAuth2Token> {
        Ok(self
            .oauth_client
            .exchange_refresh_token(&RefreshToken::new(String::from(refresh_token)))
            .request_async(|request| self.send(request))
            .await?)
    }

    pub async fn link_application(
        &self,
        token: &str,
        metadata: ApplicationMetadataUpdate,
    ) -> Result<()> {
        self.client
            .put(format!(
                "https://discord.com/api/v10/users/@me/applications/{}/role-connection",
                self.client_id
            ))
            .bearer_auth(token)
            .json(&metadata)
            .send()
            .await?;
        Ok(())
    }

    pub async fn user(&self, token: &str) -> Result<User> {
        log::info!(
            "{}",
            self.client
                .get("https://discord.com/api/v10/users/@me")
                .bearer_auth(token)
                .send()
                .await?
                .text()
                .await?
        );
        Ok(self
            .client
            .get("https://discord.com/api/v10/users/@me")
            .bearer_auth(token)
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn application_metadata(&self) -> Result<Vec<ApplicationMetadataDefinition>> {
        Ok(self
            .client
            .get(format!(
                "https://discord.com/api/v10/applications/{}/role-connections/metadata",
                self.client_id
            ))
            .header("Authorization", format!("Bot {}", self.bot_token))
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn register_application_metadata(
        &self,
        metadata: Vec<ApplicationMetadataDefinition>,
    ) -> Result<Vec<ApplicationMetadataDefinition>> {
        Ok(self
            .client
            .put(format!(
                "https://discord.com/api/v10/applications/{}/role-connections/metadata",
                self.client_id
            ))
            .header("Authorization", format!("Bot {}", self.bot_token))
            .json(&metadata)
            .send()
            .await?
            .json()
            .await?)
    }

    pub fn generate_auth_success_url(&self) -> String {
        format!(
            "discord://discordapp.com/channels/{}/{}",
            self.server_id, self.channel_id
        )
    }

    async fn send(
        &self,
        request: HttpRequest,
    ) -> std::result::Result<HttpResponse, reqwest::Error> {
        let mut request_builder = self
            .client
            .request(request.method, request.url.as_str())
            .body(request.body);
        for (name, value) in &request.headers {
            request_builder = request_builder.header(name.as_str(), value.as_bytes());
        }
        let request = request_builder.build()?;

        let response = self.client.execute(request).await?;

        let status_code = response.status();
        let headers = response.headers().to_owned();
        let chunks = response.bytes().await?;
        Ok(HttpResponse {
            status_code,
            headers,
            body: chunks.to_vec(),
        })
    }
}

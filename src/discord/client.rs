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

type OAuth2Client = oauth2::Client<
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
pub struct DiscordOAuth2Client {
    client: reqwest::Client,
    oauth_client: OAuth2Client,
}

impl DiscordOAuth2Client {
    pub fn new(
        client: reqwest::Client,
        client_id: u64,
        client_secret: &str,
        redirect_url: Option<&str>,
    ) -> DiscordOAuth2Client {
        let cid = ClientId::new(client_id.to_string());
        let cs = ClientSecret::new(String::from(client_secret));

        // Create an OAuth2 client
        let auth_url = AuthUrl::new("https://discord.com/api/oauth2/authorize".to_string())
            .expect("Invalid authorization endpoint URL");
        let token_url = TokenUrl::new("https://discord.com/api/oauth2/token".to_string())
            .expect("Invalid token endpoint URL");
        let mut oauth_client = BasicClient::new(cid, Some(cs), auth_url, Some(token_url));

        if let Some(redirect_url) = redirect_url {
            oauth_client = oauth_client.set_redirect_uri(
                RedirectUrl::new(redirect_url.into()).expect("Invalid redirect URL"),
            );
        }

        DiscordOAuth2Client {
            client,
            oauth_client,
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

#[derive(Clone, Debug)]
pub struct DiscordClient {
    client: reqwest::Client,
    bot_token: String,
}

impl DiscordClient {
    pub fn new(client: reqwest::Client, bot_token: &str) -> DiscordClient {
        DiscordClient {
            client,
            bot_token: String::from(bot_token),
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
}

fn format_url(path: &str) -> String {
    format!("https://discord.com/api/v10{path}")
}

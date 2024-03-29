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
    RevocationUrl,
    Scope,
    StandardRevocableToken,
    TokenUrl,
};
use reqwest::Url;
use std::str::{
    self,
    FromStr,
};
use tracing::instrument;

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
    ) -> DiscordOAuth2Client {
        let cid = ClientId::new(client_id.to_string());
        let cs = ClientSecret::new(String::from(client_secret));

        // Create an OAuth2 client
        let auth_url = AuthUrl::new("https://discord.com/api/oauth2/authorize".to_string())
            .expect("Invalid authorization endpoint URL");
        let token_url = TokenUrl::new("https://discord.com/api/oauth2/token".to_string())
            .expect("Invalid token endpoint URL");
        let oauth_client = BasicClient::new(cid, Some(cs), auth_url, Some(token_url))
            .set_revocation_uri(
                RevocationUrl::new(String::from("https://discord.com/api/oauth2/token/revoke"))
                    .expect("invalid revocation url"),
            );

        DiscordOAuth2Client {
            client,
            oauth_client,
        }
    }

    #[instrument(skip(self), ret)]
    pub fn authorize_url(&self, redirect_url: &str) -> (Url, CsrfToken) {
        let redirect_url = RedirectUrl::from_url(Url::from_str(redirect_url).unwrap());
        self.oauth_client
            .authorize_url(CsrfToken::new_random)
            .set_redirect_uri(std::borrow::Cow::Owned(redirect_url))
            .add_scope(Scope::new(String::from("identify")))
            .add_scope(Scope::new(String::from("role_connections.write")))
            .url()
    }

    #[instrument(skip(self), ret, err)]
    pub async fn token(&self, code: &str, redirect_url: &str) -> Result<DiscordOAuth2Token> {
        let redirect_url = RedirectUrl::from_url(Url::from_str(redirect_url).unwrap());
        let resp = self
            .oauth_client
            .exchange_code(AuthorizationCode::new(String::from(code)))
            .set_redirect_uri(std::borrow::Cow::Owned(redirect_url))
            .request_async(|request| self.send(request))
            .await?;
        Ok(resp)
    }

    #[instrument(skip(self), ret, err)]
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<DiscordOAuth2Token> {
        Ok(self
            .oauth_client
            .exchange_refresh_token(&RefreshToken::new(String::from(refresh_token)))
            .request_async(|request| self.send(request))
            .await?)
    }

    #[instrument(skip(self), ret, err)]
    pub async fn revoke_token(&self, refresh_token: &str) -> Result<()> {
        Ok(self
            .oauth_client
            .revoke_token(StandardRevocableToken::RefreshToken(RefreshToken::new(
                String::from(refresh_token),
            )))?
            .request_async(|request| self.send(request))
            .await?)
    }

    #[instrument(skip(self), fields(status, body), err)]
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

        tracing::Span::current().record("status", status_code.as_str());
        tracing::Span::current().record("body", str::from_utf8(&chunks).unwrap_or("decode_error"));
        tracing::info!("got Discord OAuth2 response");
        Ok(HttpResponse {
            status_code,
            headers,
            body: chunks.to_vec(),
        })
    }
}

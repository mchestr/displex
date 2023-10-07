pub mod constants;
pub mod models;

use anyhow::Result;
use reqwest::Url;
use tracing::instrument;

use self::{
    constants::{
        PLEX_TV_APP_URL,
        PLEX_TV_AUTH_PATH,
        PLEX_TV_PIN_PATH,
        PLEX_TV_RESOURCES_PATH,
        PLEX_TV_URL,
        PLEX_TV_USER_PATH,
    },
    models::{
        AuthContext,
        AuthDevice,
        AuthQueryParams,
        CreatePinResponse,
        Device,
        PinClaimResponse,
        User,
    },
};

#[derive(Clone, Debug)]
pub struct PlexService {
    client: reqwest::Client,
    redirect_url: String,
    client_id: String,
}

impl PlexService {
    pub fn new(client: &reqwest::Client, client_id: &str, redirect_url: &str) -> PlexService {
        PlexService {
            client: client.clone(),
            redirect_url: String::from(redirect_url),
            client_id: String::from(client_id),
        }
    }

    #[instrument(skip(self), ret)]
    pub async fn get_pin(&self) -> Result<CreatePinResponse> {
        let form_params = [
            ("strong", "true"),
            ("X-Plex-Product", &self.client_id),
            ("X-Plex-Client-Identifier", &self.client_id),
        ];

        Ok(self
            .client
            .post(format!("{PLEX_TV_URL}{PLEX_TV_PIN_PATH}"))
            .form(&form_params)
            .send()
            .await?
            .json()
            .await?)
    }

    #[instrument(skip(self), ret)]
    pub async fn generate_auth_url(&self, pin_id: u64, pin_code: &str) -> Result<String> {
        let qs = AuthQueryParams {
            client_id: String::from(&self.client_id),
            code: String::from(pin_code),
            context: AuthContext {
                device: AuthDevice {
                    product: String::from(&self.client_id),
                },
            },
            forward_url: format!("{}?id={}&code={}", &self.redirect_url, pin_id, pin_code),
        };
        let params = serde_qs::to_string(&qs)?;

        let mut url = Url::parse(&format!("{PLEX_TV_APP_URL}{PLEX_TV_AUTH_PATH}"))?;
        url.set_fragment(Some(&format!("?{}", &params)));

        tracing::debug!("generate_auth_url: {}", url);
        Ok(url.to_string())
    }

    #[instrument(skip(self), ret)]
    pub async fn pin_claim(&self, pin_id: u64, pin_code: &str) -> Result<PinClaimResponse> {
        let params: [(&str, &str); 3] = [
            ("X-Plex-Product", &self.client_id),
            ("X-Plex-Client-Identifier", &self.client_id),
            ("code", pin_code),
        ];
        let url = Url::parse_with_params(
            &format!("{PLEX_TV_URL}{PLEX_TV_PIN_PATH}/{pin_id}"),
            &params,
        )?;

        tracing::debug!("pin_claim: {}", url);
        Ok(self.client.get(url).send().await?.json().await?)
    }

    #[instrument(skip(self), ret)]
    pub async fn user(&self, auth_token: &str) -> Result<User> {
        let user_params: [(&str, &str); 3] = [
            ("X-Plex-Token", auth_token),
            ("X-Plex-Product", &self.client_id),
            ("X-Plex-Client-Identifier", &self.client_id),
        ];
        Ok(self
            .client
            .get(&format!("{PLEX_TV_URL}{PLEX_TV_USER_PATH}"))
            .query(&user_params)
            .send()
            .await?
            .json()
            .await?)
    }

    #[instrument(skip(self), ret)]
    pub async fn get_devices(&self, auth_token: &str) -> Result<Vec<Device>> {
        let user_params: [(&str, &str); 3] = [
            ("X-Plex-Token", auth_token),
            ("X-Plex-Product", &self.client_id),
            ("X-Plex-Client-Identifier", &self.client_id),
        ];
        Ok(self
            .client
            .get(&format!("{PLEX_TV_URL}{PLEX_TV_RESOURCES_PATH}"))
            .query(&user_params)
            .send()
            .await?
            .json()
            .await?)
    }
}

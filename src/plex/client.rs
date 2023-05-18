use reqwest::Url;

use super::{
    constants::{
        PLEX_TV_APP_URL, PLEX_TV_AUTH_PATH, PLEX_TV_PIN_PATH, PLEX_TV_RESOURCES_PATH, PLEX_TV_URL,
        PLEX_TV_USER_PATH,
    },
    models::{
        AuthContext, AuthDevice, AuthQueryParams, CreatePinResponse, Device, PinClaimResponse, User,
    },
};

#[derive(Clone)]
pub struct PlexClient {
    client: reqwest::Client,
    redirect_url: String,
    client_id: String,
}

impl PlexClient {
    pub fn new_with_client(
        client: reqwest::Client,
        client_id: &str,
        redirect_url: &str,
    ) -> PlexClient {
        PlexClient {
            client: client,
            redirect_url: String::from(redirect_url),
            client_id: String::from(client_id),
        }
    }

    pub async fn get_pin(&self) -> CreatePinResponse {
        let form_params = [
            ("strong", "true"),
            ("X-Plex-Product", &self.client_id),
            ("X-Plex-Client-Identifier", &self.client_id),
        ];

        self.client
            .post(format!("{}{}", PLEX_TV_URL, PLEX_TV_PIN_PATH))
            .form(&form_params)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }

    pub async fn generate_auth_url(&self, pin_id: u64, pin_code: &str) -> String {
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
        let params = serde_qs::to_string(&qs).unwrap();

        let mut url = Url::parse(&format!("{}{}", PLEX_TV_APP_URL, PLEX_TV_AUTH_PATH)).unwrap();
        url.set_fragment(Some(&format!("?{}", &params)));

        log::debug!("generate_auth_url: {}", url);
        url.to_string()
    }

    pub async fn pin_claim(&self, pin_id: u64, pin_code: &str) -> PinClaimResponse {
        let params: [(&str, &str); 3] = [
            ("X-Plex-Product", &self.client_id),
            ("X-Plex-Client-Identifier", &self.client_id),
            ("code", pin_code),
        ];
        let url = Url::parse_with_params(
            &format!("{}{}/{}", PLEX_TV_URL, PLEX_TV_PIN_PATH, pin_id),
            &params,
        )
        .unwrap();

        log::debug!("pin_claim: {}", url);
        self.client
            .get(url)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }

    #[allow(dead_code)]
    pub async fn get_user(&self, auth_token: &str) -> User {
        let user_params: [(&str, &str); 3] = [
            ("X-Plex-Token", auth_token),
            ("X-Plex-Product", &self.client_id),
            ("X-Plex-Client-Identifier", &self.client_id),
        ];
        self.client
            .get(&format!("{}{}", PLEX_TV_URL, PLEX_TV_USER_PATH))
            .query(&user_params)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }

    pub async fn get_devices(&self, auth_token: &str) -> Vec<Device> {
        let user_params: [(&str, &str); 3] = [
            ("X-Plex-Token", auth_token),
            ("X-Plex-Product", &self.client_id),
            ("X-Plex-Client-Identifier", &self.client_id),
        ];
        self.client
            .get(&format!("{}{}", PLEX_TV_URL, PLEX_TV_RESOURCES_PATH))
            .query(&user_params)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }
}

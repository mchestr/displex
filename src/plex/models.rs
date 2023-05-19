use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CreatePinResponse {
    pub id: u64,
    pub code: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Location {
    pub code: String,
    pub european_union_member: bool,
    pub continent_code: String,
    pub country: String,
    pub city: String,
    pub time_zone: String,
    pub postal_code: String,
    pub in_privacy_restricted_country: bool,
    pub subdivisions: String,
    pub coordinates: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PinClaimResponse {
    pub id: u64,
    pub code: String,
    pub product: String,
    pub trusted: bool,
    pub qr: String,
    pub client_identifier: String,
    pub location: Location,
    pub expires_in: u16,
    pub created_at: String,
    pub expires_at: String,
    pub auth_token: String,
    pub new_registration: bool,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthQueryParams {
    #[serde(rename(serialize = "clientID"))]
    pub client_id: String,
    pub code: String,
    pub forward_url: String,
    pub context: AuthContext,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct AuthContext {
    pub device: AuthDevice,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct AuthDevice {
    pub product: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub auth_token: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Subscription {
    pub active: bool,
    pub subscribed_at: String,
    pub status: String,
    pub payment_service: String,
    pub plan: String,
    pub features: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub auto_select_audio: bool,
    pub default_audio_language: String,
    pub default_subtitle_language: String,
    pub auto_select_subtitle: u32,
    pub default_subtitle_accessibility: u32,
    pub default_subtitle_forced: u32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    pub identifier: String,
    pub endpoint: String,
    pub token: Option<String>,
    pub secret: Option<String>,
    pub status: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub name: String,
    pub client_identifier: String,
}

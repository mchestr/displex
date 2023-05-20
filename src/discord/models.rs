use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct ApplicationMetadataUpdate {
    pub platform_name: String,
    pub metadata: ApplicationMetadata,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ApplicationMetadata {
    pub total_watches: i32,
    pub hours_watched: i32,
    pub is_subscriber: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub global_name: Option<String>,
    pub avatar: Option<String>,
    pub discriminator: String,
    pub public_flags: i64,
    pub flags: i64,
    pub banner: Option<String>,
    pub banner_color: Option<i64>,
    pub accent_color: Option<i64>,
    pub locale: String,
    pub mfa_enabled: bool,
    pub verified: Option<bool>,
    pub premium_type: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApplicationMetadataDefinition {
    pub key: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub type_: u8,
}

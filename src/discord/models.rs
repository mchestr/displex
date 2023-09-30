use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ApplicationMetadataUpdate {
    pub platform_name: String,
    pub platform_username: Option<String>,
    pub metadata: ApplicationMetadata,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ApplicationMetadata {
    pub watched_hours: i32,
    pub is_subscribed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApplicationMetadataDefinition {
    pub key: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub type_: u8,
}

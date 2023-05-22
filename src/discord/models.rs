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
    pub public_flags: Option<i64>,
    pub flags: Option<i64>,
    pub banner: Option<String>,
    pub banner_color: Option<i64>,
    pub accent_color: Option<i64>,
    pub locale: Option<String>,
    pub mfa_enabled: Option<bool>,
    pub verified: Option<bool>,
    pub premium_type: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApplicationMetadataDefinition {
    pub key: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub type_: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Application {
    pub bot_public: bool,
    pub bot_require_code_grant: bool,
    pub cover_image: Option<String>,
    pub description: String,
    pub guild_id: Option<String>,
    pub icon: Option<String>,
    pub id: String,
    pub name: String,
    pub owner: Option<Owner>,
    pub primary_sku_id: Option<String>,
    pub slug: Option<String>,
    pub team: Option<Team>,
    pub verify_key: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Owner {
    pub avatar: Option<String>,
    pub discriminator: String,
    pub flags: Option<i64>,
    pub id: String,
    pub username: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Team {
    pub icon: Option<String>,
    pub id: String,
    pub members: Vec<Member>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Member {
    pub membership_state: i64,
    pub permissions: Vec<String>,
    pub team_id: String,
    pub user: User,
}

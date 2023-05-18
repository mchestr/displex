use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct DiscordMetaDataPush {
    pub platform_name: String,
}

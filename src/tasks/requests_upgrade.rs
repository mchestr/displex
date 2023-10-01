use anyhow::Result;

use crate::services::AppServices;

pub async fn run(services: &AppServices) -> Result<()> {
    let overseerr_users = services.overseerr_service.get_users().await?;
    for user in overseerr_users {
        services.overseerr_service.set_request_tier(&user).await?;
    }
    Ok(())
}

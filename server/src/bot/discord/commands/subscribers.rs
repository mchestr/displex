use chrono::{
    Duration,
    DurationRound,
    Utc,
};
use poise::serenity_prelude as serenity;

use crate::services::AppServices;

/// Display all current subscribers and their token expiration times
#[poise::command(prefix_command, owners_only)]
pub async fn subscriber_tokens(
    ctx: poise::Context<'_, AppServices, serenity::Error>,
) -> Result<(), serenity::Error> {
    let now = Utc::now().duration_round(Duration::minutes(1)).unwrap();
    let subscribers: Vec<(String, String, bool)> = ctx
        .data()
        .discord_users_service
        .list_subscriber_tokens()
        .await
        .unwrap()
        .into_iter()
        .map(|(discord, token)| {
            (
                discord.username,
                token.map_or(String::from("-"), |t| {
                    humantime::format_duration(
                        (t.expires_at.duration_round(Duration::minutes(1)).unwrap() - now)
                            .to_std()
                            .unwrap(),
                    )
                    .to_string()
                }),
                true,
            )
        })
        .collect();
    let embed = serenity::CreateEmbed::new()
        .title("Current Subscribers")
        .fields(subscribers)
        .footer(serenity::CreateEmbedFooter::new("powered by displex"))
        .timestamp(Utc::now());

    ctx.channel_id()
        .send_message(ctx, serenity::CreateMessage::new().add_embed(embed))
        .await?;
    Ok(())
}

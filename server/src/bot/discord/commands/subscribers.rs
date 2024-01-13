use chrono::{
    Duration,
    DurationRound,
    Utc,
};
use serenity::{
    builder::{
        CreateEmbed,
        CreateEmbedFooter,
        CreateMessage,
    },
    framework::standard::{
        macros::command,
        CommandResult,
    },
    model::prelude::*,
    prelude::*,
};

use crate::services::AppServices;

#[command]
#[aliases("tokens")]
#[owners_only]
pub async fn subscriber_tokens(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    if let Some(services) = data.get::<AppServices>() {
        let now = Utc::now().duration_round(Duration::minutes(1)).unwrap();
        let subscribers: Vec<(String, String, bool)> = services
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
        let embed = CreateEmbed::new()
            .title("Current Subscribers")
            .fields(subscribers)
            .footer(CreateEmbedFooter::new("powered by displex"))
            .timestamp(Utc::now());

        msg.channel_id
            .send_message(ctx, CreateMessage::new().add_embed(embed))
            .await
            .unwrap();
    } else {
        msg.reply(ctx, "There was a problem getting the app services")
            .await?;
    }
    Ok(())
}

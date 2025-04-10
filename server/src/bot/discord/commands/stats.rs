use crate::services::{
    discord_user::resolver::{
        SummaryDiscordUserResult,
        UserSummaryBy,
    },
    tautulli::models::QueryDays,
    AppServices,
};
use chrono::Utc;
use poise::serenity_prelude as serenity;

/// Show your watch time statistics from Tautulli
#[poise::command(prefix_command, slash_command)]
pub async fn stats(
    ctx: poise::Context<'_, AppServices, serenity::Error>,
) -> Result<(), serenity::Error> {
    let user = ctx.author();
    if let Ok(summary) = ctx
        .data()
        .discord_users_service
        .summary(&UserSummaryBy::Id(user.id.get().to_string()))
        .await
    {
        if let SummaryDiscordUserResult::Ok(summary) = summary {
            if summary.summary.plex_users.is_empty() {
                ctx.say("No Plex accounts linked to your Discord account.")
                    .await?;
                return Ok(());
            }

            let plex_user = summary.summary.plex_users.get(0).unwrap().id.clone();

            // Get watch stats for different time periods
            let today_stats = match ctx
                .data()
                .tautulli_service
                .get_user_watch_time_stats(&plex_user, None, Some(QueryDays::Day))
                .await
            {
                Ok(stats) => stats,
                Err(err) => {
                    ctx.say(format!("Error retrieving watch stats: {}", err))
                        .await?;
                    return Ok(());
                }
            };

            let week_stats = match ctx
                .data()
                .tautulli_service
                .get_user_watch_time_stats(&plex_user, None, Some(QueryDays::Week))
                .await
            {
                Ok(stats) => stats,
                Err(err) => {
                    ctx.say(format!("Error retrieving watch stats: {}", err))
                        .await?;
                    return Ok(());
                }
            };

            let month_stats = match ctx
                .data()
                .tautulli_service
                .get_user_watch_time_stats(&plex_user, None, Some(QueryDays::Month))
                .await
            {
                Ok(stats) => stats,
                Err(err) => {
                    ctx.say(format!("Error retrieving watch stats: {}", err))
                        .await?;
                    return Ok(());
                }
            };

            let all_time_stats = match ctx
                .data()
                .tautulli_service
                .get_user_watch_time_stats(&plex_user, None, Some(QueryDays::Total))
                .await
            {
                Ok(stats) => stats,
                Err(err) => {
                    ctx.say(format!("Error retrieving watch stats: {}", err))
                        .await?;
                    return Ok(());
                }
            };

            // Format time durations nicely
            fn format_duration(seconds: i32) -> String {
                let hours = seconds / 3600;
                let minutes = (seconds % 3600) / 60;

                if hours > 0 {
                    format!("{}h {}m", hours, minutes)
                } else {
                    format!("{}m", minutes)
                }
            }

            // Create an embed with the data
            let embed = serenity::CreateEmbed::new()
                .title(format!("Watch Stats for {}", user.name))
                .color(0x00A8FC) // Plex blue color
                .thumbnail(
                    user.avatar_url()
                        .unwrap_or_else(|| user.default_avatar_url()),
                )
                .field(
                    "Today",
                    format!(
                        "**Plays:** {}\n**Time Watched:** {}",
                        today_stats.get(0).map_or(0, |s| s.total_plays),
                        today_stats
                            .get(0)
                            .map_or(String::from("0m"), |s| format_duration(s.total_time))
                    ),
                    true,
                )
                .field(
                    "This Week",
                    format!(
                        "**Plays:** {}\n**Time Watched:** {}",
                        week_stats.get(0).map_or(0, |s| s.total_plays),
                        week_stats
                            .get(0)
                            .map_or(String::from("0m"), |s| format_duration(s.total_time))
                    ),
                    true,
                )
                .field(
                    "This Month",
                    format!(
                        "**Plays:** {}\n**Time Watched:** {}",
                        month_stats.get(0).map_or(0, |s| s.total_plays),
                        month_stats
                            .get(0)
                            .map_or(String::from("0m"), |s| format_duration(s.total_time))
                    ),
                    true,
                )
                .field(
                    "All Time",
                    format!(
                        "**Plays:** {}\n**Time Watched:** {}",
                        all_time_stats.get(0).map_or(0, |s| s.total_plays),
                        all_time_stats
                            .get(0)
                            .map_or(String::from("0m"), |s| format_duration(s.total_time))
                    ),
                    true,
                )
                .footer(serenity::CreateEmbedFooter::new("powered by displex"))
                .timestamp(Utc::now());

            ctx.channel_id()
                .send_message(
                    ctx,
                    serenity::CreateMessage::new()
                        .content(format!("📊 **Watch History for {}**", user.name))
                        .add_embed(embed),
                )
                .await?;
        } else {
            send_error_message(&ctx).await?;
        }
    } else {
        send_error_message(&ctx).await?;
    }
    Ok(())
}

async fn send_error_message(
    ctx: &poise::Context<'_, AppServices, serenity::Error>,
) -> Result<(), serenity::Error> {
    let embed = serenity::CreateEmbed::new()
    .title("Account Linking Required")
    .description("To access your Plex statistics, you need to link your Discord account to your Plex account.")
    .color(0xE5A00D) // Warning color (amber)
    .field(
        "How to Link Your Account",
        "1. Go to Server Settings\n2. Click on Linked Roles\n3. Follow the steps to connect your Plex account",
        false,
    )
    .field(
        "Is it safe?",
        "Yes! The linking process is secure and only provides us with the minimal permissions needed to associate your accounts.",
        false,
    )
    .footer(serenity::CreateEmbedFooter::new("powered by displex"))
    .timestamp(Utc::now());

    ctx.channel_id()
        .send_message(
            ctx,
            serenity::CreateMessage::new()
                .content("⚠️ **Account Linking Required**")
                .add_embed(embed),
        )
        .await?;
    Ok(())
}

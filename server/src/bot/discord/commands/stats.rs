use crate::{
    bot::discord::utils::{
        send_error,
        ErrorSeverity,
    },
    services::{
        discord_user::resolver::{
            SummaryDiscordUserResult,
            UserSummaryBy,
        },
        tautulli::models::QueryDays,
        AppServices,
    },
};
use anyhow::anyhow;
use chrono::Utc;
use poise::{
    serenity_prelude as serenity,
    CreateReply,
};

/// Show your watch time statistics from Tautulli
#[poise::command(slash_command)]
pub async fn stats(
    ctx: poise::Context<'_, AppServices, serenity::Error>,
) -> Result<(), serenity::Error> {
    let user = ctx.author();
    if let Ok(SummaryDiscordUserResult::Ok(summary)) = ctx
        .data()
        .discord_users_service
        .summary(&UserSummaryBy::Id(user.id.get().to_string()))
        .await
    {
        if summary.summary.plex_users.is_empty() {
            send_error(
                &ctx,
                anyhow!("User has no linked Plex account"),
                Some("An error has occurred"),
                ErrorSeverity::Critical,
            )
            .await?;
            return Ok(());
        }

        let plex_user = summary.summary.plex_users.first().unwrap().id.clone();

        // Get watch stats for different time periods
        let today_stats = match ctx
            .data()
            .tautulli_service
            .get_user_watch_time_stats(&plex_user, None, Some(QueryDays::Day))
            .await
        {
            Ok(stats) => stats,
            Err(err) => {
                send_error(
                    &ctx,
                    err,
                    Some("An error has occurred"),
                    ErrorSeverity::Critical,
                )
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
                send_error(
                    &ctx,
                    err,
                    Some("An error has occurred"),
                    ErrorSeverity::Critical,
                )
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
                send_error(
                    &ctx,
                    err,
                    Some("An error has occurred"),
                    ErrorSeverity::Critical,
                )
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
                send_error(
                    &ctx,
                    err,
                    Some("An error has occurred"),
                    ErrorSeverity::Critical,
                )
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
                    today_stats.first().map_or(0, |s| s.total_plays),
                    today_stats
                        .first()
                        .map_or(String::from("0m"), |s| format_duration(s.total_time))
                ),
                true,
            )
            .field(
                "This Week",
                format!(
                    "**Plays:** {}\n**Time Watched:** {}",
                    week_stats.first().map_or(0, |s| s.total_plays),
                    week_stats
                        .first()
                        .map_or(String::from("0m"), |s| format_duration(s.total_time))
                ),
                true,
            )
            .field(
                "This Month",
                format!(
                    "**Plays:** {}\n**Time Watched:** {}",
                    month_stats.first().map_or(0, |s| s.total_plays),
                    month_stats
                        .first()
                        .map_or(String::from("0m"), |s| format_duration(s.total_time))
                ),
                true,
            )
            .field(
                "All Time",
                format!(
                    "**Plays:** {}\n**Time Watched:** {}",
                    all_time_stats.first().map_or(0, |s| s.total_plays),
                    all_time_stats
                        .first()
                        .map_or(String::from("0m"), |s| format_duration(s.total_time))
                ),
                true,
            )
            .footer(serenity::CreateEmbedFooter::new("powered by displex"))
            .timestamp(Utc::now());

        ctx.send(
            CreateReply::default()
                .content(format!("📊 **Watch History for {}**", user.name))
                .embed(embed),
        )
        .await?;
    } else {
        let auth_url = format!(
            "https://{}/auth/discord?next=/auth/plex?next=discord://-/channels/{}/@home",
            ctx.data().config.http.hostname,
            ctx.data().config.discord.server_id
        );

        let embed = serenity::CreateEmbed::new()
        .title("Account Linking & Verification Required")
        .description("To access your Plex statistics, you need to link your Discord account to your Plex account and confirm you're a subscriber.")
        .color(0xE5A00D) // Warning color (amber)
        .field(
            "How to Link Your Account",
            format!("Click [here]({}) to link your accounts\n\nOr follow these steps manually:\n1. Go to Server Settings\n2. Click on Linked Roles\n3. Follow the steps to connect your Plex account", auth_url),
            false,
        )
        .field(
            "Is it safe?",
            "Yes! The linking process is secure and only provides us with the minimal permissions needed to associate and verify your accounts.",
            false,
        )
        .footer(serenity::CreateEmbedFooter::new("powered by displex"))
        .timestamp(Utc::now());

        ctx.send(CreateReply::default().embed(embed)).await?;
    }
    Ok(())
}

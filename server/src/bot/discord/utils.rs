use chrono::Utc;
use poise::serenity_prelude as serenity;
use tracing::{
    error,
    warn,
};

use crate::services::AppServices;

/// Send a formatted error message to Discord and log the error
///
/// # Arguments
///
/// * `ctx` - The context of the command
/// * `error` - The error that occurred
/// * `public_message` - Optional custom message to display to the user (if None, a generic message
///   is used)
/// * `severity` - The severity of the error (determines color and logging level)
///
/// # Returns
///
/// * `Result<(), serenity::Error>` - Ok if the message was sent, Err otherwise
pub async fn send_error<E: std::fmt::Display>(
    ctx: &poise::Context<'_, AppServices, serenity::Error>,
    error: E,
    public_message: Option<&str>,
    severity: ErrorSeverity,
) -> Result<(), serenity::Error> {
    // Log the error appropriately
    match severity {
        ErrorSeverity::Critical => {
            error!("{} {} {}", ctx.author().id, error, "Discord command error")
        }
        ErrorSeverity::Warning => warn!(
            "{} {} {}",
            ctx.author().id,
            error,
            "Discord command warning"
        ),
        ErrorSeverity::Info => {
            tracing::info!("{} {} {}", ctx.author().id, error, "Discord command info")
        }
    }

    // Create the embed with appropriate colors
    let embed = serenity::CreateEmbed::new()
        .title(match severity {
            ErrorSeverity::Critical => "❌ Error Occurred",
            ErrorSeverity::Warning => "⚠️ Warning",
            ErrorSeverity::Info => "ℹ️ Information",
        })
        .description(public_message.unwrap_or(match severity {
            ErrorSeverity::Critical => {
                "An error occurred while processing your request. Our team has been notified."
            }
            ErrorSeverity::Warning => "There was an issue processing your request.",
            ErrorSeverity::Info => "Please note the following information.",
        }))
        .color(match severity {
            ErrorSeverity::Critical => 0xE74C3C, // Red
            ErrorSeverity::Warning => 0xE5A00D,  // Amber
            ErrorSeverity::Info => 0x3498DB,     // Blue
        })
        .footer(serenity::CreateEmbedFooter::new("powered by displex"))
        .timestamp(Utc::now());

    // Add a reference ID for tracking critical errors
    let embed = if matches!(severity, ErrorSeverity::Critical) {
        let reference_id = format!(
            "{:x}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        );

        embed.field("Reference ID", &reference_id, false)
    } else {
        embed
    };

    // Send the message
    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}

/// The severity level of an error, affects color and logging
pub enum ErrorSeverity {
    /// Critical errors (red) - unexpected failures that need attention
    Critical,
    /// Warnings (amber) - issues that don't prevent operation but are concerning
    Warning,
    /// Information (blue) - not really errors, but informational messages
    Info,
}

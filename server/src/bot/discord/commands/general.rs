use poise::serenity_prelude as serenity;

use crate::services::AppServices;

type Error = serenity::Error;

/// Simple ping command to check if the bot is running
#[poise::command(prefix_command)]
pub async fn ping(ctx: poise::Context<'_, AppServices, Error>) -> Result<(), Error> {
    ctx.say("Pong!").await?;
    Ok(())
}

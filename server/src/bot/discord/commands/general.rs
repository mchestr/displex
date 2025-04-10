use poise::serenity_prelude as serenity;

type Error = serenity::Error;

#[poise::command(prefix_command)]
pub async fn ping(ctx: poise::Context<'_, (), Error>) -> Result<(), Error> {
    ctx.say("Pong!").await?;
    Ok(())
}

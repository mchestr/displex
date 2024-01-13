use serenity::{
    framework::standard::{
        macros::command,
        Args,
        CommandResult,
    },
    gateway::ActivityData,
    model::prelude::*,
    prelude::*,
};

#[command]
#[owners_only]
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;
    Ok(())
}

#[command]
#[owners_only]
pub async fn status(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    ctx.set_activity(Some(ActivityData::watching(args.rest())));
    msg.reply(ctx, format!("set status: {}", args.rest()))
        .await?;
    Ok(())
}

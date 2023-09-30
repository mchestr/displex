use serenity::{
    framework::standard::{
        macros::command,
        CommandResult,
    },
    model::prelude::*,
    prelude::*,
};

#[command]
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;
    Ok(())
}

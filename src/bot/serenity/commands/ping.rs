use serenity::{prelude::*, model::prelude::*, framework::standard::{CommandResult, macros::command}};


#[command]
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}
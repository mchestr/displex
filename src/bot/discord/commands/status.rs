use serenity::{
    framework::standard::{
        macros::command,
        CommandResult,
    },
    gateway::ActivityData,
    model::prelude::*,
    prelude::*,
};

#[command]
#[owners_only]
pub async fn status(ctx: &Context, msg: &Message) -> CommandResult {
    let status = msg.content.replace("~status", "");
    ctx.set_activity(Some(ActivityData::watching(&status)));
    msg.reply(ctx, format!("set status: {}", &status)).await?;
    Ok(())
}

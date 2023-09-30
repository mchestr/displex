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
    tracing::info!("testing got here!");
    msg.reply(ctx, "Pong!").await?;

    let result = ctx
        .http
        .remove_member_role(
            434573248206733312,
            229726290972901377,
            1107886515360632902,
            Some("removed for testing"),
        )
        .await;
    tracing::info!("Result: {:?}", result);

    Ok(())
}

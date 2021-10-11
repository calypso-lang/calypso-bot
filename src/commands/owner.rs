use tracing::info;

use super::prelude::*;
use crate::data::ShardManagerContainer;

#[group]
#[commands(stop, status)]
#[owners_only]
#[help_available(false)]
pub struct Owner;

#[command]
#[aliases("sd", "shutdown", "quit", "exit")]
#[description = "Shut down the bot."]
#[owners_only]
#[help_available(false)]
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    let mgr = data
        .get::<ShardManagerContainer>()
        .expect("get ShardManagerContainer");
    info!("Shutting down by request of {}.", msg.author.tag());
    msg.react(ctx, '\u{2705}').await?;
    mgr.lock().await.shutdown_all().await;

    Ok(())
}

#[command]
#[aliases("act", "activity")]
#[description = "Set the bot's status."]
#[owners_only]
#[help_available(false)]
async fn status(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let s = args.rest();

    let parsed = match serde_json::from_str(s) {
        Ok(s) => s,
        Err(e) => {
            msg.reply_ping(&ctx, format!("Bad parse: `{}`", e)).await?;
            return Err(e.into());
        }
    };

    ctx.set_presence(Some(parsed), OnlineStatus::Online).await;
    msg.react(&ctx, '\u{2705}').await?;

    Ok(())
}

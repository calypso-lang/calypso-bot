use super::prelude::*;

#[group]
#[commands(ping)]
pub struct General;

#[command]
#[aliases("pingpong", "pong")]
#[description = "Ping the bot to test it. Helpful for making sure the bot's not broken."]
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    // let latency = {
    //     let data = ctx.data.read().await;
    //     let mgr = data
    //         .get::<ShardManagerContainer>()
    //         .expect("get ShardManagerContainer");
    //     let mgr = mgr.lock().await;
    //     let runners = mgr.runners.lock().await;
    //     let info = runners
    //         .get(&ShardId(ctx.shard_id))
    //         .expect("get ShardRunnerInfo");
    //     info.latency
    //         .map(|l| format!("{}ms", l.as_millis()))
    //         .unwrap_or_else(|| "Could not get gateway latency.".to_string())
    // };

    let mut sent = msg
        .channel_id
        .send_message(&ctx, |b| {
            b.embed(|e| {
                begin(e)
                    .title("Ping... \u{1f3d3}")
                    .description("One second, I'm gathering data.")
            })
        })
        .await?;

    let ts = sent.timestamp;
    sent.edit(ctx, |b| {
        b.embed(|e| {
            begin(e)
                .title("Ping... Pong! \u{1f3d3}")
                .description(format!(
                    r#"**Command Recv-Response Latency**:
                    {}ms"#,
                    (ts - msg.timestamp).num_milliseconds()
                ))
        })
    })
    .await?;

    Ok(())
}

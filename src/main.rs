#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]

use std::{collections::HashSet, sync::Arc};

use color_eyre::eyre::{self, WrapErr};
use serenity::{
    framework::{standard::Delimiter, StandardFramework},
    http::Http,
    prelude::Mutex,
    Client,
};
use tokio::{fs, sync::mpsc::unbounded_channel};
use tracing::trace;
use tracing_subscriber::EnvFilter;

pub mod commands;
pub mod config;
pub mod data;
pub mod embeds;
pub mod events;

#[allow(clippy::wildcard_imports)]
use commands::*;
use config::Config;
use data::{ConfigContainer, OwnersContainer, ShardManagerContainer};
use events::Handler;

use crate::data::SysFPrettyContainer;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let s = fs::read_to_string("./config.toml")
        .await
        .wrap_err("While reading config file")?;
    let cfg: Config = toml::from_str(&s).wrap_err("While parsing config file")?;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(&cfg.general.log))
        .init();

    trace!("Started tracing!");

    let http = Http::new_with_token(&cfg.discord.token);

    let info = http
        .get_current_application_info()
        .await
        .wrap_err("While getting application info")?;
    let mut owners = HashSet::new();
    if let Some(team) = info.team {
        owners.insert(team.owner_user_id);
    } else {
        owners.insert(info.owner.id);
    }

    let bot_id = http
        .get_current_user()
        .await
        .context("While getting bot id")?
        .id;

    let framework = StandardFramework::new()
        .configure(|c| {
            c.on_mention(Some(bot_id))
                .case_insensitivity(true)
                .prefix("calbot:")
                .delimiters(Vec::<Delimiter>::new())
                .owners(owners.clone())
                .no_dm_prefix(true)
        })
        .help(&HELP)
        .group(&GENERAL_GROUP)
        .group(&OWNER_GROUP)
        .group(&SYSF_GROUP);

    drop(http);

    let mut client = Client::builder(&cfg.discord.token)
        .application_id(cfg.discord.appid)
        .event_handler(Handler)
        .framework(framework)
        .await
        .context("While creating the bot client")?;

    let (sysf_tx, ppc_rx) = unbounded_channel();
    let (ppc_tx, sysf_rx) = unbounded_channel();

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<OwnersContainer>(Arc::new(owners));
        data.insert::<ConfigContainer>(Arc::new(cfg));
        data.insert::<SysFPrettyContainer>(Arc::new((ppc_tx, Mutex::new(ppc_rx))));
    }

    let shard_manager = Arc::clone(&client.shard_manager);

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctr+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    tokio::select! {
        res = client.start() => {
            res.wrap_err("While running client")?;
        }
        _ = sysf::pretty_printing(sysf_tx, sysf_rx) => {}
    }

    Ok(())
}

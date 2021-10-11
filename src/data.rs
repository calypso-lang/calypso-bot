use std::{collections::HashSet, sync::Arc};

use serenity::{client::bridge::gateway::ShardManager, model::prelude::*, prelude::*};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::{commands::sysf::ToPrettyPrint, config::Config};

pub struct ShardManagerContainer;
pub struct OwnersContainer;
pub struct ConfigContainer;
pub struct SysFPrettyContainer;

impl TypeMapKey for OwnersContainer {
    type Value = Arc<HashSet<UserId>>;
}

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

impl TypeMapKey for ConfigContainer {
    type Value = Arc<Config>;
}

impl TypeMapKey for SysFPrettyContainer {
    type Value = Arc<(
        UnboundedSender<ToPrettyPrint>,
        Mutex<UnboundedReceiver<String>>,
    )>;
}

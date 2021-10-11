use serde::Deserialize;
use serenity::model::prelude::Activity;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub discord: Discord,
    #[serde(default = "Default::default")]
    pub general: General,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Discord {
    pub token: String,
    pub appid: u64,
    #[serde(default = "default_status")]
    pub status: Activity,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct General {
    #[serde(default = "Default::default")]
    pub log: String,
}

#[must_use]
pub fn default_status() -> Activity {
    Activity::playing("\u{1fa90}\u{2728}")
}

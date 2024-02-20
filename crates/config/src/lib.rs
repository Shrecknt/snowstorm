#![feature(lazy_cell)]

use serde::Deserialize;
use smart_default::SmartDefault;
use std::{
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
};

static CONFIG: LazyLock<Arc<Config>> =
    LazyLock::new(|| Arc::new(Config::new("Snowstorm.toml").expect("Snowstorm.toml not found")));

#[derive(Deserialize)]
pub struct Config {
    pub database_url: String,
    pub testing_data: Option<PathBuf>,

    #[serde(default)]
    pub web: WebConfig,

    #[serde(default)]
    pub jwt: JwtConfig,

    #[serde(default)]
    pub scanner: ScannerConfig,

    #[serde(default)]
    pub bot: BotConfig,
}

impl Config {
    pub fn new(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&contents).expect("Unable to read config file"))
    }

    pub fn get() -> Arc<Self> {
        CONFIG.clone()
    }
}

#[derive(Deserialize, SmartDefault)]
pub struct WebConfig {
    #[serde(default = "_true")]
    #[default = false]
    pub enabled: bool,
    pub listen_uri: String,
    pub domain: String,
    pub oauth: OauthConfig,
}

#[derive(Deserialize, SmartDefault)]
pub struct JwtConfig {
    pub secret: String,
}

#[derive(Deserialize, SmartDefault)]
pub struct ScannerConfig {
    #[serde(default = "_true")]
    #[default = false]
    pub enabled: bool,
    pub interface_name: String,
    pub task_size_sanity_limit: usize,
}

#[derive(Deserialize, SmartDefault)]
pub struct BotConfig {
    #[serde(default = "_true")]
    #[default = false]
    pub enabled: bool,
    pub token: String,
    pub bot_id: String,
}

#[derive(Deserialize, SmartDefault)]
pub struct OauthConfig {
    #[serde(default)]
    pub discord: OauthDiscordConfig,

    #[serde(default)]
    pub forgejo: OauthForgejoConfig,
}

#[derive(Deserialize, SmartDefault)]
pub struct OauthDiscordConfig {
    #[serde(default = "_true")]
    #[default = false]
    pub enabled: bool,
    pub redirect_uri: String,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Deserialize, SmartDefault)]
pub struct OauthForgejoConfig {
    #[serde(default = "_true")]
    #[default = false]
    pub enabled: bool,
    pub redirect_uri: String,
    pub client_id: String,
    pub client_secret: String,
    pub base_authorize_uri: String,
    pub base_token_uri: String,
    pub user_api_uri: String,
}

fn _true() -> bool {
    true
}

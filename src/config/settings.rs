use std::env;
use std::sync::OnceLock;

use anyhow::{Context, Result};

static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub db: DbConfig,
}

#[derive(Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Clone)]
pub struct DbConfig {
    pub database_url: String,
}

pub fn init() -> Result<&'static Config> {
    let api_url = env::var("API_ADDR").context("API_ADDR must be set")?;
    let port = env::var("API_PORT").context("PORT must be set")?.parse::<u16>().context("PORT must be a valid u16")?;

    let database_url = env::var("DATABASE_URL").context("DATABASE_URL must be set")?;

    let config = Config {
        server: ServerConfig {
            host: api_url,
            port,
        },
        db: DbConfig { database_url },
    };

    Ok(CONFIG.get_or_init(|| config))
}

pub fn get() -> &'static Config {
    CONFIG.get().expect("Config not initialized — call settings::init() first")
}
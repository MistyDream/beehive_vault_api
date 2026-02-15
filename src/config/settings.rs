use std::env;

use anyhow::{Context, Result};

#[derive(Clone)]
pub struct Config {
    pub server: ServerConfig,
}

#[derive(Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

pub fn load_config() -> Result<Config> {
    let api_url = env::var("API_ADDR").context("API_ADDR must be set")?;
    let port = env::var("API_PORT").context("PORT must be set")?.parse::<u16>().context("PORT must be a valid u16")?;

    Ok(Config {
        server: ServerConfig {
            host: api_url,
            port,
        }
    })
}
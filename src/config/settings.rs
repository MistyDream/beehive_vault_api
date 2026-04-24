use std::env;
use std::sync::OnceLock;

use anyhow::{Context, Result};

static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub db: DbConfig,
    pub cors: CorsConfig,
    pub scheduler: SchedulerConfig,
    pub auth: AuthConfig,
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

#[derive(Clone)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
}

#[derive(Clone)]
pub struct SchedulerConfig {
    /// When false, the price-batch cron scheduler is not started. Intended
    /// for local development and CI where every process launch would otherwise
    /// hit Yahoo three times a day.
    pub enabled: bool,
}

#[derive(Clone)]
pub struct AuthConfig {
    /// Shared bearer token required on every `/v1/*` request. Injected
    /// server-side by the Nuxt proxy; healthchecks are exempted by scope.
    pub api_key: String,
}

pub fn init() -> Result<&'static Config> {
    let api_url = env::var("API_ADDR").context("API_ADDR must be set")?;
    let port = env::var("API_PORT").context("PORT must be set")?.parse::<u16>().context("PORT must be a valid u16")?;

    let database_url = env::var("DATABASE_URL").context("DATABASE_URL must be set")?;

    // Production MUST override CORS_ALLOWED_ORIGINS with explicit HTTPS origins.
    let allowed_origins = env::var("CORS_ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://beehive-vault.fr,http://localhost:3000".to_string())
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let scheduler_enabled = env::var("PRICE_SCHEDULER_ENABLED")
        .ok()
        .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(true);

    let api_key = env::var("API_KEY")
        .context("API_KEY must be set — shared bearer token required on /v1/*")?;
    if api_key.trim().is_empty() {
        anyhow::bail!("API_KEY must not be empty");
    }

    let config = Config {
        server: ServerConfig {
            host: api_url,
            port,
        },
        db: DbConfig { database_url },
        cors: CorsConfig { allowed_origins },
        scheduler: SchedulerConfig { enabled: scheduler_enabled },
        auth: AuthConfig { api_key },
    };

    Ok(CONFIG.get_or_init(|| config))
}

pub fn get() -> &'static Config {
    CONFIG.get().expect("Config not initialized — call settings::init() first")
}
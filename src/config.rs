use std::{
    env,
    net::{AddrParseError, SocketAddr},
    num::NonZeroU32,
};

use thiserror::Error;

const DEFAULT_BIND_ADDRESS: &str = "127.0.0.1:8080";
const DEFAULT_DATABASE_MAX_CONNECTIONS: u32 = 5;

#[derive(Debug, Clone)]
pub struct Settings {
    pub bind_address: SocketAddr,
    pub database_url: String,
    pub database_max_connections: u32,
}

impl Settings {
    pub fn from_env() -> Result<Self, SettingsError> {
        let bind_address_value =
            env::var("APP_BIND_ADDRESS").unwrap_or_else(|_| DEFAULT_BIND_ADDRESS.to_owned());
        let bind_address =
            bind_address_value
                .parse()
                .map_err(|source| SettingsError::InvalidBindAddress {
                    value: bind_address_value,
                    source,
                })?;
        let database_url =
            env::var("DATABASE_URL").map_err(|_| SettingsError::MissingDatabaseUrl)?;
        let database_max_connections = env::var("DATABASE_MAX_CONNECTIONS")
            .map(|value| {
                value
                    .parse::<NonZeroU32>()
                    .map(NonZeroU32::get)
                    .map_err(|source| SettingsError::InvalidDatabaseMaxConnections {
                        value,
                        source,
                    })
            })
            .unwrap_or(Ok(DEFAULT_DATABASE_MAX_CONNECTIONS))?;

        Ok(Self {
            bind_address,
            database_url,
            database_max_connections,
        })
    }
}

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("DATABASE_URL must be set")]
    MissingDatabaseUrl,
    #[error("APP_BIND_ADDRESS '{value}' is invalid")]
    InvalidBindAddress {
        value: String,
        #[source]
        source: AddrParseError,
    },
    #[error("DATABASE_MAX_CONNECTIONS '{value}' must be a positive integer")]
    InvalidDatabaseMaxConnections {
        value: String,
        #[source]
        source: std::num::ParseIntError,
    },
}

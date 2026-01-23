use anyhow::Result;
use std::error::Error;
use std::fmt;
use tracing::info;

pub struct Config {
    pub url: String,
    pub api_key: String,
}

pub enum ConfigError {
    NotFound(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NotFound(field) => write!(f, "{} must be set", field),
        }
    }
}

impl fmt::Debug for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Error for ConfigError {}

impl Config {
    pub fn load_env() -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok();

        let url = std::env::var("TIINGO_WS_URL")
            .map_err(|_| ConfigError::NotFound("TIINGO_WS_URL".to_string()))?;

        let api_key = std::env::var("TIINGO_API_KEY")
            .map_err(|_| ConfigError::NotFound("TIINGO_API_KEY".to_string()))?;
        info!("Configuration loaded successfully");

        Ok(Config { url, api_key })
    }
}

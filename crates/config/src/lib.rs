pub mod auth;
pub mod database;
pub mod log;
pub mod rabbitmq;

use config::ConfigError;
use dokusho_core::SourceLanguage;
use serde::Deserialize;
use std::str::FromStr;
use std::{env, time::Duration};
use url::Url;

pub use crate::{
    auth::AuthConfig, database::DatabaseConfig, log::LogConfig, rabbitmq::RabbitMqConfig,
};

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
    pub sources: SourcesConfig,
    pub database: DatabaseConfig,
    pub log: LogConfig,
    pub rabbitmq: RabbitMqConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SourcesConfig {
    pub flaresolverr: Option<FlaresolverrConfig>,
    pub enabled_languages: Vec<SourceLanguage>,
    pub enable_mock: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FlaresolverrConfig {
    pub url: Url,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        // Server
        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let server_port: u16 = env::var("SERVER_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(8080);
        let server_cors_origins: Vec<String> = env::var("SERVER_CORS_ORIGINS")
            .ok()
            .map(|v| {
                v.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_else(|| vec!["*".to_string()]);

        let server = ServerConfig {
            host: server_host,
            port: server_port,
            cors_origins: server_cors_origins,
        };

        // Sources
        let sources_enabled_languages: Vec<SourceLanguage> = env::var("SOURCES_ENABLED_LANGUAGES")
            .ok()
            .map(|v| {
                v.split(',')
                    .filter_map(|s| SourceLanguage::from_str(s.trim()).ok())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let sources_enable_mock: Option<bool> = env::var("SOURCES_ENABLE_MOCK")
            .ok()
            .and_then(|v| v.parse::<bool>().ok());
        let flaresolverr = env::var("SOURCES_FLARESOLVERR_URL")
            .ok()
            .and_then(|v| Url::parse(&v).ok())
            .map(|url| FlaresolverrConfig { url });
        let sources = SourcesConfig {
            flaresolverr,
            enabled_languages: sources_enabled_languages,
            enable_mock: sources_enable_mock,
        };

        // Database
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| ConfigError::Message("DATABASE_URL is required".into()))?;
        let connections_max: u32 = env::var("DATABASE_CONNECTIONS_MAX")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);
        let connections_min: u32 = env::var("DATABASE_CONNECTIONS_MIN")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);
        let database = DatabaseConfig {
            url: database_url,
            connections_max,
            connections_min,
        };

        // Logging
        use crate::log::{LogConfig, LogFormat, LogLevel};
        let log_level = env::var("LOG_LEVEL")
            .ok()
            .and_then(|v| v.parse::<LogLevel>().ok())
            .unwrap_or(LogLevel::Info);
        let log_format = env::var("LOG_FORMAT")
            .ok()
            .and_then(|v| v.parse::<LogFormat>().ok())
            .unwrap_or(LogFormat::Pretty);
        let log = LogConfig {
            level: log_level,
            format: log_format,
        };

        // Auth
        let base_url = env::var("BASE_URL")
            .map_err(|_| ConfigError::Message("BASE_URL is required".into()))?;
        let auth_issuer_url = env::var("AUTH_ISSUER_URL")
            .map_err(|_| ConfigError::Message("AUTH_ISSUER_URL is required".into()))?;
        let auth_public_client_id = env::var("AUTH_PUBLIC_CLIENT_ID")
            .map_err(|_| ConfigError::Message("AUTH_PUBLIC_CLIENT_ID is required".into()))?;
        let auth_client_id = env::var("AUTH_CLIENT_ID")
            .map_err(|_| ConfigError::Message("AUTH_CLIENT_ID is required".into()))?;
        let auth_client_secret = env::var("AUTH_CLIENT_SECRET")
            .map_err(|_| ConfigError::Message("AUTH_CLIENT_SECRET is required".into()))?;
        let auth_group_admin = env::var("AUTH_GROUP_ADMIN").unwrap_or_else(|_| "admin".to_string());
        let auth_group_user = env::var("AUTH_GROUP_USER").unwrap_or_else(|_| "user".to_string());
        let auth_token_cache_ttl_secs: u64 = env::var("AUTH_TOKEN_CACHE_TTL_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(300);
        let auth_jwks_cache_ttl_secs: u64 = env::var("AUTH_JWKS_CACHE_TTL_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(600);

        let auth = AuthConfig {
            issuer_url: auth_issuer_url,
            public_client_id: auth_public_client_id,
            client_id: auth_client_id,
            client_secret: auth_client_secret,
            base_url,
            group_admin: auth_group_admin,
            group_user: auth_group_user,
            token_cache_ttl: Duration::from_secs(auth_token_cache_ttl_secs),
            jwks_cache_ttl: Duration::from_secs(auth_jwks_cache_ttl_secs),
        };

        let rabbitmq_uri = env::var("RABBITMQ_URI")
            .map_err(|_| ConfigError::Message("RABBITMQ_URI is required".into()))?;
        let rabbitmq_prefetch = env::var("RABBITMQ_PREFETCH")
            .ok()
            .and_then(|v| v.parse::<u16>().ok());
        let rabbitmq = RabbitMqConfig {
            uri: rabbitmq_uri,
            prefetch: rabbitmq_prefetch,
        };

        let config = AppConfig {
            server,
            auth,
            sources,
            database,
            log,
            rabbitmq,
        };

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.auth.issuer_url.is_empty() {
            return Err(ConfigError::Message("AUTH_ISSUER_URL is required".into()));
        }
        if self.auth.client_id.is_empty() {
            return Err(ConfigError::Message("AUTH_CLIENT_ID is required".into()));
        }
        if self.auth.client_secret.is_empty() {
            return Err(ConfigError::Message(
                "AUTH_CLIENT_SECRET is required".into(),
            ));
        }
        if self.auth.base_url.is_empty() {
            return Err(ConfigError::Message("BASE_URL is required".into()));
        }
        if self.rabbitmq.uri.is_empty() {
            return Err(ConfigError::Message("RABBITMQ_URI is required".into()));
        }
        // No callback or redirect list required; clients handle provider redirects

        Ok(())
    }
}

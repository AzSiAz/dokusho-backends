use config::{Config, ConfigError, Environment};
use dokusho_core::SourceLanguage;
use serde::Deserialize;
use std::env;
use url::Url;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
    pub sources: SourcesConfig,
    pub database: DatabaseConfig,
    pub log: LoggingConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: String,
    #[serde(skip)]
    pub oauth_callback_url: String,
    pub allowed_redirect_urls: Vec<String>,
    pub jwt_secret: String,
    pub jwt_expiry_hours: u64,
    pub group_admin: String,
    pub group_user: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SourcesConfig {
    flaresolverr: Option<FlaresolverrConfig>,
    enabled_languages: Vec<SourceLanguage>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FlaresolverrConfig {
    pub url: Url,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub connections_max: u32,
    pub connections_min: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let s = Config::builder()
            // Start with default values
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 8080)?
            .set_default("server.cors_origins", vec!["*"])?
            .set_default("auth.jwt_expiry_hours", 24)?
            .set_default("auth.group_admin", "admin")?
            .set_default("auth.group_user", "user")?
            .set_default("database.connections_max", 10)?
            .set_default("database.connections_min", 1)?
            .set_default("logging.level", "info")?
            .set_default("logging.format", "pretty")?
            .add_source(
                Environment::default()
                    .try_parsing(true)
                    .separator("_")
                    .list_separator(","),
            )
            .set_override_option("logging.level", env::var("LOG_LEVEL").ok())?
            .build()?;

        let mut config: AppConfig = s.try_deserialize()?;

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
        if self.auth.oauth_callback_url.is_empty() {
            return Err(ConfigError::Message(
                "AUTH_OAUTH_CALLBACK_URL is required".into(),
            ));
        }
        if self.auth.allowed_redirect_urls.is_empty() {
            return Err(ConfigError::Message(
                "AUTH_ALLOWED_REDIRECT_URLS is required".into(),
            ));
        }

        Ok(())
    }
}

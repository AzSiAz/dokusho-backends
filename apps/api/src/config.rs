use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
    pub sources: SourcesConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    pub enabled: bool,
    pub issuer_url: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub redirect_url: Option<String>,
    pub jwt_secret: String,
    pub jwt_expiry_hours: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SourcesConfig {
    pub use_flaresolver: bool,
    pub flaresolver_url: Option<String>,
    pub api_key_enabled: bool,
    pub api_key: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let s = Config::builder()
            // Start with default values
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 8080)?
            .set_default("server.cors_origins", vec!["*"])?
            .set_default("auth.enabled", false)?
            .set_default("auth.jwt_expiry_hours", 24)?
            .set_default("sources.use_flaresolver", true)?
            .set_default("sources.api_key_enabled", true)?
            .set_default("database.max_connections", 10)?
            .set_default("database.min_connections", 1)?
            .set_default("logging.level", "info")?
            .set_default("logging.format", "pretty")?
            // Add in settings from files
            .add_source(File::with_name("config/default").required(false))
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            // Add in settings from environment variables (with prefix "DOKUSHO")
            .add_source(
                Environment::with_prefix("DOKUSHO")
                    .separator("_")
                    .try_parsing(true),
            )
            // Override specific settings from legacy environment variables
            .set_override_option("server.port", env::var("PORT").ok())?
            .set_override_option("logging.level", env::var("LOG_LEVEL").ok())?
            .set_override_option("sources.use_flaresolver", env::var("SOURCE_USE_FLARESOLVER").ok())?
            .set_override_option("sources.flaresolver_url", env::var("SOURCE_FLARESOLVER_URL").ok())?
            .set_override_option("sources.api_key_enabled", env::var("SOURCE_USE_API_KEY").ok())?
            .set_override_option("sources.api_key", env::var("SOURCE_API_KEY").ok())?
            .set_override_option("database.url", env::var("DATABASE_URL").ok())?
            .set_override_option("auth.jwt_secret", env::var("JWT_SECRET").ok())?
            .build()?;

        let mut config: AppConfig = s.try_deserialize()?;

        // Generate JWT secret if not provided
        if config.auth.jwt_secret.is_empty() {
            config.auth.jwt_secret = uuid::Uuid::new_v4().to_string();
            tracing::warn!("No JWT_SECRET provided, generated a random one. This will invalidate tokens on restart!");
        }

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.sources.use_flaresolver && self.sources.flaresolver_url.is_none() {
            return Err(ConfigError::Message(
                "FlareSolver is enabled but FLARESOLVER_URL is not set".into(),
            ));
        }

        if self.sources.api_key_enabled && self.sources.api_key.is_none() {
            return Err(ConfigError::Message(
                "API key authentication is enabled but API_KEY is not set".into(),
            ));
        }

        if self.auth.enabled {
            if self.auth.issuer_url.is_none() {
                return Err(ConfigError::Message(
                    "Auth is enabled but ISSUER_URL is not set".into(),
                ));
            }
            if self.auth.client_id.is_none() {
                return Err(ConfigError::Message(
                    "Auth is enabled but CLIENT_ID is not set".into(),
                ));
            }
            if self.auth.client_secret.is_none() {
                return Err(ConfigError::Message(
                    "Auth is enabled but CLIENT_SECRET is not set".into(),
                ));
            }
            if self.auth.redirect_url.is_none() {
                return Err(ConfigError::Message(
                    "Auth is enabled but REDIRECT_URL is not set".into(),
                ));
            }
        }

        Ok(())
    }
}
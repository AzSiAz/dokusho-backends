use config::{Config, ConfigError, Environment};
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
    pub oauth_callback_url: Option<String>,
    pub allowed_redirect_urls: Vec<String>,
    pub jwt_secret: String,
    pub jwt_expiry_hours: u64,
    pub group_admin: String,
    pub group_user: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SourcesConfig {
    pub use_flaresolver: bool,
    pub flaresolver_url: Option<String>,
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
        let s = Config::builder()
            // Start with default values
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 8080)?
            .set_default("server.cors_origins", vec!["*"])?
            .set_default("auth.enabled", false)?
            .set_default("auth.jwt_expiry_hours", 24)?
            .set_default("auth.group_admin", "admin")?
            .set_default("auth.group_user", "user")?
            .set_default(
                "auth.oauth_callback_url",
                "http://localhost:8080/auth/callback",
            )?
            .set_default(
                "auth.allowed_redirect_urls",
                vec!["http://localhost:3000/auth/success"],
            )?
            .set_default("sources.use_flaresolver", false)?
            .set_default("database.url", "postgres://localhost/dokusho")?
            .set_default("database.max_connections", 10)?
            .set_default("database.min_connections", 1)?
            .set_default("logging.level", "info")?
            .set_default("logging.format", "pretty")?
            .add_source(Environment::default().separator("_").try_parsing(true))
            // Override specific settings from legacy environment variables
            .set_override_option("server.port", env::var("PORT").ok())?
            .set_override_option("logging.level", env::var("LOG_LEVEL").ok())?
            .set_override_option(
                "sources.use_flaresolver",
                env::var("SOURCE_USE_FLARESOLVER").ok(),
            )?
            .set_override_option(
                "sources.flaresolver_url",
                env::var("SOURCE_FLARESOLVER_URL").ok(),
            )?
            .set_override_option("database.url", env::var("DATABASE_URL").ok())?
            .set_override_option("auth.jwt_secret", env::var("AUTH_JWT_SECRET").ok())?
            .set_override_option("auth.group_admin", env::var("AUTH_GROUP_ADMIN").ok())?
            .set_override_option("auth.group_user", env::var("AUTH_GROUP_USER").ok())?
            .set_override_option("auth.issuer_url", env::var("AUTH_ISSUER_URL").ok())?
            .set_override_option("auth.client_id", env::var("AUTH_CLIENT_ID").ok())?
            .set_override_option("auth.client_secret", env::var("AUTH_CLIENT_SECRET").ok())?
            .set_override_option(
                "auth.oauth_callback_url",
                env::var("AUTH_OAUTH_CALLBACK_URL").ok(),
            )?
            .set_override_option(
                "auth.allowed_redirect_urls",
                env::var("AUTH_ALLOWED_REDIRECT_URLS").ok().map(|urls| {
                    urls.split(',')
                        .map(|s| s.trim().to_string())
                        .collect::<Vec<_>>()
                }),
            )?
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
            if self.auth.oauth_callback_url.is_none() {
                return Err(ConfigError::Message(
                    "Auth is enabled but OAUTH_CALLBACK_URL is not set".into(),
                ));
            }
            if self.auth.allowed_redirect_urls.is_empty() {
                return Err(ConfigError::Message(
                    "Auth is enabled but ALLOWED_REDIRECT_URLS is not set".into(),
                ));
            }
        }

        Ok(())
    }
}

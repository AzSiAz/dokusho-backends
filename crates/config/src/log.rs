use serde::Deserialize;
use strum::{Display, EnumString};

#[derive(Debug, Deserialize, Clone, EnumString, Display)]
pub enum LogFormat {
    #[strum(serialize = "pretty")]
    Pretty,
    #[strum(serialize = "compact")]
    Compact,
    #[strum(serialize = "json")]
    Json,
}

#[derive(Debug, Deserialize, Clone, EnumString, Display)]
pub enum LogLevel {
    #[strum(serialize = "trace")]
    Trace,
    #[strum(serialize = "debug")]
    Debug,
    #[strum(serialize = "info")]
    Info,
    #[strum(serialize = "warn")]
    Warn,
    #[strum(serialize = "error")]
    Error,
}

fn default_log_level() -> LogLevel {
    LogLevel::Info
}

fn default_log_format() -> LogFormat {
    LogFormat::Pretty
}

#[derive(Debug, Deserialize, Clone)]
pub struct LogConfig {
    #[serde(default = "default_log_level")]
    pub level: LogLevel,
    #[serde(default = "default_log_format")]
    pub format: LogFormat,
}

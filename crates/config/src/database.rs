use serde::Deserialize;

fn default_connections_max() -> u32 {
    10
}

fn default_connections_min() -> u32 {
    1
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    #[serde(default = "default_connections_max")]
    pub connections_max: u32,
    #[serde(default = "default_connections_min")]
    pub connections_min: u32,
}

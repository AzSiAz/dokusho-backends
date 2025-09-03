use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    // OpenID Connect issuer configuration
    pub issuer_url: String,
    pub public_client_id: String,
    pub client_id: String,
    pub client_secret: String,
    // Public base URL of this API (e.g. https://api.example.com)
    pub base_url: String,
    pub group_admin: String,
    pub group_user: String,
    // Caching controls
    pub token_cache_ttl: Duration,
    pub jwks_cache_ttl: Duration,
}

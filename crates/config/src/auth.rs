use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    // OpenID Connect issuer configuration
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: String,
    // Public base URL of this API (e.g. https://api.example.com)
    pub base_url: String,
    #[serde(skip)]
    // Path-only callback for OAuth (e.g. "/auth/callback")
    pub oauth_callback_url: String,
    pub allowed_redirect_urls: Vec<String>, // Where clients can be redirected after auth
    pub jwt_secret: String,
    pub jwt_expiry_hours: i64,
    pub group_admin: String,
    pub group_user: String,
}

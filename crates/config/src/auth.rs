use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: String,
    #[serde(skip)]
    pub oauth_callback_url: String,
    pub allowed_redirect_urls: Vec<String>, // Where clients can be redirected after auth
    pub jwt_secret: String,
    pub jwt_expiry_hours: i64,
    pub group_admin: String,
    pub group_user: String,
}

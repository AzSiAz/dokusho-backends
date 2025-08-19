use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use dokusho_database::models::UserRole;
use openidconnect::AdditionalClaims;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomClaims {
    pub groups: Option<Vec<String>>,
}

impl AdditionalClaims for CustomClaims {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub user_id: Uuid,
    pub email: Option<String>,
    pub name: Option<String>,
    pub role: UserRole,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub oauth_callback_url: String, // The OAuth provider redirects here with the code
    pub allowed_redirect_urls: Vec<String>, // Where clients can be redirected after auth
    pub jwt_secret: String,
    pub jwt_expiry_hours: i64,
    pub group_admin: String,
    pub group_user: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationRequest {
    pub redirect_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationResponse {
    pub authorization_url: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub sub: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
}

impl Claims {
    pub fn new(
        user_id: Uuid,
        sub: String,
        email: Option<String>,
        name: Option<String>,
        role: UserRole,
        expiry_hours: i64,
    ) -> Self {
        let now = Utc::now();
        Self {
            sub,
            user_id,
            email,
            name,
            role,
            iat: now.timestamp(),
            exp: (now + chrono::Duration::hours(expiry_hours)).timestamp(),
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }
}

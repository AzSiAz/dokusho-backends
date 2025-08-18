use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, Default)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    #[default]
    User,
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub sub: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub role: UserRole,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuthState {
    pub state: String,
    pub redirect_uri: String,
    pub nonce: String,
    pub pkce_verifier: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserPreferences {
    pub user_id: Uuid,
    pub preferred_language: Option<String>,
    pub theme: Option<String>,
    pub notifications_enabled: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl User {
    pub fn new(sub: String, email: Option<String>, name: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            sub,
            email,
            name,
            role: UserRole::default(),
            created_at: Some(now),
            updated_at: Some(now),
        }
    }

    pub fn with_role(
        sub: String,
        email: Option<String>,
        name: Option<String>,
        role: UserRole,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            sub,
            email,
            name,
            role,
            created_at: Some(now),
            updated_at: Some(now),
        }
    }
}

impl AuthState {
    pub fn new(
        state: String,
        redirect_uri: String,
        nonce: String,
        pkce_verifier: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            state,
            redirect_uri,
            nonce,
            pkce_verifier,
            created_at: Some(now),
            expires_at: Some(now + chrono::Duration::minutes(10)),
        }
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at.is_some_and(|exp| Utc::now() > exp)
    }
}

impl UserSession {
    pub fn new(user_id: Uuid, token_hash: String, expiry_hours: i64) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            token_hash,
            expires_at: now + chrono::Duration::hours(expiry_hours),
            created_at: Some(now),
            last_used_at: Some(now),
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

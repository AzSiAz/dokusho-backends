use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    User,
    Admin,
}

impl From<dokusho_database::models::user::UserRole> for UserRole {
    fn from(role: dokusho_database::models::user::UserRole) -> Self {
        match role {
            dokusho_database::models::user::UserRole::User => UserRole::User,
            dokusho_database::models::user::UserRole::Admin => UserRole::Admin,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: Uuid,
    pub sub: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub role: UserRole,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<dokusho_database::models::user::User> for UserResponse {
    fn from(user: dokusho_database::models::user::User) -> Self {
        Self {
            id: user.id,
            sub: user.sub,
            email: user.email,
            name: user.name,
            role: user.role.into(),
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

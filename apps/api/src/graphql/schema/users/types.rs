use async_graphql::{Enum, SimpleObject};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// GraphQL enum for user roles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum UserRole {
    User,
    Admin,
}

impl From<dokusho_database::models::UserRole> for UserRole {
    fn from(role: dokusho_database::models::UserRole) -> Self {
        match role {
            dokusho_database::models::UserRole::User => UserRole::User,
            dokusho_database::models::UserRole::Admin => UserRole::Admin,
        }
    }
}

#[derive(Debug, Clone, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct User {
    pub id: Uuid,
    pub sub: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub role: UserRole,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<dokusho_database::models::User> for User {
    fn from(user: dokusho_database::models::User) -> Self {
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

#[derive(SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct InitiateAuthResponse {
    pub authorization_url: String,
    pub state: String,
}

use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};
use dokusho_database::models::user::{User, UserRole};
use std::sync::Arc;

use crate::{AppState, rest::errors::ApiError};

/// Extractor that requires authentication
#[derive(Debug, Clone)]
pub struct RequireAuth {
    pub user: User,
}

impl FromRequestParts<Arc<AppState>> for RequireAuth {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        // Extract token from Authorization header
        let token = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or_else(|| ApiError::unauthorized("Missing or invalid authorization header"))?;

        // Validate access token with OpenID provider
        let user = state
            .auth_service
            .validate_access_token(token)
            .await
            .map_err(ApiError::from)?;

        Ok(RequireAuth { user })
    }
}

/// Extractor that requires admin role
#[derive(Debug, Clone)]
pub struct RequireAdmin;

impl FromRequestParts<Arc<AppState>> for RequireAdmin {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        // First check authentication
        let auth = RequireAuth::from_request_parts(parts, state).await?;

        // Then check if user is admin
        if auth.user.role != UserRole::Admin {
            return Err(ApiError::forbidden(
                "This endpoint requires administrator privileges",
            ));
        }

        Ok(RequireAdmin)
    }
}

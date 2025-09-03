use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};
use dokusho_auth::models::Claims;
use dokusho_database::models::user::{User, UserRole};
use std::sync::Arc;

use crate::{AppState, rest::errors::ApiError};

/// Extractor that requires authentication
#[derive(Debug, Clone)]
pub struct RequireAuth {
    pub claims: Claims,
    pub user: User,
    pub token: String,
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

        // Validate token and session
        let (claims, user) = state
            .auth_service
            .validate_token_and_session(token)
            .await
            .map_err(ApiError::from)?;

        Ok(RequireAuth {
            claims,
            user,
            token: token.to_string(),
        })
    }
}

/// Extractor that requires admin role
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RequireAdmin {
    pub claims: Claims,
    pub user: User,
    pub token: String,
}

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

        Ok(RequireAdmin {
            claims: auth.claims,
            user: auth.user,
            token: auth.token,
        })
    }
}

/// Extractor for optional authentication
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct OptionalAuth {
    pub auth: Option<(Claims, User, String)>,
}

impl FromRequestParts<Arc<AppState>> for OptionalAuth {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        // Extract token from Authorization header if present
        let token = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "));

        if let Some(token) = token {
            // Try to validate token, but don't fail if invalid
            match state.auth_service.validate_token_and_session(token).await {
                Ok((claims, user)) => Ok(OptionalAuth {
                    auth: Some((claims, user, token.to_string())),
                }),
                Err(_) => Ok(OptionalAuth { auth: None }),
            }
        } else {
            Ok(OptionalAuth { auth: None })
        }
    }
}

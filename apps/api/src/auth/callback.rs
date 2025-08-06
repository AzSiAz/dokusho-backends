use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use dokusho_auth::{AuthConfig, AuthService};
use serde::Deserialize;

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct AuthCallbackQuery {
    code: String,
    state: Option<String>,
}

pub async fn auth_callback(
    Query(params): Query<AuthCallbackQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // Create auth config from app config
    let auth_config = AuthConfig {
        enabled: state.config.auth.enabled,
        issuer_url: state.config.auth.issuer_url.clone().unwrap_or_default(),
        client_id: state.config.auth.client_id.clone().unwrap_or_default(),
        client_secret: state.config.auth.client_secret.clone().unwrap_or_default(),
        redirect_url: state.config.auth.redirect_url.clone().unwrap_or_default(),
        jwt_secret: state.config.auth.jwt_secret.clone(),
        jwt_expiry_hours: state.config.auth.jwt_expiry_hours as i64,
    };

    // Create repositories
    use dokusho_database::repositories::{AuthStateRepository, UserRepository};
    let user_repo = UserRepository::new(state.database.pool().clone());
    let auth_state_repo = AuthStateRepository::new(state.database.pool().clone());

    // Create auth service
    let auth_service = match AuthService::new(user_repo, auth_state_repo, auth_config).await {
        Ok(service) => service,
        Err(e) => {
            tracing::error!("Failed to create auth service: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Authentication service unavailable",
            )
                .into_response();
        }
    };

    // Complete authentication
    match auth_service
        .complete_authentication(params.code, params.state.unwrap_or_default())
        .await
    {
        Ok(token_response) => {
            // Redirect to frontend with token
            let redirect_url = format!("/?token={}", token_response.access_token);
            Redirect::to(&redirect_url).into_response()
        }
        Err(e) => {
            tracing::error!("Auth callback error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Authentication failed").into_response()
        }
    }
}

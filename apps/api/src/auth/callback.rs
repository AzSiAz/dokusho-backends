use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use serde::Deserialize;

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct AuthCallbackQuery {
    code: Option<String>,
    state: Option<String>,
    token: Option<String>,
}

pub async fn auth_callback(
    Query(params): Query<AuthCallbackQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // If we already have a token, this is the final redirect - just return success
    if let Some(token) = params.token {
        // You could redirect to a success page or return the token
        return (
            StatusCode::OK,
            format!("Authentication successful. Token: {}", token),
        )
            .into_response();
    }

    // Check if we have the required OAuth parameters
    let code = match params.code {
        Some(code) => code,
        None => {
            return (StatusCode::BAD_REQUEST, "Missing authorization code").into_response();
        }
    };

    // Get auth service from app state
    let auth_service = state.auth_service.clone();

    // Get the stored auth state to retrieve the original redirect URI
    let state_str = params.state.clone().unwrap_or_default();

    // We need to access the auth state repository directly
    let auth_state_repo = state.database.auth_states();
    let stored_state = match auth_state_repo.find_by_state(&state_str).await {
        Ok(Some(state)) => state,
        Ok(None) => {
            return (StatusCode::BAD_REQUEST, "Invalid state parameter").into_response();
        }
        Err(e) => {
            tracing::error!("Failed to retrieve auth state: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Authentication failed").into_response();
        }
    };

    // Complete authentication
    match auth_service
        .complete_authentication(code, params.state.unwrap_or_default())
        .await
    {
        Ok(token_response) => {
            // Redirect to the original redirect URI with token
            let separator = if stored_state.redirect_uri.contains('?') {
                "&"
            } else {
                "?"
            };
            let redirect_url = format!(
                "{}{}token={}",
                stored_state.redirect_uri, separator, token_response.access_token
            );
            Redirect::to(&redirect_url).into_response()
        }
        Err(e) => {
            tracing::error!("Auth callback error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Authentication failed").into_response()
        }
    }
}

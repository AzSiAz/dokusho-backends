use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use dokusho_auth::AuthenticationRequest;
use std::sync::Arc;

use crate::{
    AppState,
    rest::{
        dto::auth::{
            InitiateAuthRequest, InitiateAuthResponse, LogoutResponse, RefreshTokenResponse,
        },
        errors::ApiResult,
        extractors::RequireAuth,
    },
};

#[utoipa::path(
    post,
    path = "/auth/initiate",
    request_body = InitiateAuthRequest,
    responses(
        (status = 200, description = "Authentication initiated successfully", body = InitiateAuthResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "Authentication"
)]
pub async fn initiate_authentication(
    State(state): State<Arc<AppState>>,
    Json(req): Json<InitiateAuthRequest>,
) -> ApiResult<Json<InitiateAuthResponse>> {
    let response = state
        .auth_service
        .initiate_authentication(AuthenticationRequest {
            redirect_uri: req.redirect_uri,
        })
        .await
        .map_err(|e| crate::rest::errors::ApiError::internal_server_error(e.to_string()))?;

    Ok(Json(InitiateAuthResponse {
        authorization_url: response.authorization_url,
        state: response.state,
    }))
}

#[utoipa::path(
    post,
    path = "/auth/refresh",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Token refreshed successfully", body = RefreshTokenResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "Authentication"
)]
#[axum::debug_handler]
pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
) -> ApiResult<Json<RefreshTokenResponse>> {
    let new_token = state
        .auth_service
        .refresh_token(&auth.token)
        .await
        .map_err(crate::rest::errors::ApiError::from)?;

    Ok(Json(RefreshTokenResponse { token: new_token }))
}

#[utoipa::path(
    post,
    path = "/auth/logout",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Logged out successfully", body = LogoutResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "Authentication"
)]
pub async fn logout(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
) -> ApiResult<Json<LogoutResponse>> {
    state
        .auth_service
        .logout(&auth.token)
        .await
        .map_err(crate::rest::errors::ApiError::from)?;

    Ok(Json(LogoutResponse { success: true }))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth/initiate", post(initiate_authentication))
        .route("/auth/refresh", post(refresh_token))
        .route("/auth/logout", post(logout))
        .route(
            "/auth/callback",
            get(super::auth_oauth::callback::auth_callback),
        )
        .route(
            "/auth/success",
            get(super::auth_oauth::success::auth_success),
        )
}

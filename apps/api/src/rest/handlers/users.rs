use axum::{Json, Router, extract::State, routing::get};
use std::sync::Arc;

use crate::{
    AppState,
    rest::{
        dto::users::UserResponse,
        errors::{ApiError, ApiResult},
        extractors::{RequireAdmin, RequireAuth},
    },
};

#[utoipa::path(
    get,
    path = "/users/me",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Current user information", body = UserResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found"),
    ),
    tag = "Users"
)]
pub async fn get_current_user(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
) -> ApiResult<Json<UserResponse>> {
    let user = state
        .database
        .users()
        .find_by_id(auth.claims.user_id)
        .await?
        .ok_or_else(|| ApiError::not_found("User not found"))?;

    Ok(Json(user.into()))
}

#[utoipa::path(
    get,
    path = "/users",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of all users", body = Vec<UserResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required"),
    ),
    tag = "Users"
)]
pub async fn list_users(
    State(state): State<Arc<AppState>>,
    _admin: RequireAdmin,
) -> ApiResult<Json<Vec<UserResponse>>> {
    let users = state.database.users().find_all().await?;

    Ok(Json(users.into_iter().map(Into::into).collect()))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/users/me", get(get_current_user))
        .route("/users", get(list_users))
}

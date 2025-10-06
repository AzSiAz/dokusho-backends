use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::get};
use serde::Serialize;
use std::sync::Arc;
use utoipa::ToSchema;

use crate::AppState;

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub database: String,
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse),
        (status = 503, description = "Service is unhealthy", body = HealthResponse),
    ),
    tag = "Health"
)]
pub async fn health_check(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    if state.database.health_check().await.is_ok() {
        (
            StatusCode::OK,
            Json(HealthResponse {
                status: "healthy".to_string(),
                database: "connected".to_string(),
            }),
        )
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(HealthResponse {
                status: "unhealthy".to_string(),
                database: "disconnected".to_string(),
            }),
        )
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/health", get(health_check))
}

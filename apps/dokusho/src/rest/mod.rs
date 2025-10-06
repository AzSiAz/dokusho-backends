pub mod dto;
pub mod errors;
pub mod extractors;
pub mod handlers;
pub mod openapi;

use axum::Router;
use std::sync::Arc;

use crate::AppState;

pub fn build_rest_router(state: AppState) -> Router {
    use utoipa_swagger_ui::{SwaggerUi, oauth};

    let api_doc = openapi::ApiDoc::openapi_with_config(state.config.auth.issuer_url.clone());

    let api_router = Router::new()
        .merge(handlers::health::routes())
        .merge(handlers::users::routes())
        .merge(handlers::sources::routes())
        .with_state(Arc::new(state.clone()));

    // Configure OAuth for Swagger UI
    let oauth_config = oauth::Config::new()
        .client_id(&state.config.auth.public_client_id)
        .use_pkce_with_authorization_code_grant(true)
        .app_name("Dokusho API")
        .scopes(vec![
            "openid".to_string(),
            "profile".to_string(),
            "email".to_string(),
            "groups".to_string(),
        ]);

    let swagger_ui = SwaggerUi::new("/docs")
        .url("/docs/openapi.json", api_doc)
        .oauth(oauth_config);

    Router::new()
        .nest("/api/v1", api_router)
        .merge(Router::from(swagger_ui))
}

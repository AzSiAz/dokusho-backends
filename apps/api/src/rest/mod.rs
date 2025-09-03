pub mod dto;
pub mod errors;
pub mod extractors;
pub mod handlers;
pub mod openapi;

use axum::Router;
use std::sync::Arc;

use crate::AppState;

pub fn build_rest_router(state: AppState) -> Router {
    use utoipa::OpenApi;
    use utoipa_swagger_ui::SwaggerUi;

    let api_doc = openapi::ApiDoc::openapi();

    let api_router = Router::new()
        .merge(handlers::health::routes())
        .merge(handlers::auth::routes())
        .merge(handlers::users::routes())
        .merge(handlers::sources::routes())
        .merge(handlers::admin::routes())
        .with_state(Arc::new(state));

    let swagger_ui = SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api_doc);

    Router::new()
        .nest("/api/v1", api_router)
        .merge(Router::from(swagger_ui))
}

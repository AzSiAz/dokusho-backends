mod auth;
mod config;
mod graphql;
mod middleware;

use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::State,
    http::Method,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use dokusho_scrapers::SourceRegistry;
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    auth::callback::auth_callback,
    config::AppConfig,
    graphql::{build_schema, AppSchema},
};

#[derive(Clone)]
struct AppState {
    schema: AppSchema,
    config: Arc<AppConfig>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = Arc::new(AppConfig::from_env()?);

    // Initialize tracing
    init_tracing(&config.logging.level);

    tracing::info!("Starting Dokusho API server");

    // Initialize source registry
    let sources = Arc::new(SourceRegistry::new(
        config.sources.use_flaresolver,
        config.sources.flaresolver_url.clone(),
    ));

    // Build GraphQL schema
    let schema = build_schema(sources, (*config).clone());

    // Create app state
    let state = AppState {
        schema,
        config: config.clone(),
    };

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/graphql", post(graphql_handler).get(graphql_playground))
        .route("/auth/callback", get(auth_callback))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST])
                .allow_headers(Any),
        )
        .with_state(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    tracing::info!("GraphQL playground available at http://{}/graphql", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> impl IntoResponse {
    "OK"
}

async fn graphql_handler(
    State(state): State<AppState>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    state.schema.execute(req.into_inner()).await.into()
}

async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}

fn init_tracing(level: &str) {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| format!("dokusho_api={},tower_http=debug", level).into());

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}
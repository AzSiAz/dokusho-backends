mod auth;
mod config;
mod graphql;
mod middleware;

use async_graphql::http::GraphiQLSource;
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
use dokusho_database::Database;

#[derive(Clone)]
struct AppState {
    schema: AppSchema,
    config: Arc<AppConfig>,
    database: Arc<Database>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = Arc::new(AppConfig::from_env()?);

    // Initialize tracing
    init_tracing(&config.logging.level, &config.logging.format);

    tracing::info!("Starting Dokusho API server");

    // Initialize database
    let database = Arc::new(
        Database::new_with_config(
            &config.database.url,
            config.database.max_connections,
            config.database.min_connections,
        )
        .await?,
    );

    // Run migrations
    database.migrate().await?;
    tracing::info!("Database migrations completed");

    // Initialize source registry
    let sources = Arc::new(SourceRegistry::new(
        config.sources.use_flaresolver,
        config.sources.flaresolver_url.clone(),
    ));

    // Build GraphQL schema with database
    let schema = build_schema(sources, (*config).clone(), database.clone());

    // Create app state
    let state = AppState {
        schema,
        config: config.clone(),
        database,
    };

    // Build CORS layer based on config
    let cors_layer = if config.server.cors_origins.is_empty() 
        || (config.server.cors_origins.len() == 1 && config.server.cors_origins[0] == "*") 
    {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::POST])
            .allow_headers(Any)
    } else {
        let origins: Vec<_> = config
            .server
            .cors_origins
            .iter()
            .filter_map(|s| s.parse().ok())
            .collect();
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods([Method::GET, Method::POST])
            .allow_headers(Any)
    };

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/graphql", post(graphql_handler).get(graphiql))
        .route("/auth/callback", get(auth_callback))
        .layer(cors_layer)
        .with_state(state);

    // Start server
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .unwrap_or_else(|_| SocketAddr::from(([0, 0, 0, 0], config.server.port)));
    tracing::info!("GraphiQL available at http://{}/graphql", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> impl IntoResponse {
    "OK"
}

async fn graphql_handler(State(state): State<AppState>, req: GraphQLRequest) -> GraphQLResponse {
    state.schema.execute(req.into_inner()).await.into()
}

async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

fn init_tracing(level: &str, format: &str) {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| format!("dokusho_api={},tower_http=debug", level).into());

    match format {
        "json" => {
            tracing_subscriber::registry()
                .with(filter)
                .with(tracing_subscriber::fmt::layer().json())
                .init();
        }
        "compact" => {
            tracing_subscriber::registry()
                .with(filter)
                .with(tracing_subscriber::fmt::layer().compact())
                .init();
        }
        _ => {
            tracing_subscriber::registry()
                .with(filter)
                .with(tracing_subscriber::fmt::layer())
                .init();
        }
    }
}

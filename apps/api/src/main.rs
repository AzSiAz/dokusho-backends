mod auth;
// config moved to shared crate `dokusho-config`
mod graphql;

use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    Router,
    extract::State,
    http::{Method, header},
    response::{Html, IntoResponse},
    routing::{get, post},
};
use sources::SourceRegistry;
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    auth::callback::auth_callback,
    graphql::{AppSchema, build_schema},
};
use dokusho_auth::AuthService;
use dokusho_config::{AppConfig, LogConfig, log::LogFormat};
use dokusho_database::{
    Database,
    repositories::{AuthStateRepository, UserRepository},
};

#[derive(Clone)]
struct AppState {
    schema: AppSchema,
    _config: AppConfig,
    database: Arc<Database>,
    auth_service: Arc<AuthService>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = AppConfig::from_env()?;

    // Initialize tracing
    init_tracing(&config.log);

    tracing::info!("Starting Dokusho API server");

    // Initialize database
    let database = Arc::new(
        Database::new_with_config(
            &config.database.url,
            config.database.connections_max,
            config.database.connections_min,
        )
        .await?,
    );

    // Run migrations
    database.migrate().await?;
    tracing::info!("Database migrations completed");

    // Clean up expired sessions and auth states on startup
    let user_repo = UserRepository::new(database.pool().clone());
    match user_repo.delete_expired_sessions().await {
        Ok(count) => {
            if count > 0 {
                tracing::info!("Cleaned up {} expired user sessions", count);
            }
        }
        Err(e) => {
            tracing::warn!("Failed to clean up expired sessions: {}", e);
        }
    }

    let auth_state_repo = Arc::new(AuthStateRepository::new(database.pool().clone()));
    match auth_state_repo.delete_expired().await {
        Ok(count) => {
            if count > 0 {
                tracing::info!("Cleaned up {} expired auth states", count);
            }
        }
        Err(e) => {
            tracing::warn!("Failed to clean up expired auth states: {}", e);
        }
    }

    // Initialize source registry
    let sources = Arc::new(SourceRegistry::new(config.sources.clone()));

    let auth_service = Arc::new(
        AuthService::new(user_repo, auth_state_repo, config.auth.clone())
            .await
            .map_err(|e| {
                tracing::error!("Failed to initialize AuthService: {}", e);
                e
            })?,
    );

    tracing::info!("AuthService initialized successfully");

    // Build GraphQL schema with database and auth service
    let schema = build_schema(
        sources,
        config.clone(),
        database.clone(),
        auth_service.clone(),
    );

    // Create app state
    let state = AppState {
        schema,
        _config: config.clone(),
        database,
        auth_service,
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

async fn graphql_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut request = req.into_inner();

    // Extract JWT from Authorization header if present
    if let Some(auth_header) = headers.get(header::AUTHORIZATION)
        && let Ok(auth_str) = auth_header.to_str()
        && let Some(token) = auth_str.strip_prefix("Bearer ")
    {
        // Use the auth service for validation
        // Validate token and session (checks JWT, database session, and expiration)
        if let Ok((claims, _user)) = state.auth_service.validate_token_and_session(token).await {
            request = request.data(claims);
            // Also store the token itself for operations like refresh that need it
            request = request.data(token.to_string());
        }
    }

    state.schema.execute(request).await.into()
}

async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

fn init_tracing(config: &LogConfig) {
    let from_where = ["dokusho_api", "sources"].join(",");
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| format!("{}={}", from_where, config.level).into());

    match config.format {
        LogFormat::Json => {
            tracing_subscriber::registry()
                .with(filter)
                .with(tracing_subscriber::fmt::layer().json())
                .init();
        }
        LogFormat::Compact => {
            tracing_subscriber::registry()
                .with(filter)
                .with(tracing_subscriber::fmt::layer().compact())
                .init();
        }
        LogFormat::Pretty => {
            tracing_subscriber::registry()
                .with(filter)
                .with(tracing_subscriber::fmt::layer().pretty())
                .init();
        }
    }
}

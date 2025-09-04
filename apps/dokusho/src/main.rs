mod rest;

use axum::{Router, http::Method};
use sources::SourceRegistry;
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::rest::build_rest_router;
use dokusho_auth::AuthService;
use dokusho_config::{AppConfig, LogConfig, log::LogFormat};
use dokusho_core::SourceApi;
use dokusho_dashboard::{DashboardConfig, router as dashboard_router};
use dokusho_database::Database;

#[derive(Clone)]
pub struct AppState {
    pub sources: Arc<SourceRegistry>,
    pub config: AppConfig,
    pub database: Arc<Database>,
    pub auth_service: Arc<AuthService>,
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

    // Clean up expired sessions and auth states
    database.cleanup().await?;

    // Initialize source registry
    let sources = Arc::new(SourceRegistry::new(config.sources.clone()));
    // Build list of source infos once and pass as slice reference
    let source_infos: Vec<_> = sources
        .get_sources()
        .iter()
        .map(|s| s.get_information())
        .collect();
    database
        .upsert_static_data(source_infos)
        .await
        .map_err(|e| {
            tracing::error!("Failed to upsert initial data: {}", e);
            e
        })?;

    let auth_service = Arc::new(
        AuthService::new(database.as_ref().clone(), config.auth.clone())
            .await
            .map_err(|e| {
                tracing::error!("Failed to initialize AuthService: {}", e);
                e
            })?,
    );

    tracing::info!("AuthService initialized successfully");

    // Create app state
    let state = AppState {
        sources,
        config: config.clone(),
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

    // Build routers: REST API + Dashboard mounted at '/'
    let rest_app = build_rest_router(state.clone());

    // Dashboard config from API auth config
    let dash_cfg = DashboardConfig {
        issuer_url: state.config.auth.issuer_url.clone(),
        public_client_id: state.config.auth.public_client_id.clone(),
        redirect_url: format!(
            "{}/auth/callback",
            state.config.auth.base_url.trim_end_matches('/')
        ),
    };
    let dashboard = dashboard_router(dash_cfg).await?;

    let app = Router::new()
        .merge(rest_app)
        .merge(dashboard)
        .layer(cors_layer);

    // Start server
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .unwrap_or_else(|_| SocketAddr::from(([0, 0, 0, 0], config.server.port)));
    tracing::info!("API available at http://{}/api/v1", addr);
    tracing::info!("Swagger UI available at http://{}/docs", addr);
    tracing::info!(
        "OpenAPI spec available at http://{}/docs/openapi.json",
        addr
    );

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn init_tracing(config: &LogConfig) {
    let from_where = ["dokusho", "sources", "dokusho_database", "dokusho_auth"].join(",");
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

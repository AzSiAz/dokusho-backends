mod auth;
mod config;
mod graphql;
mod middleware;

use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::State,
    http::{header, Method},
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use sources::SourceRegistry;
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
    _config: Arc<AppConfig>,
    database: Arc<Database>,
    auth_service: Option<Arc<dokusho_auth::AuthService>>,
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

    // Create AuthService if auth is enabled
    let auth_service = if config.auth.enabled {
        use dokusho_auth::{AuthConfig, AuthService};
        use dokusho_database::repositories::{AuthStateRepository, UserRepository};

        let auth_config = AuthConfig {
            enabled: config.auth.enabled,
            issuer_url: config.auth.issuer_url.clone().unwrap_or_default(),
            client_id: config.auth.client_id.clone().unwrap_or_default(),
            client_secret: config.auth.client_secret.clone().unwrap_or_default(),
            oauth_callback_url: config.auth.oauth_callback_url.clone().unwrap_or_default(),
            allowed_redirect_urls: config.auth.allowed_redirect_urls.clone(),
            jwt_secret: config.auth.jwt_secret.clone(),
            jwt_expiry_hours: config.auth.jwt_expiry_hours as i64,
            group_admin: config.auth.group_admin.clone(),
            group_user: config.auth.group_user.clone(),
        };

        let user_repo = UserRepository::new(database.pool().clone());
        let auth_state_repo = Arc::new(AuthStateRepository::new(database.pool().clone()));

        match AuthService::new(user_repo, auth_state_repo, auth_config).await {
            Ok(service) => {
                tracing::info!("AuthService initialized successfully");
                Some(Arc::new(service))
            }
            Err(e) => {
                tracing::error!("Failed to initialize AuthService: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Build GraphQL schema with database and auth service
    let schema = build_schema(
        sources,
        (*config).clone(),
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
    if let Some(auth_header) = headers.get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                // Use the shared auth service for validation
                if let Some(auth_service) = &state.auth_service {
                    // Validate token and session (checks JWT, database session, and expiration)
                    if let Ok((claims, _user)) =
                        auth_service.validate_token_and_session(token).await
                    {
                        request = request.data(claims);
                        // Also store the token itself for operations like refresh that need it
                        request = request.data(token.to_string());
                    }
                }
            }
        }
    }

    state.schema.execute(request).await.into()
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

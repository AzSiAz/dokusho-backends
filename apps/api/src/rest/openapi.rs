use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

use crate::rest::dto::{
    admin::{
        AdminSerieResponse, AdminSeriesPageResponse, CreateSerieFromSourceRequest,
        CreateSerieResponse, ExistingSerieResponse, ExistingSeriesRequest,
    },
    auth::{InitiateAuthRequest, InitiateAuthResponse, LogoutResponse, RefreshTokenResponse},
    sources::{
        ChapterDataResponse, ChapterImageResponse, ChapterTextResponse, ChaptersResponse,
        PaginatedSmallSerieResponse, SearchSerieGenresFilter, SearchSerieRequest,
        SerieChapterResponse, SerieResponse, SmallSerieResponse, SourceResponse,
    },
    users::{UserResponse, UserRole},
};

use crate::rest::handlers::{
    admin, auth,
    health::{self, HealthResponse},
    sources, users,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        // Health
        health::health_check,

        // Authentication
        auth::initiate_authentication,
        auth::refresh_token,
        auth::logout,

        // Users
        users::get_current_user,
        users::list_users,

        // Sources
        sources::list_sources,
        sources::get_source,
        sources::get_popular_series,
        sources::get_latest_series,
        sources::search_series,
        sources::get_serie,
        sources::get_serie_chapters,
        sources::get_chapter_data,

        // Admin
        admin::get_existing_series,
        admin::list_series,
        admin::create_serie_from_source,
    ),
    components(
        schemas(
            // Health
            HealthResponse,

            // Auth
            InitiateAuthRequest,
            InitiateAuthResponse,
            RefreshTokenResponse,
            LogoutResponse,

            // Users
            UserResponse,
            UserRole,

            // Sources
            SourceResponse,
            SmallSerieResponse,
            PaginatedSmallSerieResponse,
            SerieResponse,
            SearchSerieRequest,
            SearchSerieGenresFilter,
            SerieChapterResponse,
            ChaptersResponse,
            ChapterDataResponse,
            ChapterImageResponse,
            ChapterTextResponse,

            // Admin
            ExistingSerieResponse,
            ExistingSeriesRequest,
            AdminSerieResponse,
            AdminSeriesPageResponse,
            CreateSerieFromSourceRequest,
            CreateSerieResponse,

        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Health", description = "Health check endpoints"),
        (name = "Authentication", description = "Authentication and session management"),
        (name = "Users", description = "User management endpoints"),
        (name = "Sources", description = "Manga/Novel source operations"),
        (name = "Admin", description = "Administrative operations"),
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

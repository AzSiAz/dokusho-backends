use utoipa::{
    Modify, OpenApi,
    openapi::security::{OpenIdConnect, SecurityScheme},
};

use crate::rest::dto::{
    sources::{
        ChapterDataResponse, ChapterImageResponse, ChapterTextResponse, ChaptersResponse,
        PaginatedSmallSerieResponse, SearchSerieRequest, SerieChapterResponse, SerieResponse,
        SmallSerieResponse, SourceResponse,
    },
    users::{UserResponse, UserRole},
};

use crate::rest::handlers::{
    health::{self, HealthResponse},
    sources, users,
};

#[derive(OpenApi)]
#[openapi(
    servers(
        (url = "/api/v1", description = "API v1")
    ),
    paths(
        // Health
        health::health_check,

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
    ),
    components(
        schemas(
            // Health
            HealthResponse,

            // Users
            UserResponse,
            UserRole,

            // Sources
            SourceResponse,
            SmallSerieResponse,
            PaginatedSmallSerieResponse,
            SerieResponse,
            SearchSerieRequest,
            SerieChapterResponse,
            ChaptersResponse,
            ChapterDataResponse,
            ChapterImageResponse,
            ChapterTextResponse
        )
    ),
    tags(
        (name = "Health", description = "Health check endpoints"),
        (name = "Users", description = "User management endpoints"),
        (name = "Sources", description = "Manga/Novel source operations"),
    )
)]
pub struct ApiDoc;

impl ApiDoc {
    pub fn openapi_with_config(issuer_url: String) -> utoipa::openapi::OpenApi {
        let mut doc = Self::openapi();
        let addon = SecurityAddon { issuer_url };
        addon.modify(&mut doc);
        doc
    }
}

pub struct SecurityAddon {
    pub issuer_url: String,
}

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            // OpenID Connect discovery document for Swagger OAuth flows
            let openid_url = format!("{}/.well-known/openid-configuration", self.issuer_url);
            components.add_security_scheme(
                "openid_auth",
                SecurityScheme::OpenIdConnect(OpenIdConnect::new(openid_url)),
            );
        }
    }
}

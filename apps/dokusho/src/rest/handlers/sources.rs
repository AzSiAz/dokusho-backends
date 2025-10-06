use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use dokusho_core::{SourceApi, SourceSerieChapterId, SourceSerieId};
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    AppState,
    rest::{
        dto::sources::{
            ChapterDataResponse, ChaptersResponse, PaginatedSmallSerieResponse, SearchSerieRequest,
            SerieResponse, SourceResponse,
        },
        errors::{ApiError, ApiResult},
        extractors::RequireAuth,
    },
};

#[derive(Debug, Deserialize)]
pub struct PageQuery {
    #[serde(default = "default_page")]
    pub page: i16,
}

fn default_page() -> i16 {
    1
}

#[utoipa::path(
    get,
    path = "/sources",
    security(("openid_auth" = [])),
    responses(
        (status = 200, description = "List of available sources", body = Vec<SourceResponse>),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "Sources"
)]
pub async fn list_sources(
    State(state): State<Arc<AppState>>,
    _auth: RequireAuth,
) -> ApiResult<Json<Vec<SourceResponse>>> {
    let sources: Vec<SourceResponse> = state
        .sources
        .get_sources()
        .into_iter()
        .map(|source| source.get_information().into())
        .collect();

    Ok(Json(sources))
}

#[utoipa::path(
    get,
    path = "/sources/{source_id}",
    params(
        ("source_id" = String, Path, description = "Source ID")
    ),
    security(("openid_auth" = [])),
    responses(
        (status = 200, description = "Source information", body = SourceResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Source not found"),
    ),
    tag = "Sources"
)]
pub async fn get_source(
    State(state): State<Arc<AppState>>,
    Path(source_id): Path<String>,
    _auth: RequireAuth,
) -> ApiResult<Json<SourceResponse>> {
    let source = state
        .sources
        .get_source(&source_id)
        .ok_or_else(|| ApiError::not_found(format!("Source '{}' not found", source_id)))?;

    Ok(Json(source.get_information().into()))
}

#[utoipa::path(
    get,
    path = "/sources/{source_id}/series/popular",
    params(
        ("source_id" = String, Path, description = "Source ID"),
        ("page" = i16, Query, description = "Page number", minimum = 1)
    ),
    security(("openid_auth" = [])),
    responses(
        (status = 200, description = "Popular series", body = PaginatedSmallSerieResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Source not found"),
    ),
    tag = "Sources"
)]
pub async fn get_popular_series(
    State(state): State<Arc<AppState>>,
    Path(source_id): Path<String>,
    Query(params): Query<PageQuery>,
    _auth: RequireAuth,
) -> ApiResult<Json<PaginatedSmallSerieResponse>> {
    let source = state
        .sources
        .get_source(&source_id)
        .ok_or_else(|| ApiError::not_found(format!("Source '{}' not found", source_id)))?;

    let result = source
        .fetch_popular_serie(params.page)
        .await
        .map_err(|e| ApiError::internal_server_error(e.to_string()))?;

    Ok(Json(result.into()))
}

#[utoipa::path(
    get,
    path = "/sources/{source_id}/series/latest",
    params(
        ("source_id" = String, Path, description = "Source ID"),
        ("page" = i16, Query, description = "Page number", minimum = 1)
    ),
    security(("openid_auth" = [])),
    responses(
        (status = 200, description = "Latest series updates", body = PaginatedSmallSerieResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Source not found"),
    ),
    tag = "Sources"
)]
pub async fn get_latest_series(
    State(state): State<Arc<AppState>>,
    Path(source_id): Path<String>,
    Query(params): Query<PageQuery>,
    _auth: RequireAuth,
) -> ApiResult<Json<PaginatedSmallSerieResponse>> {
    let source = state
        .sources
        .get_source(&source_id)
        .ok_or_else(|| ApiError::not_found(format!("Source '{}' not found", source_id)))?;

    let result = source
        .fetch_latest_updates(params.page)
        .await
        .map_err(|e| ApiError::internal_server_error(e.to_string()))?;

    Ok(Json(result.into()))
}

#[utoipa::path(
    get,
    path = "/sources/{source_id}/series/search",
    params(
        ("source_id" = String, Path, description = "Source ID"),
        SearchSerieRequest
    ),
    security(("openid_auth" = [])),
    responses(
        (status = 200, description = "Search results", body = PaginatedSmallSerieResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Source not found"),
    ),
    tag = "Sources"
)]
pub async fn search_series(
    State(state): State<Arc<AppState>>,
    Path(source_id): Path<String>,
    _auth: RequireAuth,
    Query(req): Query<SearchSerieRequest>,
) -> ApiResult<Json<PaginatedSmallSerieResponse>> {
    let source = state
        .sources
        .get_source(&source_id)
        .ok_or_else(|| ApiError::not_found(format!("Source '{}' not found", source_id)))?;

    let page = req.page;
    let filters = req.into();

    let result = source
        .fetch_search_serie(page, filters)
        .await
        .map_err(|e| ApiError::internal_server_error(e.to_string()))?;

    Ok(Json(result.into()))
}

#[utoipa::path(
    get,
    path = "/sources/{source_id}/series/{serie_id}",
    params(
        ("source_id" = String, Path, description = "Source ID"),
        ("serie_id" = String, Path, description = "Serie ID")
    ),
    security(("openid_auth" = [])),
    responses(
        (status = 200, description = "Serie details", body = SerieResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Source or serie not found"),
    ),
    tag = "Sources"
)]
pub async fn get_serie(
    State(state): State<Arc<AppState>>,
    Path((source_id, serie_id)): Path<(String, String)>,
    _auth: RequireAuth,
) -> ApiResult<Json<SerieResponse>> {
    let source = state
        .sources
        .get_source(&source_id)
        .ok_or_else(|| ApiError::not_found(format!("Source '{}' not found", source_id)))?;

    let serie = source
        .fetch_serie_detail(SourceSerieId::from(serie_id))
        .await
        .map_err(|e| ApiError::internal_server_error(e.to_string()))?;

    Ok(Json(serie.into()))
}

#[utoipa::path(
    get,
    path = "/sources/{source_id}/series/{serie_id}/chapters",
    params(
        ("source_id" = String, Path, description = "Source ID"),
        ("serie_id" = String, Path, description = "Serie ID")
    ),
    security(("openid_auth" = [])),
    responses(
        (status = 200, description = "Serie chapters", body = ChaptersResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Source or serie not found"),
    ),
    tag = "Sources"
)]
pub async fn get_serie_chapters(
    State(state): State<Arc<AppState>>,
    Path((source_id, serie_id)): Path<(String, String)>,
    _auth: RequireAuth,
) -> ApiResult<Json<ChaptersResponse>> {
    let source = state
        .sources
        .get_source(&source_id)
        .ok_or_else(|| ApiError::not_found(format!("Source '{}' not found", source_id)))?;

    let chapters = source
        .fetch_serie_chapters(SourceSerieId::from(serie_id))
        .await
        .map_err(|e| ApiError::internal_server_error(e.to_string()))?;

    Ok(Json(chapters.into()))
}

#[utoipa::path(
    get,
    path = "/sources/{source_id}/series/{serie_id}/chapters/{chapter_id}",
    params(
        ("source_id" = String, Path, description = "Source ID"),
        ("serie_id" = String, Path, description = "Serie ID"),
        ("chapter_id" = String, Path, description = "Chapter ID")
    ),
    security(("openid_auth" = [])),
    responses(
        (status = 200, description = "Chapter data", body = ChapterDataResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Source, serie, or chapter not found"),
    ),
    tag = "Sources"
)]
pub async fn get_chapter_data(
    State(state): State<Arc<AppState>>,
    Path((source_id, serie_id, chapter_id)): Path<(String, String, String)>,
    _auth: RequireAuth,
) -> ApiResult<Json<ChapterDataResponse>> {
    let source = state
        .sources
        .get_source(&source_id)
        .ok_or_else(|| ApiError::not_found(format!("Source '{}' not found", source_id)))?;

    let data = source
        .fetch_chapter_data(
            SourceSerieId::from(serie_id),
            SourceSerieChapterId::from(chapter_id),
        )
        .await
        .map_err(|e| ApiError::internal_server_error(e.to_string()))?;

    Ok(Json(data.into()))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/sources", get(list_sources))
        .route("/sources/{source_id}", get(get_source))
        .route(
            "/sources/{source_id}/series/popular",
            get(get_popular_series),
        )
        .route("/sources/{source_id}/series/latest", get(get_latest_series))
        .route("/sources/{source_id}/series/search", get(search_series))
        .route("/sources/{source_id}/series/{serie_id}", get(get_serie))
        .route(
            "/sources/{source_id}/series/{serie_id}/chapters",
            get(get_serie_chapters),
        )
        .route(
            "/sources/{source_id}/series/{serie_id}/chapters/{chapter_id}",
            get(get_chapter_data),
        )
}

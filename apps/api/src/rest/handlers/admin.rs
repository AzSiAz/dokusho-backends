use axum::{
    Json, Router,
    extract::{Query, State},
    routing::{get, post},
};
use dokusho_core::{MultiLanguageString, SourceApi, SourceLanguage, SourceSerieId};
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    AppState,
    rest::{
        dto::admin::{
            AdminSerieResponse, AdminSeriesPageResponse, CreateSerieFromSourceRequest,
            CreateSerieResponse, ExistingSerieResponse, ExistingSeriesRequest,
        },
        errors::{ApiError, ApiResult},
        extractors::RequireAdmin,
    },
};

#[derive(Debug, Deserialize)]
pub struct SeriesListQuery {
    #[serde(default = "default_page")]
    pub page: i32,
    #[serde(default = "default_per_page")]
    pub per_page: i32,
}

fn default_page() -> i32 {
    1
}

fn default_per_page() -> i32 {
    24
}

#[utoipa::path(
    post,
    path = "/admin/series/existing",
    request_body = ExistingSeriesRequest,
    security(("openid_auth" = [])),
    responses(
        (status = 200, description = "Existing series for the given source and external IDs", body = Vec<ExistingSerieResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required"),
    ),
    tag = "Admin"
)]
pub async fn get_existing_series(
    State(state): State<Arc<AppState>>,
    _admin: RequireAdmin,
    Json(req): Json<ExistingSeriesRequest>,
) -> ApiResult<Json<Vec<ExistingSerieResponse>>> {
    let pairs = state
        .database
        .series()
        .find_existing_by_source_and_external_ids(&req.source_id, &req.external_ids)
        .await?;

    let response: Vec<ExistingSerieResponse> = pairs
        .into_iter()
        .map(|(external_id, serie_id)| ExistingSerieResponse {
            external_id,
            serie_id,
        })
        .collect();

    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/admin/series",
    params(
        ("page" = i32, Query, description = "Page number", minimum = 1),
        ("per_page" = i32, Query, description = "Items per page", minimum = 1, maximum = 100)
    ),
    security(("openid_auth" = [])),
    responses(
        (status = 200, description = "Paginated list of series", body = AdminSeriesPageResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required"),
    ),
    tag = "Admin"
)]
pub async fn list_series(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SeriesListQuery>,
    _admin: RequireAdmin,
) -> ApiResult<Json<AdminSeriesPageResponse>> {
    // Validate pagination parameters
    if params.page < 1 {
        return Err(ApiError::bad_request("Page must be at least 1"));
    }
    if params.per_page < 1 || params.per_page > 100 {
        return Err(ApiError::bad_request("Per page must be between 1 and 100"));
    }

    let offset = ((params.page - 1) * params.per_page) as u64;
    let limit_plus_one = (params.per_page + 1) as u64;

    let mut items = state
        .database
        .series()
        .list_with_titles(limit_plus_one, offset)
        .await?;

    let has_next_page = (items.len() as i32) > params.per_page;
    if has_next_page {
        items.truncate(params.per_page as usize);
    }

    let series: Vec<AdminSerieResponse> = items
        .into_iter()
        .map(|st| {
            // Build MultiLanguageString from SerieTitle vec
            let mut ml = MultiLanguageString::new();
            for t in st.titles {
                // Only include primary titles (not alternates)
                if !t.is_alternate {
                    // Parse language string to SourceLanguage enum
                    let lang = match t.language.as_str() {
                        "En" | "EN" => Some(SourceLanguage::En),
                        "Fr" | "FR" => Some(SourceLanguage::Fr),
                        "Jp" | "JP" => Some(SourceLanguage::Jp),
                        "JpRo" | "JP-RO" => Some(SourceLanguage::JpRo),
                        "Ko" | "KO" => Some(SourceLanguage::Ko),
                        "ZhHk" | "ZH-HK" => Some(SourceLanguage::ZhHk),
                        "Zh" | "ZH" => Some(SourceLanguage::Zh),
                        _ => None,
                    };

                    if let Some(lang) = lang {
                        ml = ml.insert(lang, t.title);
                    }
                }
            }
            AdminSerieResponse {
                id: *st.serie.id,
                cover: st.serie.cover_url,
                title: ml.into_hashmap(),
            }
        })
        .collect();

    Ok(Json(AdminSeriesPageResponse {
        has_next_page,
        series,
        page: params.page,
        per_page: params.per_page,
    }))
}

#[utoipa::path(
    post,
    path = "/admin/series/from-source",
    request_body = CreateSerieFromSourceRequest,
    security(("openid_auth" = [])),
    responses(
        (status = 200, description = "Serie created or updated successfully", body = CreateSerieResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required"),
        (status = 404, description = "Source not found"),
    ),
    tag = "Admin"
)]
pub async fn create_serie_from_source(
    State(state): State<Arc<AppState>>,
    _admin: RequireAdmin,
    Json(req): Json<CreateSerieFromSourceRequest>,
) -> ApiResult<Json<CreateSerieResponse>> {
    let source = state
        .sources
        .get_source(&req.source_id)
        .ok_or_else(|| ApiError::not_found(format!("Source '{}' not found", req.source_id)))?;

    let serie = source
        .fetch_serie_detail(SourceSerieId::from(req.serie_id))
        .await
        .map_err(|e| ApiError::internal_server_error(e.to_string()))?;

    // Build a core::Source value from the dynamic source
    let core_source = dokusho_core::sources::Source {
        source_information: source.get_information(),
        source_api_information: source.get_api_information(),
    };

    let serie_id = state.database.series().upsert(&serie, &core_source).await?;

    Ok(Json(CreateSerieResponse {
        success: true,
        serie_id: Some(serie_id.id.0),
    }))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/admin/series", get(list_series))
        .route("/admin/series/existing", post(get_existing_series))
        .route("/admin/series/from-source", post(create_serie_from_source))
}

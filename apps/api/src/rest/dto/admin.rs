use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ExistingSerieResponse {
    pub external_id: String,
    pub serie_id: Uuid,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ExistingSeriesRequest {
    pub source_id: String,
    pub external_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AdminSerieResponse {
    pub id: Uuid,
    pub cover: String,
    pub title: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AdminSeriesPageResponse {
    pub has_next_page: bool,
    pub series: Vec<AdminSerieResponse>,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateSerieFromSourceRequest {
    pub source_id: String,
    pub serie_id: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CreateSerieResponse {
    pub success: bool,
    pub serie_id: Option<Uuid>,
}

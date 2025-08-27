use async_graphql::{Context, Object, Result, SimpleObject};
use dokusho_core::{MultiLanguageString, SourceLanguage};
use dokusho_core::{SourceApi, SourceId, SourceSerieId};
use dokusho_database::entities::{prelude::*, serie_titles};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::graphql::{AdminGuard, GraphQLContext};

#[derive(Default)]
pub struct AdminMutation;

#[derive(Default)]
pub struct AdminQuery;

#[derive(Debug, Clone, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct ExistingSerie {
    pub external_id: String,
    pub serie_id: uuid::Uuid,
}

#[Object(rename_fields = "snake_case")]
impl AdminQuery {
    /// Returns the list of existing series for the given source and external ids.
    /// Admin-only
    #[graphql(guard = "AdminGuard")]
    async fn existing_series_for_source(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "source_id")] source_id: String,
        #[graphql(name = "external_ids")] external_ids: Vec<String>,
    ) -> Result<Vec<ExistingSerie>> {
        let context = ctx.data::<GraphQLContext>()?;
        let repo = context.database.series();
        let pairs = repo
            .find_existing_by_source_and_external_ids(&source_id, &external_ids)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(pairs
            .into_iter()
            .map(|(external_id, serie_id)| ExistingSerie { external_id, serie_id })
            .collect())
    }

    /// Paginated list of series in the database (admin only)
    #[graphql(guard = "AdminGuard")]
    async fn series_list(
        &self,
        ctx: &Context<'_>,
        #[graphql(validator(minimum = 1), default = 1)] page: i32,
        #[graphql(name = "per_page", validator(minimum = 1, maximum = 100), default = 24)] per_page: i32,
    ) -> Result<AdminSeriesPage> {
        let context = ctx.data::<GraphQLContext>()?;
        let repo = context.database.series();
        let offset = ((page - 1) * per_page) as u64;
        let limit_plus_one = (per_page + 1) as u64;
        let mut items = repo
            .list_with_titles(limit_plus_one, offset)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        let has_next_page = (items.len() as i32) > per_page;
        if has_next_page {
            items.truncate(per_page as usize);
        }

        let out = items
            .into_iter()
            .map(|(s, titles)| {
                let mut ml = MultiLanguageString::new();
                for t in titles {
                    if let Some(lang) = parse_lang(&t.language) {
                        ml = ml.insert(lang, t.title);
                    }
                }
                AdminSerie { id: s.id, cover: s.cover_url, title: ml }
            })
            .collect();

        Ok(AdminSeriesPage { has_next_page, series: out })
    }
}

#[derive(Debug, Clone, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct AdminSerie {
    pub id: uuid::Uuid,
    pub cover: String,
    pub title: MultiLanguageString,
}

#[derive(Debug, Clone, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct AdminSeriesPage {
    pub has_next_page: bool,
    pub series: Vec<AdminSerie>,
}

fn parse_lang(s: &str) -> Option<SourceLanguage> {
    match s {
        "En" | "EN" => Some(SourceLanguage::En),
        "Fr" | "FR" => Some(SourceLanguage::Fr),
        "Jp" | "JP" => Some(SourceLanguage::Jp),
        "JpRo" | "JP-RO" => Some(SourceLanguage::JpRo),
        "Ko" | "KO" => Some(SourceLanguage::Ko),
        "ZhHk" | "ZH-HK" => Some(SourceLanguage::ZhHk),
        "Zh" | "ZH" => Some(SourceLanguage::Zh),
        _ => None,
    }
}

#[Object(rename_fields = "snake_case")]
impl AdminMutation {
    /// Create or update a serie in the database from a given source and serie id
    /// Admin-only
    #[graphql(guard = "AdminGuard")]
    async fn create_serie_from_source(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "sourceId")] source_id: SourceId,
        #[graphql(name = "serieId")] serie_id: SourceSerieId,
    ) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;

        let source = context.sources.get_source(&source_id).ok_or_else(|| {
            async_graphql::Error::new(format!("Source '{}' not found", source_id))
        })?;

        let serie = source
            .fetch_serie_detail(serie_id)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        // Build a core::Source value from the dynamic source
        let core_source = dokusho_core::sources::Source {
            source_information: source.get_information(),
            source_api_information: source.get_api_information(),
        };

        context
            .database
            .series()
            .upsert(&serie, &core_source)
            .await
            .map(|_| true)
            .map_err(|e| async_graphql::Error::new(e.to_string()))
    }
}

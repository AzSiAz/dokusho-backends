use async_graphql::{Context, Object, Result};
use dokusho_core::{Chapter, GraphQLSource, PaginatedSmallSeries, SearchFilters, Serie};

use super::schema::GraphQLContext;

pub struct Query;

#[Object]
impl Query {
    async fn health(&self) -> &'static str {
        "OK"
    }

    async fn sources(&self, ctx: &Context<'_>) -> Result<Vec<GraphQLSource>> {
        let context = ctx.data::<GraphQLContext>()?;
        Ok(context.sources.list_sources())
    }

    async fn source(&self, ctx: &Context<'_>, name: String) -> Result<Option<GraphQLSource>> {
        let context = ctx.data::<GraphQLContext>()?;
        Ok(context
            .sources
            .list_sources()
            .into_iter()
            .find(|s| s.name == name))
    }

    async fn popular_series(
        &self,
        ctx: &Context<'_>,
        source_name: String,
        page: i32,
    ) -> Result<PaginatedSmallSeries> {
        let context = ctx.data::<GraphQLContext>()?;
        let source = context.sources.get_source(&source_name).ok_or_else(|| {
            async_graphql::Error::new(format!("Source '{}' not found", source_name))
        })?;

        source
            .fetch_popular_series(page)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    async fn latest_series(
        &self,
        ctx: &Context<'_>,
        source_name: String,
        page: i32,
    ) -> Result<PaginatedSmallSeries> {
        let context = ctx.data::<GraphQLContext>()?;
        let source = context.sources.get_source(&source_name).ok_or_else(|| {
            async_graphql::Error::new(format!("Source '{}' not found", source_name))
        })?;

        source
            .fetch_latest_updates(page)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    async fn search_series(
        &self,
        ctx: &Context<'_>,
        source_name: String,
        query: String,
        page: Option<i32>,
    ) -> Result<PaginatedSmallSeries> {
        let context = ctx.data::<GraphQLContext>()?;
        let source = context.sources.get_source(&source_name).ok_or_else(|| {
            async_graphql::Error::new(format!("Source '{}' not found", source_name))
        })?;

        let filters = SearchFilters {
            query,
            ..Default::default()
        };

        source
            .search_series(page.unwrap_or(1), filters)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    async fn serie(
        &self,
        ctx: &Context<'_>,
        source_name: String,
        serie_id: String,
    ) -> Result<Option<Serie>> {
        let context = ctx.data::<GraphQLContext>()?;
        let source = context.sources.get_source(&source_name).ok_or_else(|| {
            async_graphql::Error::new(format!("Source '{}' not found", source_name))
        })?;

        let serie_id = serie_id.into();
        match source.fetch_serie_detail(&serie_id).await {
            Ok(serie) => Ok(Some(serie)),
            Err(e) => {
                // Check if it's a not found error
                if e.to_string().contains("not found") {
                    Ok(None)
                } else {
                    Err(async_graphql::Error::new(e.to_string()))
                }
            }
        }
    }

    async fn serie_chapters(
        &self,
        ctx: &Context<'_>,
        source_name: String,
        serie_id: String,
    ) -> Result<Vec<Chapter>> {
        let context = ctx.data::<GraphQLContext>()?;
        let source = context.sources.get_source(&source_name).ok_or_else(|| {
            async_graphql::Error::new(format!("Source '{}' not found", source_name))
        })?;

        let serie_id = serie_id.into();
        // Get the serie detail which includes chapters in volumes
        let serie = source
            .fetch_serie_detail(&serie_id)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        // Flatten all chapters from all volumes
        let chapters: Vec<Chapter> = serie
            .volumes
            .into_iter()
            .flat_map(|volume| volume.chapters)
            .collect();

        Ok(chapters)
    }

    async fn chapter_pages(
        &self,
        ctx: &Context<'_>,
        source_name: String,
        serie_id: String,
        volume_id: String,
        chapter_id: String,
    ) -> Result<Vec<String>> {
        let context = ctx.data::<GraphQLContext>()?;
        let source = context.sources.get_source(&source_name).ok_or_else(|| {
            async_graphql::Error::new(format!("Source '{}' not found", source_name))
        })?;

        let serie_id = serie_id.into();
        let volume_id = volume_id.into();
        let chapter_id = chapter_id.into();

        let chapter_data = source
            .fetch_chapter_data(&serie_id, &volume_id, &chapter_id)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        // Extract URLs from chapter data
        let pages = match (chapter_data.images, chapter_data.texts) {
            (Some(images), _) => images.into_iter().map(|img| img.url).collect(),
            (_, Some(texts)) => texts.into_iter().map(|txt| txt.text).collect(),
            _ => Vec::new(),
        };

        Ok(pages)
    }
}

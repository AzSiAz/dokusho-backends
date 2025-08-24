use async_graphql::{Context, Object, Result};
use dokusho_core::{SourceApi, SourceId, SourceSerieChapterId, SourceSerieId};

use crate::graphql::{AuthGuard, GraphQLContext};

use super::types::{
    GraphQLChapters, GraphQLFetchSearchSerieFilter, GraphQLPaginatedSmallSerie, GraphQLSerie,
    GraphQLSerieChapterData, GraphQLSource,
};

#[derive(Default)]
pub struct SourcesQuery;

#[Object(rename_fields = "snake_case")]
impl SourcesQuery {
    #[graphql(guard = "AuthGuard")]
    async fn sources(&self, ctx: &Context<'_>) -> Result<Vec<GraphQLSource>> {
        let context = ctx.data::<GraphQLContext>()?;
        Ok(context
            .sources
            .get_sources()
            .into_iter()
            .map(|source| source.get_information().into())
            .collect())
    }

    #[graphql(guard = "AuthGuard")]
    async fn source(&self, ctx: &Context<'_>, id: SourceId) -> Result<Option<GraphQLSource>> {
        let context = ctx.data::<GraphQLContext>()?;
        let source = context.sources.get_source(id.as_str());

        Ok(source.map(|source| source.get_information().into()))
    }

    #[graphql(guard = "AuthGuard")]
    async fn source_popular_series(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "source_id")] source_id: SourceId,
        #[graphql(validator(minimum = 1), default = 1)] page: i16,
    ) -> Result<GraphQLPaginatedSmallSerie> {
        let context = ctx.data::<GraphQLContext>()?;
        let source = context.sources.get_source(&source_id).ok_or_else(|| {
            async_graphql::Error::new(format!("Source '{}' not found", source_id))
        })?;

        source
            .fetch_popular_serie(page)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))
            .map(|data| data.into())
    }

    #[graphql(guard = "AuthGuard")]
    async fn source_latest_series(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "source_id")] source_id: SourceId,
        #[graphql(validator(minimum = 1), default = 1)] page: i16,
    ) -> Result<GraphQLPaginatedSmallSerie> {
        let context = ctx.data::<GraphQLContext>()?;
        let source = context.sources.get_source(&source_id).ok_or_else(|| {
            async_graphql::Error::new(format!("Source '{}' not found", source_id))
        })?;

        source
            .fetch_latest_updates(page)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))
            .map(|data| data.into())
    }

    #[graphql(guard = "AuthGuard")]
    async fn source_search_series(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "source_id")] source_id: SourceId,
        filters: GraphQLFetchSearchSerieFilter,
        #[graphql(validator(minimum = 1), default = 1)] page: i16,
    ) -> Result<GraphQLPaginatedSmallSerie> {
        let context = ctx.data::<GraphQLContext>()?;
        let source = context.sources.get_source(&source_id).ok_or_else(|| {
            async_graphql::Error::new(format!("Source '{}' not found", source_id))
        })?;

        source
            .fetch_search_serie(page, filters.into())
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))
            .map(|data| data.into())
    }

    #[graphql(guard = "AuthGuard")]
    async fn source_serie(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "source_id")] source_id: SourceId,
        #[graphql(name = "serie_id")] serie_id: SourceSerieId,
    ) -> Result<Option<GraphQLSerie>> {
        let context = ctx.data::<GraphQLContext>()?;
        let source = context.sources.get_source(&source_id).ok_or_else(|| {
            async_graphql::Error::new(format!("Source '{}' not found", source_id))
        })?;

        match source.fetch_serie_detail(serie_id).await {
            Ok(serie) => Ok(Some(serie.into())),
            Err(_) => Ok(None),
        }
    }

    #[graphql(guard = "AuthGuard")]
    async fn source_serie_chapters(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "source_id")] source_id: SourceId,
        #[graphql(name = "serie_id")] serie_id: SourceSerieId,
    ) -> Result<GraphQLChapters> {
        let context = ctx.data::<GraphQLContext>()?;
        let source = context.sources.get_source(&source_id).ok_or_else(|| {
            async_graphql::Error::new(format!("Source '{}' not found", source_id))
        })?;

        source
            .fetch_serie_chapters(serie_id)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))
            .map(|data| data.into())
    }

    #[graphql(guard = "AuthGuard")]
    async fn source_serie_chapters_data(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "source_id")] source_id: SourceId,
        #[graphql(name = "serie_id")] serie_id: SourceSerieId,
        #[graphql(name = "chapter_id")] chapter_id: SourceSerieChapterId,
    ) -> Result<GraphQLSerieChapterData> {
        let context = ctx.data::<GraphQLContext>()?;
        let source = context.sources.get_source(&source_id).ok_or_else(|| {
            async_graphql::Error::new(format!("Source '{}' not found", source_id))
        })?;

        source
            .fetch_chapter_data(serie_id, chapter_id)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))
            .map(|data| data.into())
    }
}

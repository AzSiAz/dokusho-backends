use async_graphql::{Context, Object, Result};
use dokusho_core::{SourceApi, SourceId};

use crate::graphql::{AuthGuard, GraphQLContext};

use super::types::{GraphQLFetchSearchSerieFilter, GraphQLPaginatedSmallSerie, GraphQLSource};

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
}

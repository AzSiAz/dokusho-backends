use async_graphql::{Context, Object, Result};
use dokusho_auth::models::Claims;
use dokusho_core::{FetchSearchSerieFilter, SourceApi, SourceId};
use dokusho_database::repositories::UserRepository;

use crate::graphql::{
    guards::{AdminGuard, AuthGuard},
    schema::GraphQLContext,
    types::{GraphQLPaginatedSmallSerie, GraphQLSource, User},
};

pub struct Query;

#[Object(rename_fields = "snake_case")]
impl Query {
    async fn health(&self) -> &'static str {
        "OK"
    }

    /// Get the current authenticated user's information
    #[graphql(guard = "AuthGuard")]
    async fn me(&self, ctx: &Context<'_>) -> Result<User> {
        let context = ctx.data::<GraphQLContext>()?;
        let claims = ctx.data::<Claims>()?;

        let user_repo = UserRepository::new(context.database.pool().clone());
        let user = user_repo
            .find_by_id(claims.user_id)
            .await?
            .ok_or_else(|| async_graphql::Error::new("User not found"))?;

        Ok(user.into())
    }

    /// Get all users (admin only)
    #[graphql(guard = "AdminGuard")]
    async fn users(&self, ctx: &Context<'_>) -> Result<Vec<User>> {
        let context = ctx.data::<GraphQLContext>()?;

        let user_repo = UserRepository::new(context.database.pool().clone());
        let users = user_repo.find_all().await?;

        Ok(users.into_iter().map(Into::into).collect())
    }

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
        query: String,
        #[graphql(validator(minimum = 1), default = 1)] page: i16,
    ) -> Result<GraphQLPaginatedSmallSerie> {
        let context = ctx.data::<GraphQLContext>()?;
        let source = context.sources.get_source(&source_id).ok_or_else(|| {
            async_graphql::Error::new(format!("Source '{}' not found", source_id))
        })?;

        let filters = FetchSearchSerieFilter {
            query: Some(query),
            ..Default::default()
        };

        source
            .fetch_search_serie(page, filters)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))
            .map(|data| data.into())
    }

    //     async fn source_serie(
    //         &self,
    //         ctx: &Context<'_>,
    //         source_name: String,
    //         serie_id: String,
    //     ) -> Result<Option<Serie>> {
    //         let context = ctx.data::<GraphQLContext>()?;
    //         let source = context.sources.get_source(&source_name).ok_or_else(|| {
    //             async_graphql::Error::new(format!("Source '{}' not found", source_name))
    //         })?;

    //         let serie_id = serie_id.into();
    //         match source.fetch_serie_detail(&serie_id).await {
    //             Ok(serie) => Ok(Some(serie)),
    //             Err(e) => {
    //                 // Check if it's a not found error
    //                 if e.to_string().contains("not found") {
    //                     Ok(None)
    //                 } else {
    //                     Err(async_graphql::Error::new(e.to_string()))
    //                 }
    //             }
    //         }
    //     }

    //     async fn source_serie_chapters(
    //         &self,
    //         ctx: &Context<'_>,
    //         source_name: String,
    //         serie_id: String,
    //     ) -> Result<Vec<Chapter>> {
    //         let context = ctx.data::<GraphQLContext>()?;
    //         let source = context.sources.get_source(&source_name).ok_or_else(|| {
    //             async_graphql::Error::new(format!("Source '{}' not found", source_name))
    //         })?;

    //         let serie_id = serie_id.into();
    //         // Get the serie detail which includes chapters in volumes
    //         let serie = source
    //             .fetch_serie_detail(&serie_id)
    //             .await
    //             .map_err(|e| async_graphql::Error::new(e.to_string()))?;

    //         // Flatten all chapters from all volumes
    //         let chapters: Vec<Chapter> = serie
    //             .volumes
    //             .into_iter()
    //             .flat_map(|volume| volume.chapters)
    //             .collect();

    //         Ok(chapters)
    //     }

    //     async fn source_chapter_pages(
    //         &self,
    //         ctx: &Context<'_>,
    //         source_name: String,
    //         serie_id: String,
    //         volume_id: String,
    //         chapter_id: String,
    //     ) -> Result<Vec<String>> {
    //         let context = ctx.data::<GraphQLContext>()?;
    //         let source = context.sources.get_source(&source_name).ok_or_else(|| {
    //             async_graphql::Error::new(format!("Source '{}' not found", source_name))
    //         })?;

    //         let serie_id = serie_id.into();
    //         let volume_id = volume_id.into();
    //         let chapter_id = chapter_id.into();

    //         let chapter_data = source
    //             .fetch_chapter_data(&serie_id, &volume_id, &chapter_id)
    //             .await
    //             .map_err(|e| async_graphql::Error::new(e.to_string()))?;

    //         // Extract URLs from chapter data
    //         let pages = match (chapter_data.images, chapter_data.texts) {
    //             (Some(images), _) => images.into_iter().map(|img| img.url).collect(),
    //             (_, Some(texts)) => texts.into_iter().map(|txt| txt.text).collect(),
    //             _ => Vec::new(),
    //         };

    //         Ok(pages)
    //     }
}

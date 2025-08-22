use async_graphql::{Context, Object, Result, SimpleObject};
use chrono::{DateTime, Utc};
use dokusho_auth::models::Claims;
use dokusho_core::{
    MultiLanguageString, SourceApi, SourceId, SourceInformation, SourceLanguage,
    SourcePaginatedSmallSerie, SourceSerieId, SourceSmallSerie, SupportedFilters,
};
use dokusho_database::repositories::UserRepository;

use super::{
    guards::{AdminGuard, AuthGuard},
    schema::GraphQLContext,
    types::User,
};

#[derive(Debug, Clone, SimpleObject)]
struct GraphQLSource {
    id: SourceId,
    pub name: String,
    pub url: String,
    pub icon: String,
    pub languages: Vec<SourceLanguage>,
    #[graphql(name = "enabled_languages")]
    pub enabled_languages: Vec<SourceLanguage>,
    #[graphql(name = "updated_at")]
    pub updated_at: DateTime<Utc>,
    pub version: String,
    #[graphql(name = "include_nsfw")]
    pub include_nsfw: bool,
    pub filters: SupportedFilters,
}

impl From<SourceInformation> for GraphQLSource {
    fn from(value: SourceInformation) -> Self {
        Self {
            id: value.id,
            name: value.name,
            icon: value.icon.to_string(),
            enabled_languages: value.enabled_languages,
            languages: value.languages,
            updated_at: value.updated_at,
            version: value.version,
            include_nsfw: value.include_nsfw,
            url: value.url.to_string(),
            filters: value.search_filters,
        }
    }
}

#[derive(Debug, Clone, SimpleObject)]
pub struct GraphQLSmallSerie {
    pub id: SourceSerieId,
    pub title: MultiLanguageString,
    pub cover: String,
}

impl From<SourceSmallSerie> for GraphQLSmallSerie {
    fn from(value: SourceSmallSerie) -> Self {
        Self {
            id: value.id,
            title: value.title,
            cover: value.cover.to_string(),
        }
    }
}

#[derive(Debug, Clone, SimpleObject)]
pub struct GraphQLPaginatedSmallSerie {
    pub has_next_page: bool,
    pub series: Vec<GraphQLSmallSerie>,
}

impl From<SourcePaginatedSmallSerie> for GraphQLPaginatedSmallSerie {
    fn from(value: SourcePaginatedSmallSerie) -> Self {
        Self {
            has_next_page: value.has_next_page,
            series: value.series.into_iter().map(Into::into).collect(),
        }
    }
}

pub struct Query;

#[Object]
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

    async fn sources(&self, ctx: &Context<'_>) -> Result<Vec<GraphQLSource>> {
        let context = ctx.data::<GraphQLContext>()?;
        Ok(context
            .sources
            .get_sources()
            .into_iter()
            .map(|source| source.get_information().into())
            .collect())
    }

    async fn source(&self, ctx: &Context<'_>, name: String) -> Result<Option<GraphQLSource>> {
        let context = ctx.data::<GraphQLContext>()?;
        let source = context.sources.get_source(name.as_str());

        Ok(source.map(|source| source.get_information().into()))
    }

    #[graphql(name = "source_popular_series")]
    async fn source_popular_series(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "source_name")] source_name: String,
        page: i16,
    ) -> Result<GraphQLPaginatedSmallSerie> {
        let context = ctx.data::<GraphQLContext>()?;
        let source = context.sources.get_source(&source_name).ok_or_else(|| {
            async_graphql::Error::new(format!("Source '{}' not found", source_name))
        })?;

        source
            .fetch_popular_serie(page)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))
            .map(|data| data.into())
    }

    //     async fn latest_series(
    //         &self,
    //         ctx: &Context<'_>,
    //         source_name: String,
    //         page: i32,
    //     ) -> Result<PaginatedSmallSeries> {
    //         let context = ctx.data::<GraphQLContext>()?;
    //         let source = context.sources.get_source(&source_name).ok_or_else(|| {
    //             async_graphql::Error::new(format!("Source '{}' not found", source_name))
    //         })?;

    //         source
    //             .fetch_latest_updates(page)
    //             .await
    //             .map_err(|e| async_graphql::Error::new(e.to_string()))
    //     }

    //     async fn search_series(
    //         &self,
    //         ctx: &Context<'_>,
    //         source_name: String,
    //         query: String,
    //         page: Option<i32>,
    //     ) -> Result<PaginatedSmallSeries> {
    //         let context = ctx.data::<GraphQLContext>()?;
    //         let source = context.sources.get_source(&source_name).ok_or_else(|| {
    //             async_graphql::Error::new(format!("Source '{}' not found", source_name))
    //         })?;

    //         let filters = SearchFilters {
    //             query,
    //             ..Default::default()
    //         };

    //         source
    //             .search_series(page.unwrap_or(1), filters)
    //             .await
    //             .map_err(|e| async_graphql::Error::new(e.to_string()))
    //     }

    //     async fn serie(
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

    //     async fn serie_chapters(
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

    //     async fn chapter_pages(
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

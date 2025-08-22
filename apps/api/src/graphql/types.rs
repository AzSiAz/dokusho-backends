use async_graphql::{Enum, SimpleObject};
use chrono::{DateTime, Utc};
use dokusho_core::{
    MultiLanguageString, SourceId, SourceInformation, SourceLanguage, SourcePaginatedSmallSerie,
    SourceSerieId, SourceSmallSerie, SupportedFilters,
};
use uuid::Uuid;

/// GraphQL enum for user roles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum UserRole {
    User,
    Admin,
}

impl From<dokusho_database::models::UserRole> for UserRole {
    fn from(role: dokusho_database::models::UserRole) -> Self {
        match role {
            dokusho_database::models::UserRole::User => UserRole::User,
            dokusho_database::models::UserRole::Admin => UserRole::Admin,
        }
    }
}

#[derive(Debug, Clone, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct User {
    pub id: Uuid,
    pub sub: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub role: UserRole,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<dokusho_database::models::User> for User {
    fn from(user: dokusho_database::models::User) -> Self {
        Self {
            id: user.id,
            sub: user.sub,
            email: user.email,
            name: user.name,
            role: user.role.into(),
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

#[derive(Debug, Clone, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct GraphQLSource {
    id: SourceId,
    pub name: String,
    pub url: String,
    pub icon: String,
    pub languages: Vec<SourceLanguage>,
    pub enabled_languages: Vec<SourceLanguage>,
    pub updated_at: DateTime<Utc>,
    pub version: String,
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
#[graphql(rename_fields = "snake_case")]
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
#[graphql(rename_fields = "snake_case")]
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

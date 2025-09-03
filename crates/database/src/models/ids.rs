use serde::{Deserialize, Serialize};
use std::ops::Deref;
use uuid::Uuid;

// Newtype wrapper for Serie ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[cfg_attr(feature = "graphql", derive(async_graphql::NewType))]
pub struct SerieId(pub Uuid);

// Newtype wrapper for Genre ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[cfg_attr(feature = "graphql", derive(async_graphql::NewType))]
pub struct GenreId(pub Uuid);

// Newtype wrapper for Status ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[cfg_attr(feature = "graphql", derive(async_graphql::NewType))]
pub struct StatusId(pub Uuid);

// Newtype wrapper for Author ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[cfg_attr(feature = "graphql", derive(async_graphql::NewType))]
pub struct AuthorId(pub Uuid);

// Newtype wrapper for Artist ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[cfg_attr(feature = "graphql", derive(async_graphql::NewType))]
pub struct ArtistId(pub Uuid);

// Newtype wrapper for SerieType ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[cfg_attr(feature = "graphql", derive(async_graphql::NewType))]
pub struct SerieTypeId(pub Uuid);

// Newtype wrapper for SerieTitle ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[cfg_attr(feature = "graphql", derive(async_graphql::NewType))]
pub struct SerieTitleId(pub Uuid);

// Newtype wrapper for SerieSynopsis ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[cfg_attr(feature = "graphql", derive(async_graphql::NewType))]
pub struct SerieSynopsisId(pub Uuid);

// Newtype wrapper for Source ID (String-based)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[cfg_attr(feature = "graphql", derive(async_graphql::NewType))]
pub struct SourceId(pub String);

// Helper implementations for creating new IDs
impl SerieId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl GenreId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl StatusId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl AuthorId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl ArtistId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl SerieTypeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl SerieTitleId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl SerieSynopsisId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl SourceId {
    pub fn new(id: String) -> Self {
        Self(id)
    }
}

// Implement Default for ID types that need it
impl Default for SerieId {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for GenreId {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for StatusId {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AuthorId {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ArtistId {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SerieTypeId {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SerieTitleId {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SerieSynopsisId {
    fn default() -> Self {
        Self::new()
    }
}

// From implementations: async_graphql::NewType provides these when graphql feature is enabled
// For non-graphql builds, we provide them manually
#[cfg(not(feature = "graphql"))]
impl From<Uuid> for SerieId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

#[cfg(not(feature = "graphql"))]
impl From<Uuid> for GenreId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

#[cfg(not(feature = "graphql"))]
impl From<Uuid> for StatusId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

#[cfg(not(feature = "graphql"))]
impl From<Uuid> for AuthorId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

#[cfg(not(feature = "graphql"))]
impl From<Uuid> for ArtistId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

#[cfg(not(feature = "graphql"))]
impl From<Uuid> for SerieTypeId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

#[cfg(not(feature = "graphql"))]
impl From<Uuid> for SerieTitleId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

#[cfg(not(feature = "graphql"))]
impl From<Uuid> for SerieSynopsisId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

#[cfg(not(feature = "graphql"))]
impl From<String> for SourceId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

// Implement Deref for all UUID-based ID types to make them easier to use with SQLx
impl Deref for SerieId {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for GenreId {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for StatusId {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for AuthorId {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for ArtistId {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for SerieTypeId {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for SerieTitleId {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for SerieSynopsisId {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for SourceId {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

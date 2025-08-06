use async_trait::async_trait;

use crate::errors::SourceError;
use crate::types::{
    ChapterData, ChapterId, PaginatedSmallSeries, SearchFilters, Serie, SerieId,
    SourceApiInformation, SourceInformation, VolumeId,
};

#[async_trait]
pub trait SourceApi: Send + Sync {
    fn get_information(&self) -> SourceInformation;

    fn get_api_information(&self) -> SourceApiInformation;

    async fn fetch_popular_series(&self, page: i32) -> Result<PaginatedSmallSeries, SourceError>;

    async fn fetch_latest_updates(&self, page: i32) -> Result<PaginatedSmallSeries, SourceError>;

    async fn search_series(
        &self,
        page: i32,
        filters: SearchFilters,
    ) -> Result<PaginatedSmallSeries, SourceError>;

    async fn fetch_serie_detail(&self, serie_id: &SerieId) -> Result<Serie, SourceError>;

    async fn fetch_chapter_data(
        &self,
        serie_id: &SerieId,
        volume_id: &VolumeId,
        chapter_id: &ChapterId,
    ) -> Result<ChapterData, SourceError>;

    async fn get_serie_url(&self, serie_id: &SerieId) -> Result<String, SourceError> {
        Ok(format!("{}/serie/{}", self.get_information().url, serie_id))
    }
}

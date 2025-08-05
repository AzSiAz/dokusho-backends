use async_trait::async_trait;

use crate::errors::SourceError;
use crate::types::{
    ChapterData, ChapterId, PaginatedSmallSeries, SearchFilters, Serie, SerieId, SourceInformation,
    VolumeId,
};

#[async_trait]
pub trait SourceApi: Send + Sync {
    fn information(&self) -> &SourceInformation;

    async fn fetch_popular_series(&self, page: u32) -> Result<PaginatedSmallSeries, SourceError>;

    async fn fetch_latest_series(&self, page: u32) -> Result<PaginatedSmallSeries, SourceError>;

    async fn search_series(
        &self,
        filters: &SearchFilters,
        page: u32,
    ) -> Result<PaginatedSmallSeries, SourceError>;

    async fn fetch_serie_detail(&self, serie_id: &SerieId) -> Result<Serie, SourceError>;

    async fn fetch_chapter_data(
        &self,
        serie_id: &SerieId,
        volume_id: &VolumeId,
        chapter_id: &ChapterId,
    ) -> Result<ChapterData, SourceError>;

    async fn get_serie_url(&self, serie_id: &SerieId) -> Result<String, SourceError> {
        Ok(format!("{}/serie/{}", self.information().base_url, serie_id))
    }

    async fn get_chapter_url(
        &self,
        serie_id: &SerieId,
        volume_id: &VolumeId,
        chapter_id: &ChapterId,
    ) -> Result<String, SourceError> {
        Ok(format!(
            "{}/serie/{}/volume/{}/chapter/{}",
            self.information().base_url,
            serie_id,
            volume_id,
            chapter_id
        ))
    }
}
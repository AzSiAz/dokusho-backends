use async_trait::async_trait;
use url::Url;

/// Trait for HTTP clients that can fetch HTML content for scraping
#[async_trait]
pub trait ScraperClient: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Fetch HTML content from a URL
    async fn get_html(&self, url: &Url) -> Result<String, Self::Error>;

    /// Get the underlying reqwest client for advanced operations
    fn get_reqwest_client(&self) -> &reqwest::Client;
}

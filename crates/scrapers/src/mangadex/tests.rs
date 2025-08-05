use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{method, path, path_regex, query_param};

use dokusho_core::{
    ChapterDataType, ChapterId, SearchFilters, SerieId, SourceApi, VolumeId,
};

use super::*;

#[tokio::test]
async fn test_mangadex_serie_detail() {
    let mock_server = MockServer::start().await;
    let fixture = include_str!("fixtures/serie_detail.json");

    // Mock the chapters endpoint FIRST (more specific path)
    let chapters_fixture = include_str!("fixtures/serie_detail_volume.json");
    
    Mock::given(method("GET"))
        .and(path_regex(r"^/manga/.*/feed.*"))
        .respond_with(ResponseTemplate::new(200).set_body_string(chapters_fixture))
        .mount(&mock_server)
        .await;

    // Then mock the manga detail endpoint
    Mock::given(method("GET"))
        .and(path("/manga/32d76d19-8a05-4db0-9fc2-e0b0648fe9d0"))
        .and(query_param("includes[]", "cover_art"))
        .and(query_param("includes[]", "author"))
        .and(query_param("includes[]", "artist"))
        .respond_with(ResponseTemplate::new(200).set_body_string(fixture))
        .mount(&mock_server)
        .await;

    let mangadex = MangaDex::new_with_url(&mock_server.uri());
    let serie_id = SerieId::new("32d76d19-8a05-4db0-9fc2-e0b0648fe9d0");
    let result = mangadex.fetch_serie_detail(&serie_id).await;

    // Check that we can parse the response without errors
    assert!(result.is_ok());
    let serie = result.unwrap();
    
    // Verify some basic fields
    assert_eq!(serie.title.en.as_ref().map(|s| s.as_str()), Some("Solo Leveling"));
    assert_eq!(serie.id.as_str(), "32d76d19-8a05-4db0-9fc2-e0b0648fe9d0");
    assert!(!serie.volumes.is_empty());
}

#[tokio::test]
async fn test_mangadex_search() {
    let mock_server = MockServer::start().await;
    let fixture = include_str!("fixtures/search_serie.json");

    Mock::given(method("GET"))
        .and(path("/manga"))
        .respond_with(ResponseTemplate::new(200).set_body_string(fixture))
        .mount(&mock_server)
        .await;

    let mangadex = MangaDex::new_with_url(&mock_server.uri());
    let filters = SearchFilters {
        query: "Solo Leveling".to_string(),
        ..Default::default()
    };
    
    let result = mangadex.search_series(1, filters).await;

    assert!(result.is_ok());
    let paginated = result.unwrap();
    
    assert!(!paginated.series.is_empty());
    assert_eq!(paginated.series[0].title.en.as_ref().map(|s| s.as_str()), Some("Kimi wa 008"));
}

#[tokio::test]
async fn test_mangadex_chapter_images() {
    let mock_server = MockServer::start().await;
    let fixture = include_str!("fixtures/chapter_images.json");

    Mock::given(method("GET"))
        .and(path("/at-home/server/test-chapter-id"))
        .respond_with(ResponseTemplate::new(200).set_body_string(fixture))
        .mount(&mock_server)
        .await;

    let mangadex = MangaDex::new_with_url(&mock_server.uri());
    let result = mangadex
        .fetch_chapter_data(
            &SerieId::new("test"),
            &VolumeId::new("test"),
            &ChapterId::new("test-chapter-id"),
        )
        .await;

    assert!(result.is_ok());
    let chapter_data = result.unwrap();
    assert_eq!(chapter_data.data_type, ChapterDataType::Image);
    let images = chapter_data.images.expect("Expected images");
    assert!(!images.is_empty());
    assert!(images[0].url.contains("data"));
}
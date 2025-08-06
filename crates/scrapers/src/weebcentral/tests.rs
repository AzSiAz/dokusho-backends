use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

use dokusho_core::{
    ChapterDataType, ChapterId, SearchFilters, SerieId, SerieType, SourceApi, VolumeId,
};

use super::*;

#[tokio::test]
async fn test_weebcentral_serie_detail() {
    let mock_server = MockServer::start().await;
    let serie_fixture = include_str!("fixtures/serie.html");
    let chapters_fixture = include_str!("fixtures/chapters_list.html");

    // Mock the serie page
    Mock::given(method("GET"))
        .and(path("/series/01J76XYGC5B3EH5D5XDR7M490Q"))
        .respond_with(ResponseTemplate::new(200).set_body_string(serie_fixture))
        .mount(&mock_server)
        .await;

    // Mock the chapters page
    Mock::given(method("GET"))
        .and(path("/series/01J76XYGC5B3EH5D5XDR7M490Q/full-chapter-list"))
        .respond_with(ResponseTemplate::new(200).set_body_string(chapters_fixture))
        .mount(&mock_server)
        .await;

    let weebcentral = WeebCentral::new_for_testing(mock_server.uri()).unwrap();
    let serie_id = SerieId::new("01J76XYGC5B3EH5D5XDR7M490Q");
    let result = weebcentral.fetch_serie_detail(&serie_id).await;

    assert!(result.is_ok());
    let serie = result.unwrap();

    assert_eq!(serie.id.as_str(), "01J76XYGC5B3EH5D5XDR7M490Q");
    assert_eq!(
        serie.title.en.as_ref().map(|s| s.as_str()),
        Some("Sono Munou, Jitsu wa Sekai Saikyou no Mahoutsukai")
    );
    assert_eq!(serie.serie_type, SerieType::Manga);
}

#[tokio::test]
async fn test_weebcentral_search() {
    let mock_server = MockServer::start().await;
    let fixture = include_str!("fixtures/search.html");

    // Mock the search API
    Mock::given(method("GET"))
        .and(path_regex(r"^/search/data.*"))
        .respond_with(ResponseTemplate::new(200).set_body_string(fixture))
        .mount(&mock_server)
        .await;

    let weebcentral = WeebCentral::new_for_testing(mock_server.uri()).unwrap();
    let filters = SearchFilters {
        query: "test".to_string(),
        ..Default::default()
    };

    let result = weebcentral.search_series(1, filters).await;

    assert!(result.is_ok());
    let paginated = result.unwrap();

    assert!(!paginated.series.is_empty());
}

#[tokio::test]
async fn test_weebcentral_chapter_images() {
    let mock_server = MockServer::start().await;
    let fixture = include_str!("fixtures/chapter_images.html");

    // Mock the chapter images API
    Mock::given(method("GET"))
        .and(path_regex(r"^/chapters/.*/images.*"))
        .respond_with(ResponseTemplate::new(200).set_body_string(fixture))
        .mount(&mock_server)
        .await;

    let weebcentral = WeebCentral::new_for_testing(mock_server.uri()).unwrap();
    let result = weebcentral
        .fetch_chapter_data(
            &SerieId::new("test"),
            &VolumeId::new("all"),
            &ChapterId::new("test"),
        )
        .await;

    assert!(result.is_ok());
    let chapter_data = result.unwrap();
    assert_eq!(chapter_data.data_type, ChapterDataType::Image);
    let images = chapter_data.images.expect("Expected images");
    assert!(!images.is_empty());
    assert!(images[0].url.contains("http"));
}
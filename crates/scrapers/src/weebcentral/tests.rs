use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{method, path};

use dokusho_core::{
    ChapterData, ChapterId, Language, SearchFilters, SerieId, SerieType, SourceApi, VolumeId,
};

use super::*;

#[tokio::test]
async fn test_weebcentral_serie_detail() {
    let mock_server = MockServer::start().await;
    let fixture = include_str!("fixtures/serie.html");

    Mock::given(method("POST"))
        .and(path("/v1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "ok",
            "message": "",
            "startTimestamp": 1000,
            "endTimestamp": 2000,
            "version": "3.3.0",
            "solution": {
                "url": "https://weebcentral.com/series/01J76XYGC5B3EH5D5XDR7M490Q",
                "status": 200,
                "headers": {},
                "response": fixture,
                "cookies": [],
                "userAgent": "Mozilla/5.0"
            }
        })))
        .mount(&mock_server)
        .await;

    let weebcentral = WeebCentral::new(&mock_server.uri()).unwrap();
    let serie_id = SerieId::new("01J76XYGC5B3EH5D5XDR7M490Q");
    let result = weebcentral.fetch_serie_detail(&serie_id).await;

    assert!(result.is_ok());
    let serie = result.unwrap();
    
    assert_eq!(serie.id.as_str(), "01J76XYGC5B3EH5D5XDR7M490Q");
    assert_eq!(
        serie.title.get(Language::English),
        Some("Sono Munou, Jitsu wa Sekai Saikyou no Mahoutsukai")
    );
    assert_eq!(serie.serie_type, SerieType::Manga);
}

#[tokio::test]
async fn test_weebcentral_search() {
    let mock_server = MockServer::start().await;
    let fixture = include_str!("fixtures/search.html");

    Mock::given(method("POST"))
        .and(path("/v1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "ok",
            "message": "",
            "startTimestamp": 1000,
            "endTimestamp": 2000,
            "version": "3.3.0",
            "solution": {
                "url": "https://weebcentral.com/browse?q=test",
                "status": 200,
                "headers": {},
                "response": fixture,
                "cookies": [],
                "userAgent": "Mozilla/5.0"
            }
        })))
        .mount(&mock_server)
        .await;

    let weebcentral = WeebCentral::new(&mock_server.uri()).unwrap();
    let filters = SearchFilters {
        query: Some("test".to_string()),
        ..Default::default()
    };
    
    let result = weebcentral.search_series(&filters, 1).await;

    assert!(result.is_ok());
    let paginated = result.unwrap();
    
    assert!(!paginated.series.is_empty());
}

#[tokio::test]
async fn test_weebcentral_chapter_images() {
    let mock_server = MockServer::start().await;
    let fixture = include_str!("fixtures/chapter_images.html");

    Mock::given(method("POST"))
        .and(path("/v1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "ok",
            "message": "",
            "startTimestamp": 1000,
            "endTimestamp": 2000,
            "version": "3.3.0",
            "solution": {
                "url": "https://weebcentral.com/series/test/chapter/test",
                "status": 200,
                "headers": {},
                "response": fixture,
                "cookies": [],
                "userAgent": "Mozilla/5.0"
            }
        })))
        .mount(&mock_server)
        .await;

    let weebcentral = WeebCentral::new(&mock_server.uri()).unwrap();
    let result = weebcentral
        .fetch_chapter_data(
            &SerieId::new("test"),
            &VolumeId::new("all"),
            &ChapterId::new("test"),
        )
        .await;

    assert!(result.is_ok());
    match result.unwrap() {
        ChapterData::Image { images } => {
            assert!(!images.is_empty());
            assert!(images[0].url.contains("http"));
        }
        _ => panic!("Expected image data"),
    }
}
use std::time::Duration;

use reqwest::{Client, ClientBuilder, Response};
use serde::de::DeserializeOwned;
use tracing::{debug, trace};

use crate::retry::{retry_with_backoff, RetryConfig};

pub struct HttpClient {
    client: Client,
    retry_config: RetryConfig,
}

impl HttpClient {
    pub fn new() -> Result<Self, reqwest::Error> {
        Self::with_timeout(Duration::from_secs(30))
    }

    pub fn with_timeout(timeout: Duration) -> Result<Self, reqwest::Error> {
        let client = ClientBuilder::new()
            .timeout(timeout)
            .user_agent("Dokusho/1.0")
            .build()?;

        Ok(Self {
            client,
            retry_config: RetryConfig::default(),
        })
    }

    pub fn with_retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    pub async fn get(&self, url: &str) -> Result<Response, reqwest::Error> {
        debug!("GET {}", url);
        
        retry_with_backoff(
            || async {
                let response = self.client.get(url).send().await?;
                trace!("Response status: {}", response.status());
                
                // Check for server errors that should be retried
                if response.status().is_server_error() {
                    return response.error_for_status();
                }
                
                Ok(response)
            },
            &self.retry_config,
            &format!("GET {}", url),
        )
        .await
    }

    pub async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T, reqwest::Error> {
        let response = self.get(url).await?;
        response.error_for_status()?.json().await
    }

    pub async fn post_json<B, R>(&self, url: &str, body: &B) -> Result<R, reqwest::Error>
    where
        B: serde::Serialize + ?Sized,
        R: DeserializeOwned,
    {
        debug!("POST {}", url);
        
        retry_with_backoff(
            || async {
                let response = self.client.post(url).json(body).send().await?;
                trace!("Response status: {}", response.status());
                response.error_for_status()?.json().await
            },
            &self.retry_config,
            &format!("POST {}", url),
        )
        .await
    }

    pub fn inner(&self) -> &Client {
        &self.client
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default HTTP client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use wiremock::matchers::{method, path};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestData {
        message: String,
        value: i32,
    }

    #[tokio::test]
    async fn test_get_json() {
        let mock_server = MockServer::start().await;
        let test_data = TestData {
            message: "Hello".to_string(),
            value: 42,
        };

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&test_data))
            .mount(&mock_server)
            .await;

        let client = HttpClient::new().unwrap();
        let result: TestData = client
            .get_json(&format!("{}/test", mock_server.uri()))
            .await
            .unwrap();

        assert_eq!(result, test_data);
    }

    #[tokio::test]
    async fn test_post_json() {
        let mock_server = MockServer::start().await;
        let request_data = TestData {
            message: "Request".to_string(),
            value: 10,
        };
        let response_data = TestData {
            message: "Response".to_string(),
            value: 20,
        };

        Mock::given(method("POST"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response_data))
            .mount(&mock_server)
            .await;

        let client = HttpClient::new().unwrap();
        let result: TestData = client
            .post_json(&format!("{}/test", mock_server.uri()), &request_data)
            .await
            .unwrap();

        assert_eq!(result, response_data);
    }

    #[tokio::test]
    async fn test_retry_on_failure() {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;
        
        let mock_server = MockServer::start().await;
        let test_data = TestData {
            message: "Success".to_string(),
            value: 100,
        };
        
        let request_count = Arc::new(AtomicU32::new(0));
        let request_count_clone = request_count.clone();
        let test_data_clone = test_data.clone();

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(move |_req: &wiremock::Request| {
                let count = request_count_clone.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    ResponseTemplate::new(500)
                } else {
                    ResponseTemplate::new(200).set_body_json(&test_data_clone)
                }
            })
            .mount(&mock_server)
            .await;

        let client = HttpClient::new()
            .unwrap()
            .with_retry_config(RetryConfig {
                max_retries: 3,
                initial_interval: Duration::from_millis(10),
                ..Default::default()
            });

        let result: TestData = client
            .get_json(&format!("{}/test", mock_server.uri()))
            .await
            .unwrap();

        assert_eq!(result, test_data);
        assert_eq!(request_count.load(Ordering::SeqCst), 3);
    }
}
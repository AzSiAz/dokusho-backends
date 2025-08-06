use std::time::Duration;

use reqwest::{Client, ClientBuilder};
use tracing::{debug, error, info, warn};

use super::errors::FlareSolverError;
use super::types::{FlareSolverRequest, FlareSolverResponse};
use crate::retry::{retry_with_backoff, RetryConfig};

pub struct FlareSolverClient {
    client: Client,
    base_url: String,
    default_timeout: Duration,
    retry_config: RetryConfig,
}

impl FlareSolverClient {
    pub fn new(base_url: impl Into<String>) -> Result<Self, FlareSolverError> {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(120)) // 2 minutes max timeout
            .build()
            .map_err(FlareSolverError::Network)?;

        Ok(Self {
            client,
            base_url: base_url.into(),
            default_timeout: Duration::from_secs(60),
            retry_config: RetryConfig::default(),
        })
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self
    }

    pub async fn get_html(&self, url: &str) -> Result<String, FlareSolverError> {
        let url_clone = url.to_string();

        retry_with_backoff(
            || {
                let request = FlareSolverRequest::new(&url_clone)
                    .with_timeout(self.default_timeout.as_millis() as u32);
                self.send_request(request)
            },
            &self.retry_config,
            &format!("FlareSolver request to {}", url),
        )
        .await
        .and_then(|response| match response.solution {
            Some(solution) => Ok(solution.response),
            None => Err(FlareSolverError::NoSolution),
        })
    }

    pub async fn get_with_session(
        &self,
        url: &str,
        session: &str,
    ) -> Result<String, FlareSolverError> {
        let url_clone = url.to_string();
        let session_clone = session.to_string();

        retry_with_backoff(
            || {
                let request = FlareSolverRequest::new(&url_clone)
                    .with_session(session_clone.clone())
                    .with_timeout(self.default_timeout.as_millis() as u32);
                self.send_request(request)
            },
            &self.retry_config,
            &format!("FlareSolver request to {} with session", url),
        )
        .await
        .and_then(|response| match response.solution {
            Some(solution) => Ok(solution.response),
            None => Err(FlareSolverError::NoSolution),
        })
    }

    async fn send_request(
        &self,
        request: FlareSolverRequest,
    ) -> Result<FlareSolverResponse, FlareSolverError> {
        debug!("Sending FlareSolver request to URL: {}", request.url);

        let url = format!("{}/v1", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(FlareSolverError::Network)?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!(
                "FlareSolver request failed with status {}: {}",
                status, body
            );
            return Err(FlareSolverError::RequestFailed(format!(
                "HTTP {}: {}",
                status, body
            )));
        }

        let flare_response: FlareSolverResponse = response.json().await.map_err(|e| {
            error!("Failed to parse FlareSolver response: {}", e);
            FlareSolverError::InvalidResponse
        })?;

        if flare_response.status != "ok" {
            warn!("FlareSolver returned error: {}", flare_response.message);
            return Err(FlareSolverError::ErrorStatus(flare_response.message));
        }

        info!(
            "FlareSolver request completed in {}ms",
            flare_response.end_timestamp - flare_response.start_timestamp
        );

        Ok(flare_response)
    }

    pub async fn health_check(&self) -> Result<bool, FlareSolverError> {
        let url = format!("{}/health", self.base_url);

        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(e) => {
                warn!("FlareSolver health check failed: {}", e);
                Ok(false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_flaresolver_get_html() {
        let mock_server = MockServer::start().await;

        let mock_response = FlareSolverResponse {
            status: "ok".to_string(),
            message: "".to_string(),
            start_timestamp: 1000,
            end_timestamp: 2000,
            version: "3.3.0".to_string(),
            solution: Some(super::super::types::FlareSolverSolution {
                url: "https://example.com".to_string(),
                status: 200,
                headers: serde_json::json!({}),
                response: "<html><body>Test</body></html>".to_string(),
                cookies: vec![],
                user_agent: "Mozilla/5.0".to_string(),
            }),
        };

        Mock::given(method("POST"))
            .and(path("/v1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
            .mount(&mock_server)
            .await;

        let client = FlareSolverClient::new(mock_server.uri()).unwrap();
        let html = client.get_html("https://example.com").await.unwrap();

        assert_eq!(html, "<html><body>Test</body></html>");
    }

    #[tokio::test]
    async fn test_flaresolver_error_response() {
        let mock_server = MockServer::start().await;

        let error_response = FlareSolverResponse {
            status: "error".to_string(),
            message: "Cloudflare challenge failed".to_string(),
            start_timestamp: 1000,
            end_timestamp: 2000,
            version: "3.3.0".to_string(),
            solution: None,
        };

        Mock::given(method("POST"))
            .and(path("/v1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&error_response))
            .mount(&mock_server)
            .await;

        let client = FlareSolverClient::new(mock_server.uri()).unwrap();
        let result = client.get_html("https://example.com").await;

        assert!(matches!(result, Err(FlareSolverError::ErrorStatus(_))));
    }

    #[tokio::test]
    async fn test_health_check() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/health"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let client = FlareSolverClient::new(mock_server.uri()).unwrap();
        let is_healthy = client.health_check().await.unwrap();

        assert!(is_healthy);
    }
}

use std::time::Duration;

use reqwest::{Client, ClientBuilder, Response, StatusCode};
use serde::de::DeserializeOwned;
use tracing::{debug, info, trace, warn};

use crate::flaresolver::{FlareSolverClient, FlareSolverError};
use crate::retry::{retry_with_backoff, RetryConfig};

pub struct CloudflareAwareHttpClient {
    client: Client,
    flaresolver_client: Option<FlareSolverClient>,
    retry_config: RetryConfig,
}

impl CloudflareAwareHttpClient {
    pub fn new() -> Result<Self, reqwest::Error> {
        Self::with_timeout(Duration::from_secs(30))
    }

    pub fn with_timeout(timeout: Duration) -> Result<Self, reqwest::Error> {
        let client = ClientBuilder::new()
            .timeout(timeout)
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:71.0) Gecko/20100101 Firefox/77.0")
            .build()?;

        Ok(Self {
            client,
            flaresolver_client: None,
            retry_config: RetryConfig::default(),
        })
    }

    pub fn with_flaresolver(mut self, flaresolver_url: String) -> Result<Self, FlareSolverError> {
        self.flaresolver_client = Some(FlareSolverClient::new(flaresolver_url)?);
        Ok(self)
    }

    pub fn with_retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    pub async fn get(&self, url: &str) -> Result<Response, CloudflareError> {
        debug!("GET {}", url);

        retry_with_backoff(
            || async {
                let response = self.client.get(url).send().await?;
                trace!("Response status: {}", response.status());

                if Self::is_cloudflare_challenge(&response) {
                    warn!("Cloudflare challenge detected for {}", url);
                    
                    if let Some(flaresolver) = &self.flaresolver_client {
                        info!("Using FlareSolver to bypass Cloudflare for {}", url);
                        let _html = flaresolver.get_html(url).await
                            .map_err(|e| CloudflareError::FlareSolver(e))?;
                        
                        return Err(CloudflareError::CannotReturnResponseFromFlareSolver);
                    } else {
                        return Err(CloudflareError::CloudflareBlocked);
                    }
                }

                if response.status().is_server_error() {
                    return Err(CloudflareError::ServerError(response.status()));
                }

                Ok(response)
            },
            &self.retry_config,
            &format!("GET {}", url),
        )
        .await
    }

    pub async fn get_html(&self, url: &str) -> Result<String, CloudflareError> {
        debug!("GET HTML {}", url);

        retry_with_backoff(
            || async {
                let response = self.client.get(url).send().await?;
                trace!("Response status: {}", response.status());

                if Self::is_cloudflare_challenge(&response) {
                    warn!("Cloudflare challenge detected for {}", url);
                    
                    if let Some(flaresolver) = &self.flaresolver_client {
                        info!("Using FlareSolver to bypass Cloudflare for {}", url);
                        return flaresolver.get_html(url).await
                            .map_err(|e| CloudflareError::FlareSolver(e));
                    } else {
                        return Err(CloudflareError::CloudflareBlocked);
                    }
                }

                if response.status().is_server_error() {
                    return Err(CloudflareError::ServerError(response.status()));
                }

                response.text().await.map_err(|e| e.into())
            },
            &self.retry_config,
            &format!("GET HTML {}", url),
        )
        .await
    }

    pub async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T, CloudflareError> {
        let response = self.get(url).await?;
        response.json().await.map_err(|e| e.into())
    }

    pub async fn post_json<B, R>(&self, url: &str, body: &B) -> Result<R, CloudflareError>
    where
        B: serde::Serialize + ?Sized,
        R: DeserializeOwned,
    {
        debug!("POST {}", url);

        retry_with_backoff(
            || async {
                let response = self.client.post(url).json(body).send().await?;
                trace!("Response status: {}", response.status());
                
                if Self::is_cloudflare_challenge(&response) {
                    return Err(CloudflareError::CloudflareBlocked);
                }

                if response.status().is_server_error() {
                    return Err(CloudflareError::ServerError(response.status()));
                }

                response.json().await.map_err(|e| e.into())
            },
            &self.retry_config,
            &format!("POST {}", url),
        )
        .await
    }

    fn is_cloudflare_challenge(response: &Response) -> bool {
        if response.status() == StatusCode::FORBIDDEN || response.status() == StatusCode::SERVICE_UNAVAILABLE {
            if let Some(server) = response.headers().get("server") {
                if let Ok(server_str) = server.to_str() {
                    if server_str.to_lowercase().contains("cloudflare") {
                        return true;
                    }
                }
            }
            
            if let Some(cf_ray) = response.headers().get("cf-ray") {
                return cf_ray.to_str().is_ok();
            }
        }
        
        false
    }

    pub fn inner(&self) -> &Client {
        &self.client
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CloudflareError {
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
    
    #[error("FlareSolver error: {0}")]
    FlareSolver(FlareSolverError),
    
    #[error("Cloudflare protection detected but FlareSolver not configured")]
    CloudflareBlocked,
    
    #[error("Cannot return Response object when using FlareSolver")]
    CannotReturnResponseFromFlareSolver,
    
    #[error("Server error: {0}")]
    ServerError(StatusCode),
}

impl Default for CloudflareAwareHttpClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default HTTP client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_normal_request() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(200).set_body_string("Hello World"))
            .mount(&mock_server)
            .await;

        let client = CloudflareAwareHttpClient::new().unwrap();
        let result = client.get_html(&format!("{}/test", mock_server.uri())).await.unwrap();
        
        assert_eq!(result, "Hello World");
    }

    #[tokio::test]
    async fn test_cloudflare_detection() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(
                ResponseTemplate::new(403)
                    .insert_header("server", "cloudflare")
                    .insert_header("cf-ray", "123456789")
                    .set_body_string("Cloudflare challenge")
            )
            .mount(&mock_server)
            .await;

        let client = CloudflareAwareHttpClient::new().unwrap();
        let result = client.get_html(&format!("{}/test", mock_server.uri())).await;
        
        assert!(matches!(result, Err(CloudflareError::CloudflareBlocked)));
    }
}
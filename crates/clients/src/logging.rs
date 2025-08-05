use std::time::Instant;

use reqwest::{Request, Response};
use tracing::{debug, trace};

pub fn log_request(request: &Request) {
    debug!(
        method = %request.method(),
        url = %request.url(),
        "Sending HTTP request"
    );
    
    trace!(
        headers = ?request.headers(),
        "Request headers"
    );
}

pub fn log_response(response: &Response, duration: std::time::Duration) {
    let status = response.status();
    
    if status.is_success() {
        debug!(
            status = %status,
            duration_ms = duration.as_millis(),
            "HTTP request completed successfully"
        );
    } else {
        debug!(
            status = %status,
            duration_ms = duration.as_millis(),
            "HTTP request failed"
        );
    }
    
    trace!(
        headers = ?response.headers(),
        "Response headers"
    );
}

pub struct LoggingMiddleware;

impl LoggingMiddleware {
    pub async fn wrap_request<F, Fut>(operation: F) -> Result<Response, reqwest::Error>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<Response, reqwest::Error>>,
    {
        let start = Instant::now();
        let result = operation().await;
        let duration = start.elapsed();
        
        match &result {
            Ok(response) => log_response(response, duration),
            Err(err) => {
                debug!(
                    error = %err,
                    duration_ms = duration.as_millis(),
                    "HTTP request error"
                );
            }
        }
        
        result
    }
}
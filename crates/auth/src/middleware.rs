#[cfg(feature = "axum")]
pub mod axum_middleware {
    use axum::{
        extract::Request,
        http::{header, StatusCode},
        middleware::Next,
        response::Response,
    };

    use crate::token::validate_jwt;

    /// Authentication middleware for Axum
    /// Validates JWT tokens in the Authorization header
    pub async fn auth_middleware(
        mut req: Request,
        next: Next,
        jwt_secret: String,
        skip_paths: Vec<String>,
    ) -> Result<Response, StatusCode> {
        // Skip auth for specified paths
        let path = req.uri().path();
        if skip_paths.iter().any(|p| path == p) {
            return Ok(next.run(req).await);
        }

        // Extract token from Authorization header
        let token = req
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|auth_header| auth_header.to_str().ok())
            .and_then(|auth_value| {
                auth_value
                    .strip_prefix("Bearer ")
                    .map(|token| token.to_string())
            });

        match token {
            Some(token) => {
                // Validate token
                match validate_jwt(&token, &jwt_secret) {
                    Ok(claims) => {
                        // Insert claims into request extensions
                        req.extensions_mut().insert(claims);
                        Ok(next.run(req).await)
                    }
                    Err(e) => {
                        tracing::warn!("Invalid JWT: {}", e);
                        Err(StatusCode::UNAUTHORIZED)
                    }
                }
            }
            None => {
                // No token provided
                Err(StatusCode::UNAUTHORIZED)
            }
        }
    }

    /// API Key middleware for Axum
    /// Validates API keys in the X-API-Key header
    pub async fn api_key_middleware(
        req: Request,
        next: Next,
        api_key: String,
        skip_paths: Vec<String>,
    ) -> Result<Response, StatusCode> {
        // Skip API key check for specified paths
        let path = req.uri().path();
        if skip_paths.iter().any(|p| path == p) {
            return Ok(next.run(req).await);
        }

        // Check for API key in header
        let provided_key = req.headers().get("X-API-Key").and_then(|v| v.to_str().ok());

        match provided_key {
            Some(key) if key == api_key => Ok(next.run(req).await),
            _ => {
                tracing::warn!("Invalid or missing API key");
                Err(StatusCode::UNAUTHORIZED)
            }
        }
    }
}

#[cfg(feature = "axum")]
pub use axum_middleware::*;

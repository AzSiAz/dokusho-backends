use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use chrono::{Duration, Utc};
use openidconnect::{
    core::{CoreClient, CoreProviderMetadata},
    reqwest::async_http_client,
    AuthorizationCode, ClientId, ClientSecret, IssuerUrl, RedirectUrl,
    OAuth2TokenResponse as TokenResponse,
};
use serde::Deserialize;
use std::sync::Arc;

use super::{generate_jwt, Claims};
use crate::{config::AppConfig, AppState};

#[derive(Debug, Deserialize)]
pub struct AuthCallbackQuery {
    code: String,
    state: Option<String>,
}

pub async fn auth_callback(
    Query(params): Query<AuthCallbackQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match handle_auth_callback(params, &state.config).await {
        Ok(token) => {
            // Redirect to frontend with token
            let redirect_url = format!("/?token={}", token);
            Redirect::to(&redirect_url).into_response()
        }
        Err(e) => {
            tracing::error!("Auth callback error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Authentication failed").into_response()
        }
    }
}

async fn handle_auth_callback(
    params: AuthCallbackQuery,
    config: &AppConfig,
) -> Result<String, anyhow::Error> {
    let issuer_url = IssuerUrl::new(
        config.auth.issuer_url
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Issuer URL not configured"))?
            .clone()
    )?;

    let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, async_http_client).await?;

    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(
            config.auth.client_id
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Client ID not configured"))?
                .clone()
        ),
        Some(ClientSecret::new(
            config.auth.client_secret
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Client secret not configured"))?
                .clone()
        )),
    )
    .set_redirect_uri(RedirectUrl::new(
        config.auth.redirect_url
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Redirect URL not configured"))?
            .clone()
    )?);

    // Exchange the authorization code for an access token
    let token_response = client
        .exchange_code(AuthorizationCode::new(params.code))
        .request_async(async_http_client)
        .await?;

    // Get user info
    let userinfo_claims: openidconnect::UserInfoClaims<
        openidconnect::EmptyAdditionalClaims,
        openidconnect::core::CoreGenderClaim,
    > = client
        .user_info(token_response.access_token().to_owned(), None)?
        .request_async(async_http_client)
        .await?;

    // Create JWT claims
    let claims = Claims {
        sub: userinfo_claims.subject().to_string(),
        email: userinfo_claims.email().map(|e| e.to_string()),
        name: userinfo_claims.name().and_then(|n| n.get(None)).map(|n| n.to_string()),
        exp: (Utc::now() + Duration::hours(config.auth.jwt_expiry_hours as i64)).timestamp() as usize,
        iat: Utc::now().timestamp() as usize,
    };

    // Generate JWT
    let token = generate_jwt(&claims, &config.auth.jwt_secret)?;

    Ok(token)
}
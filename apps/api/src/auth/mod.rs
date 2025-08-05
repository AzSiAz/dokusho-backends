pub mod callback;
pub mod middleware;

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use openidconnect::{
    core::{CoreClient, CoreProviderMetadata},
    reqwest::async_http_client,
    ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce, RedirectUrl,
};
use serde::{Deserialize, Serialize};

use crate::config::AppConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub exp: usize,
    pub iat: usize,
}

pub async fn generate_auth_url(config: &AppConfig) -> Result<String, anyhow::Error> {
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

    let (auth_url, _csrf_token, _nonce) = client
        .authorize_url(
            openidconnect::AuthenticationFlow::<openidconnect::core::CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(openidconnect::Scope::new("openid".to_string()))
        .add_scope(openidconnect::Scope::new("email".to_string()))
        .add_scope(openidconnect::Scope::new("profile".to_string()))
        .url();

    Ok(auth_url.to_string())
}

pub fn generate_jwt(claims: &Claims, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn validate_jwt(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
}
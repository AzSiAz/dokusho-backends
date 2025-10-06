use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;

use dokusho_config::AuthConfig;
use dokusho_database::{
    Database,
    models::user::{User, UserRole},
};
use openidconnect::UserInfoClaims;
use openidconnect::{
    AccessToken, ClientId, IdToken, IdTokenClaims, IdTokenVerifier, IssuerUrl, JsonWebKeySet,
    Nonce,
    core::{
        CoreGenderClaim, CoreJsonWebKey, CoreJweContentEncryptionAlgorithm,
        CoreJwsSigningAlgorithm, CoreProviderMetadata,
    },
};
use reqwest::Client as HttpClient;
use tokio::sync::RwLock;
use tracing::debug;

use crate::CustomClaims;
use crate::{errors::AuthError, openid::OpenIDClient};

/// Cache entry for validated tokens
#[derive(Clone, Debug)]
struct TokenCacheEntry {
    user: User,
    validated_at: Instant,
}

/// Auth service that validates OpenID access tokens
pub struct AuthService {
    database: Database,
    config: AuthConfig,
    openid_client: OpenIDClient,
    provider_metadata: CoreProviderMetadata,
    /// Cache for validated tokens (token -> user)
    token_cache: Arc<RwLock<std::collections::HashMap<String, TokenCacheEntry>>>,
    /// JWKS cache
    jwks: Arc<RwLock<JsonWebKeySet<CoreJsonWebKey>>>,
    jwks_fetched_at: Arc<RwLock<Instant>>,
}

impl AuthService {
    pub async fn new(database: Database, config: AuthConfig) -> Result<Self, AuthError> {
        // Initialize OpenID client
        let openid_client = OpenIDClient::new(config.clone()).await?;
        debug!("OpenID client initialized");

        // Get provider metadata
        let issuer_url = IssuerUrl::new(config.issuer_url.clone())
            .map_err(|e| AuthError::Configuration(format!("Invalid issuer URL: {}", e)))?;

        let http_client = HttpClient::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| AuthError::Network(format!("Failed to build HTTP client: {}", e)))?;

        let provider_metadata =
            CoreProviderMetadata::discover_async(issuer_url, &http_client).await?;
        debug!("Provider metadata fetched");

        // Fetch initial JWKS
        let jwks_url = provider_metadata.jwks_uri().url().clone();
        let jwks: JsonWebKeySet<CoreJsonWebKey> = http_client
            .get(jwks_url)
            .send()
            .await
            .map_err(|e| AuthError::Network(format!("Failed to fetch JWKS: {}", e)))?
            .json()
            .await
            .map_err(|e| AuthError::Network(format!("Failed to parse JWKS: {}", e)))?;
        debug!("JWKS fetched");

        Ok(Self {
            database,
            config: config.clone(),
            openid_client,
            provider_metadata,
            token_cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
            jwks: Arc::new(RwLock::new(jwks)),
            jwks_fetched_at: Arc::new(RwLock::new(Instant::now())),
        })
    }

    /// Validate an access token and return the associated user
    pub async fn validate_access_token(&self, token: &str) -> Result<User, AuthError> {
        // Check cache first
        if let Some(user) = self.check_token_cache(token).await {
            return Ok(user);
        }

        // Try to validate as JWT
        let user = match self.validate_jwt_token(token).await {
            Ok(user) => user,
            Err(_) => {
                // If JWT validation fails, try token introspection
                // This would be for opaque tokens, but most OpenID providers use JWT
                self.validate_via_userinfo(token).await?
            }
        };

        // Cache the result
        self.cache_token_validation(token, user.clone()).await;

        Ok(user)
    }

    /// Validate a JWT access token
    async fn validate_jwt_token(&self, token: &str) -> Result<User, AuthError> {
        // Ensure JWKS is fresh
        self.refresh_jwks_if_needed().await?;

        // Parse the token with support for custom claims (groups)
        let id_token: IdToken<
            crate::models::CustomClaims,
            CoreGenderClaim,
            CoreJweContentEncryptionAlgorithm,
            CoreJwsSigningAlgorithm,
        > = IdToken::from_str(token)
            .map_err(|e| AuthError::InvalidToken(format!("Failed to parse token: {}", e)))?;

        // Get JWKS for validation
        let jwks = self.jwks.read().await;

        // Create verifier for custom-claim id tokens
        let verifier = IdTokenVerifier::new_public_client(
            ClientId::new(self.config.client_id.clone()),
            self.provider_metadata.issuer().clone(),
            jwks.clone(),
        );

        // Validate the token
        // Note: We're treating access tokens as ID tokens for validation purposes
        // This works because many OpenID providers use JWT for both
        let claims = id_token
            .claims(&verifier, &Nonce::new("unused".to_string()))
            .map_err(|e| AuthError::InvalidToken(format!("Token validation failed: {}", e)))?;

        // Extract user information from claims
        let sub = claims.subject().to_string();
        let email = claims.email().map(|e| e.to_string());
        let name = claims
            .name()
            .and_then(|n| n.get(None))
            .map(|n| n.to_string());

        // Determine role from claims (customize based on your provider)
        let role = self.determine_role_from_claims(claims);

        // Create or update user in database
        let user = self
            .database
            .users()
            .create_or_update_user(sub, email, name, role)
            .await?;

        Ok(user)
    }

    /// Validate token via UserInfo endpoint (fallback for opaque tokens)
    async fn validate_via_userinfo(&self, token: &str) -> Result<User, AuthError> {
        let access_token = AccessToken::new(token.to_string());

        // Get user info from the OpenID provider
        let userinfo = self.openid_client.get_user_info(access_token).await?;

        let sub = userinfo.subject().to_string();
        let email = userinfo.email().map(|e| e.to_string());
        let name = userinfo
            .name()
            .and_then(|n| n.get(None))
            .map(|n| n.to_string());

        // Determine role
        let role = self.determine_user_role(&userinfo);

        // Create or update user in database
        let user = self
            .database
            .users()
            .create_or_update_user(sub, email, name, role)
            .await?;

        Ok(user)
    }

    /// Check if a token is in the cache and still valid
    async fn check_token_cache(&self, token: &str) -> Option<User> {
        let cache = self.token_cache.read().await;
        if let Some(entry) = cache.get(token)
            && entry.validated_at.elapsed() < self.config.token_cache_ttl
        {
            return Some(entry.user.clone());
        }
        None
    }

    /// Cache a successful token validation
    async fn cache_token_validation(&self, token: &str, user: User) {
        let mut cache = self.token_cache.write().await;

        // Clean up old entries (simple cleanup strategy)
        if cache.len() > 1000 {
            cache.retain(|_, entry| entry.validated_at.elapsed() < self.config.token_cache_ttl);
        }

        cache.insert(
            token.to_string(),
            TokenCacheEntry {
                user,
                validated_at: Instant::now(),
            },
        );
    }

    /// Refresh JWKS if needed
    async fn refresh_jwks_if_needed(&self) -> Result<(), AuthError> {
        let last_fetched = *self.jwks_fetched_at.read().await;

        if last_fetched.elapsed() > self.config.jwks_cache_ttl {
            let http_client = HttpClient::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .map_err(|e| AuthError::Network(format!("Failed to build HTTP client: {}", e)))?;

            let jwks_url = self.provider_metadata.jwks_uri().url().clone();
            let new_jwks: JsonWebKeySet<CoreJsonWebKey> = http_client
                .get(jwks_url)
                .send()
                .await
                .map_err(|e| AuthError::Network(format!("Failed to fetch JWKS: {}", e)))?
                .json()
                .await
                .map_err(|e| AuthError::Network(format!("Failed to parse JWKS: {}", e)))?;

            *self.jwks.write().await = new_jwks;
            *self.jwks_fetched_at.write().await = Instant::now();
        }

        Ok(())
    }

    /// Determine user role from ID token claims (using custom `groups` claim)
    fn determine_role_from_claims(
        &self,
        claims: &IdTokenClaims<CustomClaims, CoreGenderClaim>,
    ) -> UserRole {
        self.role_from_groups(claims.additional_claims().groups.as_ref())
    }

    /// Determine user role from UserInfo
    fn determine_user_role(
        &self,
        userinfo: &UserInfoClaims<CustomClaims, CoreGenderClaim>,
    ) -> UserRole {
        self.role_from_groups(userinfo.additional_claims().groups.as_ref())
    }

    fn role_from_groups(&self, groups: Option<&Vec<String>>) -> UserRole {
        let admin = self.config.group_admin.to_lowercase();

        let lowered = groups
            .map(|v| v.iter().map(|g| g.to_lowercase()).collect::<Vec<_>>())
            .unwrap_or_default();

        if lowered.iter().any(|g| g == &admin) {
            UserRole::Admin
        } else {
            UserRole::User
        }
    }

    /// Clear the token cache (useful for logout or testing)
    pub async fn clear_cache(&self) {
        self.token_cache.write().await.clear();
    }
}

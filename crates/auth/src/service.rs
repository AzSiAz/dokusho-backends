use std::sync::Arc;

use dokusho_config::AuthConfig;
use openidconnect::{CsrfToken, Nonce, OAuth2TokenResponse};

use dokusho_database::{
    Database,
    models::user::{User, UserRole},
};

use crate::{
    errors::AuthError,
    models::{AuthenticationRequest, AuthenticationResponse, Claims, TokenResponse, UserInfo},
    openid::OpenIDClient,
    token::{generate_jwt, generate_nonce, generate_state_token, hash_token},
};

pub struct AuthService {
    openid_client: OpenIDClient,
    database: Arc<Database>,
    config: AuthConfig,
}

impl AuthService {
    pub async fn new(database: Arc<Database>, config: AuthConfig) -> Result<Self, AuthError> {
        let openid_client = OpenIDClient::new(config.clone()).await?;

        Ok(Self {
            openid_client,
            database,
            config,
        })
    }

    pub async fn initiate_authentication(
        &self,
        request: AuthenticationRequest,
    ) -> Result<AuthenticationResponse, AuthError> {
        // Validate redirect URL against whitelist
        if !self.is_redirect_url_allowed(&request.redirect_uri) {
            return Err(AuthError::Configuration(format!(
                "Redirect URL '{}' is not in the allowed list",
                request.redirect_uri
            )));
        }

        let state = generate_state_token();
        let nonce = generate_nonce();

        // Build absolute callback URL from BASE_URL and path-only callback
        let callback_url = format!(
            "{}/{}",
            self.config.base_url.trim_end_matches('/'),
            self.config.oauth_callback_url.trim_start_matches('/')
        );

        // Generate authorization URL with OAuth callback and PKCE
        let (auth_url, _, _, pkce_verifier) = self.openid_client.generate_authorization_url(
            callback_url,
            CsrfToken::new(state.clone()),
            Nonce::new(nonce.clone()),
        )?;

        // Store state in database with the redirect URI and PKCE verifier
        self.database
            .auth_states()
            .create(
                state.clone(),
                request.redirect_uri.clone(),
                nonce,
                Some(pkce_verifier.secret().to_string()),
            )
            .await?;

        Ok(AuthenticationResponse {
            authorization_url: auth_url.to_string(),
            state,
        })
    }

    pub async fn complete_authentication(
        &self,
        code: String,
        state: String,
    ) -> Result<TokenResponse, AuthError> {
        // Validate state
        let auth_state = self
            .database
            .auth_states()
            .find_by_state(&state)
            .await?
            .ok_or(AuthError::InvalidState)?;

        if dokusho_database::repositories::auth_state::AuthStateRepository::is_expired(&auth_state)
        {
            return Err(AuthError::StateExpired);
        }

        // Exchange code for tokens with PKCE verifier
        let pkce_verifier = auth_state
            .pkce_verifier
            .map(openidconnect::PkceCodeVerifier::new)
            .ok_or_else(|| AuthError::Configuration("Missing PKCE verifier".to_string()))?;

        let token_response = self
            .openid_client
            .exchange_code(code, pkce_verifier)
            .await?;

        // Get user info
        let userinfo = self
            .openid_client
            .get_user_info(token_response.access_token().clone())
            .await?;

        // Determine user role based on groups
        let role = self.determine_user_role(&userinfo);

        // Create or update user
        let user = self
            .database
            .users()
            .create_or_update_user(
                userinfo.subject().to_string(),
                userinfo.email().map(|e| e.to_string()),
                userinfo
                    .name()
                    .and_then(|n| n.get(None))
                    .map(|n| n.to_string()),
                role,
            )
            .await?;

        // Create JWT
        let claims = Claims::new(
            user.id,
            user.sub.clone(),
            user.email.clone(),
            user.name.clone(),
            user.role,
            self.config.jwt_expiry_hours,
        );

        let jwt = generate_jwt(&claims, &self.config.jwt_secret)?;

        // Create session
        let token_hash = hash_token(&jwt);
        self.database
            .users()
            .create_session(user.id, token_hash, self.config.jwt_expiry_hours)
            .await?;

        // Clean up auth state
        self.database.auth_states().delete(&state).await?;

        Ok(TokenResponse {
            access_token: jwt,
            token_type: "Bearer".to_string(),
            expires_in: self.config.jwt_expiry_hours * 3600,
            user: UserInfo {
                id: user.id,
                sub: user.sub,
                email: user.email,
                name: user.name,
                role: user.role,
                created_at: user
                    .created_at
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(chrono::Utc::now),
            },
        })
    }

    /// Validates a token completely: JWT signature, expiration, and database session
    /// This is the main validation method that should be used everywhere
    pub async fn validate_token_and_session(
        &self,
        token: &str,
    ) -> Result<(Claims, User), AuthError> {
        // First validate JWT (checks signature and exp claim)
        let claims = crate::token::validate_jwt(token, &self.config.jwt_secret)?;

        if claims.is_expired() {
            return Err(AuthError::TokenExpired);
        }

        // Then check session in database (ensures not logged out)
        let token_hash = hash_token(token);
        let session = self
            .database
            .users()
            .find_session_by_token(&token_hash)
            .await?
            .ok_or(AuthError::InvalidToken)?;

        // Double-check expiration (belt and suspenders)
        if dokusho_database::repositories::user::UserRepository::is_session_expired(&session) {
            return Err(AuthError::TokenExpired);
        }

        // Verify the session belongs to the same user as the JWT claims
        if session.user_id != claims.user_id {
            return Err(AuthError::InvalidToken);
        }

        // Update last used timestamp
        self.database
            .users()
            .update_session_last_used(session.id)
            .await?;

        // Get user
        let user = self
            .database
            .users()
            .find_by_id(session.user_id)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        Ok((claims, user))
    }

    /// Legacy method for compatibility - validates session and returns user only
    pub async fn validate_session(&self, token: &str) -> Result<User, AuthError> {
        let (_, user) = self.validate_token_and_session(token).await?;
        Ok(user)
    }

    pub async fn refresh_token(&self, old_token: &str) -> Result<String, AuthError> {
        // Validate token and session completely (JWT, database, expiration, etc.)
        let (_claims, user) = self.validate_token_and_session(old_token).await?;

        // Create new claims
        let claims = Claims::new(
            user.id,
            user.sub.clone(),
            user.email.clone(),
            user.name.clone(),
            user.role,
            self.config.jwt_expiry_hours,
        );

        // Generate new JWT
        let new_jwt = generate_jwt(&claims, &self.config.jwt_secret)?;

        // Use a transaction to atomically delete old session and create new one
        let old_token_hash = hash_token(old_token);
        let new_token_hash = hash_token(&new_jwt);

        // First find and delete the old session
        if let Some(old_session) = self
            .database
            .users()
            .find_session_by_token(&old_token_hash)
            .await?
        {
            self.database.users().delete_session(old_session.id).await?;
        }

        // Create new session
        self.database
            .users()
            .create_session(user.id, new_token_hash, self.config.jwt_expiry_hours)
            .await?;

        Ok(new_jwt)
    }

    pub async fn logout(&self, token: &str) -> Result<(), AuthError> {
        let token_hash = hash_token(token);

        // Find and delete session
        if let Some(session) = self
            .database
            .users()
            .find_session_by_token(&token_hash)
            .await?
        {
            self.database.users().delete_session(session.id).await?;
        }

        Ok(())
    }

    pub async fn cleanup_expired(&self) -> Result<(), AuthError> {
        self.database.auth_states().delete_expired().await?;
        self.database.users().delete_expired_sessions().await?;
        Ok(())
    }

    fn determine_user_role(
        &self,
        userinfo: &openidconnect::UserInfoClaims<
            crate::models::CustomClaims,
            openidconnect::core::CoreGenderClaim,
        >,
    ) -> UserRole {
        // Check if user has groups in additional claims
        if let Some(groups) = userinfo.additional_claims().groups.as_ref() {
            // Check if user is in admin group
            if groups.contains(&self.config.group_admin) {
                return UserRole::Admin;
            }
        }

        // Default to user role
        UserRole::User
    }

    fn is_redirect_url_allowed(&self, redirect_url: &str) -> bool {
        self.config.allowed_redirect_urls.iter().any(|allowed| {
            // Exact match or wildcard match
            if allowed.ends_with("*") {
                let prefix = &allowed[..allowed.len() - 1];
                redirect_url.starts_with(prefix)
            } else {
                redirect_url == allowed
            }
        })
    }
}

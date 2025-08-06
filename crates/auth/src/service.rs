use openidconnect::{CsrfToken, Nonce, OAuth2TokenResponse};

use dokusho_database::{
    models::User,
    repositories::{AuthStateRepository, UserRepository},
};

use crate::{
    errors::AuthError,
    models::{
        AuthConfig, AuthenticationRequest, AuthenticationResponse, Claims, TokenResponse, UserInfo,
    },
    openid::OpenIDClient,
    token::{generate_jwt, generate_nonce, generate_state_token, hash_token},
};

pub struct AuthService {
    openid_client: OpenIDClient,
    user_repo: UserRepository,
    auth_state_repo: AuthStateRepository,
    config: AuthConfig,
}

impl AuthService {
    pub async fn new(
        user_repo: UserRepository,
        auth_state_repo: AuthStateRepository,
        config: AuthConfig,
    ) -> Result<Self, AuthError> {
        let openid_client = OpenIDClient::new(config.clone()).await?;

        Ok(Self {
            openid_client,
            user_repo,
            auth_state_repo,
            config,
        })
    }

    pub async fn initiate_authentication(
        &self,
        request: AuthenticationRequest,
    ) -> Result<AuthenticationResponse, AuthError> {
        let state = generate_state_token();
        let nonce = generate_nonce();

        // Store state in database
        self.auth_state_repo
            .create(state.clone(), request.redirect_uri.clone(), nonce.clone())
            .await?;

        // Generate authorization URL
        let (auth_url, _, _) = self
            .openid_client
            .generate_authorization_url(CsrfToken::new(state.clone()), Nonce::new(nonce));

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
            .auth_state_repo
            .find_by_state(&state)
            .await?
            .ok_or(AuthError::InvalidState)?;

        if auth_state.is_expired() {
            return Err(AuthError::StateExpired);
        }

        // Exchange code for tokens
        let token_response = self.openid_client.exchange_code(code).await?;

        // Get user info
        let userinfo = self
            .openid_client
            .get_user_info(token_response.access_token().clone())
            .await?;

        // Create or update user
        let user = self
            .user_repo
            .create_or_update_user(
                userinfo.subject().to_string(),
                userinfo.email().map(|e| e.to_string()),
                userinfo
                    .name()
                    .and_then(|n| n.get(None))
                    .map(|n| n.to_string()),
            )
            .await?;

        // Create JWT
        let claims = Claims::new(
            user.id,
            user.sub.clone(),
            user.email.clone(),
            user.name.clone(),
            self.config.jwt_expiry_hours,
        );

        let jwt = generate_jwt(&claims, &self.config.jwt_secret)?;

        // Create session
        let token_hash = hash_token(&jwt);
        self.user_repo
            .create_session(user.id, token_hash, self.config.jwt_expiry_hours)
            .await?;

        // Clean up auth state
        self.auth_state_repo.delete(&state).await?;

        Ok(TokenResponse {
            access_token: jwt,
            token_type: "Bearer".to_string(),
            expires_in: self.config.jwt_expiry_hours * 3600,
            user: UserInfo {
                id: user.id,
                sub: user.sub,
                email: user.email,
                name: user.name,
                created_at: user.created_at.unwrap_or_else(chrono::Utc::now),
            },
        })
    }

    pub async fn validate_session(&self, token: &str) -> Result<User, AuthError> {
        // Validate JWT
        let claims = crate::token::validate_jwt(token, &self.config.jwt_secret)?;

        if claims.is_expired() {
            return Err(AuthError::TokenExpired);
        }

        // Check session in database
        let token_hash = hash_token(token);
        let session = self
            .user_repo
            .find_session_by_token(&token_hash)
            .await?
            .ok_or(AuthError::InvalidToken)?;

        if session.is_expired() {
            return Err(AuthError::TokenExpired);
        }

        // Update last used timestamp
        self.user_repo.update_session_last_used(session.id).await?;

        // Get user
        let user = self
            .user_repo
            .find_by_id(session.user_id)
            .await?
            .ok_or(AuthError::Unauthorized)?;

        Ok(user)
    }

    pub async fn refresh_token(&self, old_token: &str) -> Result<String, AuthError> {
        // Validate existing token
        let user = self.validate_session(old_token).await?;

        // Create new claims
        let claims = Claims::new(
            user.id,
            user.sub.clone(),
            user.email.clone(),
            user.name.clone(),
            self.config.jwt_expiry_hours,
        );

        // Generate new JWT
        let new_jwt = generate_jwt(&claims, &self.config.jwt_secret)?;

        // Create new session
        let token_hash = hash_token(&new_jwt);
        self.user_repo
            .create_session(user.id, token_hash, self.config.jwt_expiry_hours)
            .await?;

        Ok(new_jwt)
    }

    pub async fn logout(&self, token: &str) -> Result<(), AuthError> {
        let token_hash = hash_token(token);

        // Find and delete session
        if let Some(session) = self.user_repo.find_session_by_token(&token_hash).await? {
            self.user_repo.delete_session(session.id).await?;
        }

        Ok(())
    }

    pub async fn cleanup_expired(&self) -> Result<(), AuthError> {
        self.auth_state_repo.delete_expired().await?;
        self.user_repo.delete_expired_sessions().await?;
        Ok(())
    }
}

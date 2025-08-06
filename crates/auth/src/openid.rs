use openidconnect::{
    core::{CoreClient, CoreProviderMetadata, CoreResponseType, CoreTokenResponse},
    reqwest::async_http_client,
    url::Url,
    AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce,
    RedirectUrl, Scope,
};

use crate::{errors::AuthError, models::AuthConfig};

pub struct OpenIDClient {
    client: CoreClient,
    config: AuthConfig,
}

impl OpenIDClient {
    pub async fn new(config: AuthConfig) -> Result<Self, AuthError> {
        if !config.enabled {
            return Err(AuthError::Configuration(
                "Authentication is not enabled".to_string(),
            ));
        }

        let issuer_url = IssuerUrl::new(config.issuer_url.clone())
            .map_err(|e| AuthError::Configuration(format!("Invalid issuer URL: {}", e)))?;

        let provider_metadata =
            CoreProviderMetadata::discover_async(issuer_url.clone(), async_http_client).await?;

        let client = CoreClient::from_provider_metadata(
            provider_metadata,
            ClientId::new(config.client_id.clone()),
            Some(ClientSecret::new(config.client_secret.clone())),
        )
        .set_redirect_uri(
            RedirectUrl::new(config.redirect_url.clone())
                .map_err(|e| AuthError::Configuration(format!("Invalid redirect URL: {}", e)))?,
        );

        Ok(Self { client, config })
    }

    pub fn generate_authorization_url(
        &self,
        state: CsrfToken,
        nonce: Nonce,
    ) -> (Url, CsrfToken, Nonce) {
        self.client
            .authorize_url(
                AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
                || state,
                || nonce,
            )
            .add_scope(Scope::new("openid".to_string()))
            .add_scope(Scope::new("email".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .url()
    }

    pub async fn exchange_code(&self, code: String) -> Result<CoreTokenResponse, AuthError> {
        let token_response = self
            .client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(async_http_client)
            .await?;

        Ok(token_response)
    }

    pub async fn get_user_info(
        &self,
        access_token: openidconnect::AccessToken,
    ) -> Result<
        openidconnect::UserInfoClaims<
            openidconnect::EmptyAdditionalClaims,
            openidconnect::core::CoreGenderClaim,
        >,
        AuthError,
    > {
        let userinfo_request = self
            .client
            .user_info(access_token, None)
            .map_err(|e| AuthError::OpenIDConnect(e.to_string()))?;

        let userinfo = userinfo_request
            .request_async(async_http_client)
            .await
            .map_err(|e| AuthError::OpenIDConnect(e.to_string()))?;

        Ok(userinfo)
    }

    pub fn config(&self) -> &AuthConfig {
        &self.config
    }
}

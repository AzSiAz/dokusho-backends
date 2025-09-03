use dokusho_config::AuthConfig;
use openidconnect::{
    ClientId, ClientSecret, EmptyAdditionalClaims, EndpointMaybeSet, EndpointNotSet, EndpointSet,
    IssuerUrl, StandardErrorResponse,
    core::{
        CoreAuthDisplay, CoreAuthPrompt, CoreErrorResponseType, CoreGenderClaim, CoreJsonWebKey,
        CoreJweContentEncryptionAlgorithm, CoreProviderMetadata, CoreRevocableToken,
        CoreRevocationErrorResponse, CoreTokenIntrospectionResponse, CoreTokenResponse,
    },
};
use reqwest;

use crate::{errors::AuthError, models::CustomClaims};

// Create a stateful HTTP client that doesn't follow redirects (SSRF prevention)
fn create_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Failed to build HTTP client")
}

// The client type returned by from_provider_metadata has auth endpoint always set,
// device auth endpoint never set, and others as MaybeSet
type DiscoveredClient = openidconnect::Client<
    EmptyAdditionalClaims,
    CoreAuthDisplay,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJsonWebKey,
    CoreAuthPrompt,
    StandardErrorResponse<CoreErrorResponseType>,
    CoreTokenResponse,
    CoreTokenIntrospectionResponse,
    CoreRevocableToken,
    CoreRevocationErrorResponse,
    EndpointSet,      // HasAuthUrl - always set from metadata
    EndpointNotSet,   // HasDeviceAuthUrl - not from discovery
    EndpointNotSet,   // HasIntrospectionUrl - not from discovery
    EndpointNotSet,   // HasRevocationUrl - not from discovery
    EndpointMaybeSet, // HasTokenUrl - maybe from metadata
    EndpointMaybeSet, // HasUserInfoUrl - maybe from metadata
>;

pub struct OpenIDClient {
    client: DiscoveredClient,
    config: AuthConfig,
    http_client: reqwest::Client,
}

impl OpenIDClient {
    pub async fn new(config: AuthConfig) -> Result<Self, AuthError> {
        let issuer_url = IssuerUrl::new(config.issuer_url.clone())
            .map_err(|e| AuthError::Configuration(format!("Invalid issuer URL: {}", e)))?;

        let http_client = create_http_client();

        let provider_metadata =
            CoreProviderMetadata::discover_async(issuer_url.clone(), &http_client).await?;

        // Use the concrete type that from_provider_metadata returns
        let client = openidconnect::Client::from_provider_metadata(
            provider_metadata,
            ClientId::new(config.client_id.clone()),
            Some(ClientSecret::new(config.client_secret.clone())),
        );

        Ok(Self {
            client,
            config,
            http_client,
        })
    }

    // Authorization URL generation and code exchange are not needed for a pure resource server.

    pub async fn get_user_info(
        &self,
        access_token: openidconnect::AccessToken,
    ) -> Result<
        openidconnect::UserInfoClaims<CustomClaims, openidconnect::core::CoreGenderClaim>,
        AuthError,
    > {
        let userinfo_request = self.client.user_info(access_token, None).map_err(|e| {
            AuthError::Configuration(format!("UserInfo endpoint not available: {}", e))
        })?;

        let userinfo = userinfo_request
            .request_async(&self.http_client)
            .await
            .map_err(|e| AuthError::OpenIDConnect(e.to_string()))?;

        Ok(userinfo)
    }

    pub fn config(&self) -> &AuthConfig {
        &self.config
    }
}

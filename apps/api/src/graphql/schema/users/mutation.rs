use async_graphql::{Context, Object, Result};
use dokusho_auth::AuthenticationRequest;

use crate::graphql::{AuthGuard, GraphQLContext};

use super::types::InitiateAuthResponse;

#[derive(Default)]
pub struct UsersMutation;

#[Object(rename_fields = "snake_case")]
impl UsersMutation {
    async fn initiate_authentication(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "redirect_uri")] redirect_uri: String,
    ) -> Result<InitiateAuthResponse> {
        let context = ctx.data::<GraphQLContext>()?;

        // Get auth service from context
        let auth_service = &context.auth_service;

        // Initiate authentication
        let response = auth_service
            .initiate_authentication(AuthenticationRequest { redirect_uri })
            .await
            .map_err(|e| {
                async_graphql::Error::new(format!("Failed to initiate authentication: {}", e))
            })?;

        Ok(InitiateAuthResponse {
            authorization_url: response.authorization_url,
            state: response.state,
        })
    }

    /// Refresh the current user's authentication token
    /// Requires a valid JWT token in the Authorization header
    #[graphql(guard = "AuthGuard")]
    async fn refresh_token(&self, ctx: &Context<'_>) -> Result<String> {
        let context = ctx.data::<GraphQLContext>()?;

        // Get auth service from context
        let auth_service = &context.auth_service;

        // Get the current token from context (set in graphql_handler)
        let current_token = ctx.data::<String>()?;

        // Refresh token using the existing session rotation method
        auth_service
            .refresh_token(current_token)
            .await
            .map_err(|e| async_graphql::Error::new(format!("Failed to refresh token: {}", e)))
    }

    /// Logout the current user by invalidating their session
    /// Requires a valid JWT token in the Authorization header
    #[graphql(guard = "AuthGuard")]
    async fn logout(&self, ctx: &Context<'_>) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;

        // Get auth service from context
        let auth_service = &context.auth_service;

        // Get the current token from context (set in graphql_handler)
        let current_token = ctx.data::<String>()?;

        // Logout
        auth_service
            .logout(current_token)
            .await
            .map_err(|e| async_graphql::Error::new(format!("Failed to logout: {}", e)))?;

        Ok(true)
    }
}

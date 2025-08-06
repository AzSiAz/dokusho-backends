use async_graphql::{Context, Object, Result};
use dokusho_auth::{AuthConfig, AuthService, AuthenticationRequest};

use super::schema::GraphQLContext;

pub struct Mutation;

#[Object]
impl Mutation {
    async fn initiate_authentication(
        &self,
        ctx: &Context<'_>,
        redirect_uri: String,
    ) -> Result<InitiateAuthResponse> {
        let context = ctx.data::<GraphQLContext>()?;

        if !context.config.auth.enabled {
            return Err(async_graphql::Error::new("Authentication is not enabled"));
        }

        // Create auth config
        let auth_config = AuthConfig {
            enabled: context.config.auth.enabled,
            issuer_url: context.config.auth.issuer_url.clone().unwrap_or_default(),
            client_id: context.config.auth.client_id.clone().unwrap_or_default(),
            client_secret: context
                .config
                .auth
                .client_secret
                .clone()
                .unwrap_or_default(),
            redirect_url: context.config.auth.redirect_url.clone().unwrap_or_default(),
            jwt_secret: context.config.auth.jwt_secret.clone(),
            jwt_expiry_hours: context.config.auth.jwt_expiry_hours as i64,
        };

        // Create repositories
        use dokusho_database::repositories::{AuthStateRepository, UserRepository};
        let user_repo = UserRepository::new(context.database.pool().clone());
        let auth_state_repo = AuthStateRepository::new(context.database.pool().clone());

        // Create auth service
        let auth_service = AuthService::new(user_repo, auth_state_repo, auth_config)
            .await
            .map_err(|e| {
                async_graphql::Error::new(format!("Failed to create auth service: {}", e))
            })?;

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

    async fn refresh_token(&self, ctx: &Context<'_>, token: String) -> Result<String> {
        let context = ctx.data::<GraphQLContext>()?;

        if !context.config.auth.enabled {
            return Err(async_graphql::Error::new("Authentication is not enabled"));
        }

        // Create auth config
        let auth_config = AuthConfig {
            enabled: context.config.auth.enabled,
            issuer_url: context.config.auth.issuer_url.clone().unwrap_or_default(),
            client_id: context.config.auth.client_id.clone().unwrap_or_default(),
            client_secret: context
                .config
                .auth
                .client_secret
                .clone()
                .unwrap_or_default(),
            redirect_url: context.config.auth.redirect_url.clone().unwrap_or_default(),
            jwt_secret: context.config.auth.jwt_secret.clone(),
            jwt_expiry_hours: context.config.auth.jwt_expiry_hours as i64,
        };

        // Create repositories
        use dokusho_database::repositories::{AuthStateRepository, UserRepository};
        let user_repo = UserRepository::new(context.database.pool().clone());
        let auth_state_repo = AuthStateRepository::new(context.database.pool().clone());

        // Create auth service
        let auth_service = AuthService::new(user_repo, auth_state_repo, auth_config)
            .await
            .map_err(|e| {
                async_graphql::Error::new(format!("Failed to create auth service: {}", e))
            })?;

        // Refresh token
        auth_service
            .refresh_token(&token)
            .await
            .map_err(|e| async_graphql::Error::new(format!("Failed to refresh token: {}", e)))
    }

    async fn logout(&self, ctx: &Context<'_>, token: String) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;

        if !context.config.auth.enabled {
            return Err(async_graphql::Error::new("Authentication is not enabled"));
        }

        // Create auth config
        let auth_config = AuthConfig {
            enabled: context.config.auth.enabled,
            issuer_url: context.config.auth.issuer_url.clone().unwrap_or_default(),
            client_id: context.config.auth.client_id.clone().unwrap_or_default(),
            client_secret: context
                .config
                .auth
                .client_secret
                .clone()
                .unwrap_or_default(),
            redirect_url: context.config.auth.redirect_url.clone().unwrap_or_default(),
            jwt_secret: context.config.auth.jwt_secret.clone(),
            jwt_expiry_hours: context.config.auth.jwt_expiry_hours as i64,
        };

        // Create repositories
        use dokusho_database::repositories::{AuthStateRepository, UserRepository};
        let user_repo = UserRepository::new(context.database.pool().clone());
        let auth_state_repo = AuthStateRepository::new(context.database.pool().clone());

        // Create auth service
        let auth_service = AuthService::new(user_repo, auth_state_repo, auth_config)
            .await
            .map_err(|e| {
                async_graphql::Error::new(format!("Failed to create auth service: {}", e))
            })?;

        // Logout
        auth_service
            .logout(&token)
            .await
            .map_err(|e| async_graphql::Error::new(format!("Failed to logout: {}", e)))?;

        Ok(true)
    }
}

#[derive(async_graphql::SimpleObject)]
struct InitiateAuthResponse {
    authorization_url: String,
    state: String,
}

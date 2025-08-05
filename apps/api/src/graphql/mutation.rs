use async_graphql::{Context, Object, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};

use super::schema::GraphQLContext;
use crate::auth::{Claims, generate_auth_url};

pub struct Mutation;

#[Object]
impl Mutation {
    async fn generate_auth_url(&self, ctx: &Context<'_>) -> Result<String> {
        let context = ctx.data::<GraphQLContext>()?;
        
        if !context.config.auth.enabled {
            return Err(async_graphql::Error::new("Authentication is not enabled"));
        }

        generate_auth_url(&context.config)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    async fn refresh_token(&self, ctx: &Context<'_>) -> Result<String> {
        let context = ctx.data::<GraphQLContext>()?;
        let claims = ctx.data::<Claims>()
            .map_err(|_| async_graphql::Error::new("Unauthorized"))?;

        // Create new claims with extended expiration
        let new_claims = Claims {
            sub: claims.sub.clone(),
            email: claims.email.clone(),
            name: claims.name.clone(),
            exp: (Utc::now() + Duration::hours(context.config.auth.jwt_expiry_hours as i64)).timestamp() as usize,
            iat: Utc::now().timestamp() as usize,
        };

        // Generate new token
        let token = encode(
            &Header::default(),
            &new_claims,
            &EncodingKey::from_secret(context.config.auth.jwt_secret.as_bytes()),
        )
        .map_err(|e| async_graphql::Error::new(format!("Failed to generate token: {}", e)))?;

        Ok(token)
    }
}
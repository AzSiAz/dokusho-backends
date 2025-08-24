use async_graphql::{Context, Object, Result};
use dokusho_auth::models::Claims;

use crate::graphql::{AdminGuard, AuthGuard, GraphQLContext};

use super::types::User;

#[derive(Default)]
pub struct UsersQuery;

#[Object(rename_fields = "snake_case")]
impl UsersQuery {
    /// Get the current authenticated user's information
    #[graphql(guard = "AuthGuard")]
    async fn me(&self, ctx: &Context<'_>) -> Result<User> {
        let context = ctx.data::<GraphQLContext>()?;
        let claims = ctx.data::<Claims>()?;

        let user = context.database.users()
            .find_by_id(claims.user_id)
            .await?
            .ok_or_else(|| async_graphql::Error::new("User not found"))?;

        Ok(user.into())
    }

    /// Get all users (admin only)
    #[graphql(guard = "AdminGuard")]
    async fn users(&self, ctx: &Context<'_>) -> Result<Vec<User>> {
        let context = ctx.data::<GraphQLContext>()?;

        let users = context.database.users().find_all().await?;

        Ok(users.into_iter().map(Into::into).collect())
    }
}

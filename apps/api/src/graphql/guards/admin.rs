use async_graphql::{Context, Error, Guard, Result};
use dokusho_auth::models::Claims;
use dokusho_database::models::user::UserRole;

pub struct AdminGuard;

impl Guard for AdminGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        let claims = ctx
            .data_opt::<Claims>()
            .ok_or_else(|| Error::new("Authentication required"))?;

        if claims.role != UserRole::Admin {
            return Err(Error::new("Admin role required"));
        }

        Ok(())
    }
}

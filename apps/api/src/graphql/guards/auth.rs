use async_graphql::{Context, Error, Guard, Result};
use dokusho_auth::models::Claims;

pub struct AuthGuard;

impl Guard for AuthGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        ctx.data_opt::<Claims>()
            .ok_or_else(|| Error::new("Authentication required"))?;
        Ok(())
    }
}

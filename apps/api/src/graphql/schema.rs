use async_graphql::{Context, EmptySubscription, Schema};
use dokusho_scrapers::SourceRegistry;
use std::sync::Arc;

use super::{mutation::Mutation, query::Query};
use crate::auth::Claims;

pub type AppSchema = Schema<Query, Mutation, EmptySubscription>;

#[derive(Clone)]
pub struct GraphQLContext {
    pub sources: Arc<SourceRegistry>,
    pub config: crate::config::AppConfig,
}

pub fn build_schema(sources: Arc<SourceRegistry>, config: crate::config::AppConfig) -> AppSchema {
    Schema::build(Query, Mutation, EmptySubscription)
        .data(GraphQLContext { sources, config })
        .finish()
}

pub trait AuthenticatedContext {
    fn get_claims(&self) -> Result<&Claims, async_graphql::Error>;
}

impl AuthenticatedContext for Context<'_> {
    fn get_claims(&self) -> Result<&Claims, async_graphql::Error> {
        self.data::<Claims>()
            .map_err(|_| async_graphql::Error::new("Unauthorized"))
    }
}
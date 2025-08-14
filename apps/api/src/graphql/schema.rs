use async_graphql::{EmptySubscription, Schema};
use dokusho_database::Database;
use sources::SourceRegistry;
use std::sync::Arc;

use super::{mutation::Mutation, query::Query};

pub type AppSchema = Schema<Query, Mutation, EmptySubscription>;

#[derive(Clone)]
pub struct GraphQLContext {
    pub sources: Arc<SourceRegistry>,
    pub config: crate::config::AppConfig,
    pub database: Arc<Database>,
}

pub fn build_schema(
    sources: Arc<SourceRegistry>,
    config: crate::config::AppConfig,
    database: Arc<Database>,
) -> AppSchema {
    Schema::build(Query, Mutation, EmptySubscription)
        .data(GraphQLContext {
            sources,
            config,
            database,
        })
        .finish()
}

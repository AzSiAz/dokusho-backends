use async_graphql::{EmptySubscription, Schema};
use dokusho_auth::AuthService;
use dokusho_database::Database;
use sources::SourceRegistry;
use std::sync::Arc;

use super::{mutation::Mutation, query::Query};

pub type AppSchema = Schema<Query, Mutation, EmptySubscription>;

#[derive(Clone)]
pub struct GraphQLContext {
    // pub sources: Arc<SourceRegistry>,
    pub database: Arc<Database>,
    pub auth_service: Arc<AuthService>,
}

pub fn build_schema(
    _sources: Arc<SourceRegistry>,
    _config: dokusho_config::AppConfig,
    database: Arc<Database>,
    auth_service: Arc<AuthService>,
) -> AppSchema {
    Schema::build(Query, Mutation, EmptySubscription)
        .data(GraphQLContext {
            // sources,
            database,
            auth_service,
        })
        .finish()
}

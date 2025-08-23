use async_graphql::{EmptySubscription, Schema};
use dokusho_auth::AuthService;
use dokusho_database::Database;
use sources::SourceRegistry;
use std::sync::Arc;

use async_graphql::MergedObject;

use crate::graphql::schema::{
    health::HealthQuery,
    sources::SourcesQuery,
    users::{UsersMutation, UsersQuery},
};

#[derive(MergedObject, Default)]
pub struct Query(HealthQuery, UsersQuery, SourcesQuery);

#[derive(MergedObject, Default)]
pub struct Mutation(UsersMutation);

pub type AppSchema = Schema<Query, Mutation, EmptySubscription>;

#[derive(Clone)]
pub struct GraphQLContext {
    pub sources: Arc<SourceRegistry>,
    pub database: Arc<Database>,
    pub auth_service: Arc<AuthService>,
}

pub fn build_schema(
    sources: Arc<SourceRegistry>,
    _config: dokusho_config::AppConfig,
    database: Arc<Database>,
    auth_service: Arc<AuthService>,
) -> AppSchema {
    Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .data(GraphQLContext {
            sources,
            database,
            auth_service,
        })
        .finish()
}

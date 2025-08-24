pub use sea_orm_migration::prelude::*;

mod m20250824_165701_create_table;
mod m20250824_165728_add_indexes;
mod m20250824_165811_add_user_roles;
mod m20250824_165830_add_pkce_verifier;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250824_165701_create_table::Migration),
            Box::new(m20250824_165728_add_indexes::Migration),
            Box::new(m20250824_165811_add_user_roles::Migration),
            Box::new(m20250824_165830_add_pkce_verifier::Migration),
        ]
    }
}

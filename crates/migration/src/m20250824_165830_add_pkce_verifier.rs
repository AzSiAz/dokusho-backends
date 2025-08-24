use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add PKCE verifier to auth_states table
        manager
            .alter_table(
                Table::alter()
                    .table(AuthState::Table)
                    .add_column(ColumnDef::new(AuthState::PkceVerifier).text())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop PKCE verifier column
        manager
            .alter_table(
                Table::alter()
                    .table(AuthState::Table)
                    .drop_column(AuthState::PkceVerifier)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum AuthState {
    Table,
    PkceVerifier,
}

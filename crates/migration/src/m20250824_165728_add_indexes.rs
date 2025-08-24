use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Auth states indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_auth_states_expires")
                    .table(AuthState::Table)
                    .col(AuthState::ExpiresAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_auth_states_created")
                    .table(AuthState::Table)
                    .col(AuthState::CreatedAt)
                    .to_owned(),
            )
            .await?;

        // Users indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_users_sub")
                    .table(User::Table)
                    .col(User::Sub)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_users_email")
                    .table(User::Table)
                    .col(User::Email)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_users_created")
                    .table(User::Table)
                    .col(User::CreatedAt)
                    .to_owned(),
            )
            .await?;

        // User sessions indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_user_sessions_user_id")
                    .table(UserSession::Table)
                    .col(UserSession::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_user_sessions_expires")
                    .table(UserSession::Table)
                    .col(UserSession::ExpiresAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_user_sessions_token")
                    .table(UserSession::Table)
                    .col(UserSession::TokenHash)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop indexes in reverse order
        manager
            .drop_index(
                Index::drop()
                    .name("idx_user_sessions_token")
                    .table(UserSession::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_user_sessions_expires")
                    .table(UserSession::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_user_sessions_user_id")
                    .table(UserSession::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_users_created")
                    .table(User::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_users_email")
                    .table(User::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_users_sub")
                    .table(User::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_auth_states_created")
                    .table(AuthState::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_auth_states_expires")
                    .table(AuthState::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum AuthState {
    Table,
    ExpiresAt,
    CreatedAt,
}

#[derive(DeriveIden)]
enum User {
    Table,
    Sub,
    Email,
    CreatedAt,
}

#[derive(DeriveIden)]
enum UserSession {
    Table,
    UserId,
    ExpiresAt,
    TokenHash,
}

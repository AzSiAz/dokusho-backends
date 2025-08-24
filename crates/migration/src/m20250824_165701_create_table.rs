use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create auth_states table
        manager
            .create_table(
                Table::create()
                    .table(AuthState::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AuthState::State)
                            .string_len(255)
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AuthState::RedirectUri).text().not_null())
                    .col(ColumnDef::new(AuthState::Nonce).string_len(255).not_null())
                    .col(
                        ColumnDef::new(AuthState::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(AuthState::ExpiresAt)
                            .timestamp_with_time_zone()
                            .default(
                                Expr::current_timestamp().add(Expr::cust("INTERVAL '10 minutes'")),
                            ),
                    )
                    .to_owned(),
            )
            .await?;

        // Create users table
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(User::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("gen_random_uuid()"))
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(User::Sub)
                            .string_len(255)
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(User::Email).string_len(255))
                    .col(ColumnDef::new(User::Name).string_len(255))
                    .col(
                        ColumnDef::new(User::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(User::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create user_sessions table
        manager
            .create_table(
                Table::create()
                    .table(UserSession::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserSession::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("gen_random_uuid()"))
                            .primary_key(),
                    )
                    .col(ColumnDef::new(UserSession::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(UserSession::TokenHash)
                            .string_len(255)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(UserSession::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserSession::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(UserSession::LastUsedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(UserSession::Table, UserSession::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create user_preferences table
        manager
            .create_table(
                Table::create()
                    .table(UserPreference::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserPreference::UserId)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(UserPreference::PreferredLanguage)
                            .string_len(10)
                            .default("en"),
                    )
                    .col(
                        ColumnDef::new(UserPreference::Theme)
                            .string_len(20)
                            .default("light"),
                    )
                    .col(
                        ColumnDef::new(UserPreference::NotificationsEnabled)
                            .boolean()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(UserPreference::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(UserPreference::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(UserPreference::Table, UserPreference::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create update trigger function
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE OR REPLACE FUNCTION update_updated_at_column()
                RETURNS TRIGGER AS $$
                BEGIN
                    NEW.updated_at = NOW();
                    RETURN NEW;
                END;
                $$ LANGUAGE plpgsql;
                "#,
            )
            .await?;

        // Add triggers (DROP IF EXISTS first to avoid conflicts)
        manager
            .get_connection()
            .execute_unprepared(
                "DROP TRIGGER IF EXISTS update_users_updated_at ON \"user\";
                CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON \"user\"
                    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "DROP TRIGGER IF EXISTS update_user_preferences_updated_at ON \"user_preference\";
                CREATE TRIGGER update_user_preferences_updated_at BEFORE UPDATE ON \"user_preference\"
                    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop triggers
        manager
            .get_connection()
            .execute_unprepared("DROP TRIGGER IF EXISTS update_users_updated_at ON \"user\";")
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "DROP TRIGGER IF EXISTS update_user_preferences_updated_at ON \"user_preference\";",
            )
            .await?;

        // Drop function
        manager
            .get_connection()
            .execute_unprepared("DROP FUNCTION IF EXISTS update_updated_at_column();")
            .await?;

        // Drop tables
        manager
            .drop_table(
                Table::drop()
                    .table(UserPreference::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(UserSession::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(User::Table).if_exists().to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(AuthState::Table).if_exists().to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum AuthState {
    Table,
    State,
    RedirectUri,
    Nonce,
    CreatedAt,
    ExpiresAt,
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
    Sub,
    Email,
    Name,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum UserSession {
    Table,
    Id,
    UserId,
    TokenHash,
    ExpiresAt,
    CreatedAt,
    LastUsedAt,
}

#[derive(DeriveIden)]
enum UserPreference {
    Table,
    UserId,
    PreferredLanguage,
    Theme,
    NotificationsEnabled,
    CreatedAt,
    UpdatedAt,
}

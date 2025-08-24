use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create user_role enum type
        manager
            .get_connection()
            .execute_unprepared("CREATE TYPE user_role AS ENUM ('user', 'admin');")
            .await?;

        // Add role column to users table
        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .add_column(
                        ColumnDef::new(User::Role)
                            .custom(Alias::new("user_role"))
                            .not_null()
                            .default(Expr::cust("'user'")),
                    )
                    .to_owned(),
            )
            .await?;

        // Add index for role queries
        manager
            .create_index(
                Index::create()
                    .name("idx_users_role")
                    .table(User::Table)
                    .col(User::Role)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop index
        manager
            .drop_index(
                Index::drop()
                    .name("idx_users_role")
                    .table(User::Table)
                    .to_owned(),
            )
            .await?;

        // Drop role column
        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .drop_column(User::Role)
                    .to_owned(),
            )
            .await?;

        // Drop enum type
        manager
            .get_connection()
            .execute_unprepared("DROP TYPE IF EXISTS user_role;")
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum User {
    Table,
    Role,
}

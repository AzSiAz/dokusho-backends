use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create sources lookup table
        manager
            .create_table(
                Table::create()
                    .table(Sources::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Sources::Id)
                            .string_len(100)
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Sources::Name).string_len(100).not_null())
                    .col(ColumnDef::new(Sources::Url).text())
                    .col(
                        ColumnDef::new(Sources::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Sources::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create serie_sources junction table
        manager
            .create_table(
                Table::create()
                    .table(SerieSources::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(SerieSources::SerieId).uuid().not_null())
                    .col(
                        ColumnDef::new(SerieSources::SourceId)
                            .string_len(100)
                            .not_null(),
                    )
                    .col(ColumnDef::new(SerieSources::ExternalId).text())
                    .col(ColumnDef::new(SerieSources::Url).text())
                    .col(
                        ColumnDef::new(SerieSources::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .primary_key(
                        Index::create()
                            .col(SerieSources::SerieId)
                            .col(SerieSources::SourceId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_serie_sources_serie")
                            .from(SerieSources::Table, SerieSources::SerieId)
                            .to(Series::Table, Series::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_serie_sources_source")
                            .from(SerieSources::Table, SerieSources::SourceId)
                            .to(Sources::Table, Sources::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index for serie_sources to improve lookups by source
        manager
            .create_index(
                Index::create()
                    .name("idx_serie_sources_source")
                    .table(SerieSources::Table)
                    .col(SerieSources::SourceId)
                    .to_owned(),
            )
            .await?;

        // Create index for serie_sources to improve lookups by external_id
        manager
            .create_index(
                Index::create()
                    .name("idx_serie_sources_external_id")
                    .table(SerieSources::Table)
                    .col(SerieSources::SourceId)
                    .col(SerieSources::ExternalId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop junction table first (has foreign keys)
        manager
            .drop_table(Table::drop().table(SerieSources::Table).to_owned())
            .await?;

        // Drop sources table
        manager
            .drop_table(Table::drop().table(Sources::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Series {
    Table,
    Id,
}

#[derive(Iden)]
enum Sources {
    Table,
    Id,
    Name,
    Url,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum SerieSources {
    Table,
    SerieId,
    SourceId,
    ExternalId,
    Url,
    CreatedAt,
}

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create series table
        manager
            .create_table(
                Table::create()
                    .table(Series::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Series::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("gen_random_uuid()"))
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Series::CoverUrl).text().not_null())
                    .col(ColumnDef::new(Series::SerieTypeId).uuid().not_null())
                    .col(
                        ColumnDef::new(Series::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Series::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create serie_types lookup table
        manager
            .create_table(
                Table::create()
                    .table(SerieTypes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SerieTypes::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("gen_random_uuid()"))
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SerieTypes::SerieType)
                            .string_len(100)
                            .not_null()
                            .unique_key(),
                    )
                    .to_owned(),
            )
            .await?;

        // Add foreign key constraint from Series to SerieTypes
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_series_serie_type")
                    .from(Series::Table, Series::SerieTypeId)
                    .to(SerieTypes::Table, SerieTypes::Id)
                    .on_delete(ForeignKeyAction::Restrict)
                    .to_owned(),
            )
            .await?;

        // Create serie_titles table
        manager
            .create_table(
                Table::create()
                    .table(SerieTitles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SerieTitles::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("gen_random_uuid()"))
                            .primary_key(),
                    )
                    .col(ColumnDef::new(SerieTitles::SerieId).uuid().not_null())
                    .col(
                        ColumnDef::new(SerieTitles::Language)
                            .string_len(10)
                            .not_null(),
                    )
                    .col(ColumnDef::new(SerieTitles::Title).text().not_null())
                    .col(
                        ColumnDef::new(SerieTitles::IsAlternate)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_serie_titles_serie")
                            .from(SerieTitles::Table, SerieTitles::SerieId)
                            .to(Series::Table, Series::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index for serie_titles
        manager
            .create_index(
                Index::create()
                    .name("idx_serie_titles_serie_lang")
                    .table(SerieTitles::Table)
                    .col(SerieTitles::SerieId)
                    .col(SerieTitles::Language)
                    .col(SerieTitles::IsAlternate)
                    .to_owned(),
            )
            .await?;

        // Create unique constraint for serie_titles
        manager
            .create_index(
                Index::create()
                    .name("uniq_serie_titles")
                    .table(SerieTitles::Table)
                    .col(SerieTitles::SerieId)
                    .col(SerieTitles::Language)
                    .col(SerieTitles::Title)
                    .col(SerieTitles::IsAlternate)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Create serie_synopsis table
        manager
            .create_table(
                Table::create()
                    .table(SerieSynopsis::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SerieSynopsis::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("gen_random_uuid()"))
                            .primary_key(),
                    )
                    .col(ColumnDef::new(SerieSynopsis::SerieId).uuid().not_null())
                    .col(
                        ColumnDef::new(SerieSynopsis::Language)
                            .string_len(10)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SerieSynopsis::Synopsis)
                            .array(ColumnType::Text)
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_serie_synopsis_serie")
                            .from(SerieSynopsis::Table, SerieSynopsis::SerieId)
                            .to(Series::Table, Series::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index for serie_synopsis
        manager
            .create_index(
                Index::create()
                    .name("idx_serie_synopsis_serie_lang")
                    .table(SerieSynopsis::Table)
                    .col(SerieSynopsis::SerieId)
                    .col(SerieSynopsis::Language)
                    .to_owned(),
            )
            .await?;

        // Create unique constraint for serie_synopsis
        manager
            .create_index(
                Index::create()
                    .name("uniq_serie_synopsis")
                    .table(SerieSynopsis::Table)
                    .col(SerieSynopsis::SerieId)
                    .col(SerieSynopsis::Language)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Create statuses lookup table
        manager
            .create_table(
                Table::create()
                    .table(Statuses::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Statuses::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("gen_random_uuid()"))
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Statuses::Status)
                            .string_len(50)
                            .not_null()
                            .unique_key(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create genres lookup table
        manager
            .create_table(
                Table::create()
                    .table(Genres::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Genres::Id)
                            .uuid()
                            .default(Expr::cust("gen_random_uuid()"))
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Genres::Genre)
                            .string_len(100)
                            .not_null()
                            .unique_key(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create authors lookup table
        manager
            .create_table(
                Table::create()
                    .table(Authors::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Authors::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("gen_random_uuid()"))
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Authors::Name).text().not_null().unique_key())
                    .to_owned(),
            )
            .await?;

        // Create artists lookup table
        manager
            .create_table(
                Table::create()
                    .table(Artists::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Artists::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("gen_random_uuid()"))
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Artists::Name).text().not_null().unique_key())
                    .to_owned(),
            )
            .await?;

        // Create serie_status junction table
        manager
            .create_table(
                Table::create()
                    .table(SerieStatus::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(SerieStatus::SerieId).uuid().not_null())
                    .col(ColumnDef::new(SerieStatus::StatusId).uuid().not_null())
                    .primary_key(
                        Index::create()
                            .col(SerieStatus::SerieId)
                            .col(SerieStatus::StatusId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_serie_status_serie")
                            .from(SerieStatus::Table, SerieStatus::SerieId)
                            .to(Series::Table, Series::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_serie_status_status")
                            .from(SerieStatus::Table, SerieStatus::StatusId)
                            .to(Statuses::Table, Statuses::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create serie_genres junction table
        manager
            .create_table(
                Table::create()
                    .table(SerieGenres::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(SerieGenres::SerieId).uuid().not_null())
                    .col(ColumnDef::new(SerieGenres::GenreId).uuid().not_null())
                    .primary_key(
                        Index::create()
                            .col(SerieGenres::SerieId)
                            .col(SerieGenres::GenreId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_serie_genres_serie")
                            .from(SerieGenres::Table, SerieGenres::SerieId)
                            .to(Series::Table, Series::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_serie_genres_genre")
                            .from(SerieGenres::Table, SerieGenres::GenreId)
                            .to(Genres::Table, Genres::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create serie_authors junction table
        manager
            .create_table(
                Table::create()
                    .table(SerieAuthors::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(SerieAuthors::SerieId).uuid().not_null())
                    .col(ColumnDef::new(SerieAuthors::AuthorId).uuid().not_null())
                    .primary_key(
                        Index::create()
                            .col(SerieAuthors::SerieId)
                            .col(SerieAuthors::AuthorId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_serie_authors_serie")
                            .from(SerieAuthors::Table, SerieAuthors::SerieId)
                            .to(Series::Table, Series::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_serie_authors_author")
                            .from(SerieAuthors::Table, SerieAuthors::AuthorId)
                            .to(Authors::Table, Authors::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create serie_artists junction table
        manager
            .create_table(
                Table::create()
                    .table(SerieArtists::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(SerieArtists::SerieId).uuid().not_null())
                    .col(ColumnDef::new(SerieArtists::ArtistId).uuid().not_null())
                    .primary_key(
                        Index::create()
                            .col(SerieArtists::SerieId)
                            .col(SerieArtists::ArtistId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_serie_artists_serie")
                            .from(SerieArtists::Table, SerieArtists::SerieId)
                            .to(Series::Table, Series::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_serie_artists_artist")
                            .from(SerieArtists::Table, SerieArtists::ArtistId)
                            .to(Artists::Table, Artists::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop junction tables first (they have foreign keys)
        manager
            .drop_table(Table::drop().table(SerieArtists::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(SerieAuthors::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(SerieGenres::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(SerieStatus::Table).to_owned())
            .await?;

        // Drop dependent tables
        manager
            .drop_table(Table::drop().table(SerieSynopsis::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(SerieTitles::Table).to_owned())
            .await?;

        // Drop main table (must be before lookup tables due to foreign keys)
        manager
            .drop_table(Table::drop().table(Series::Table).to_owned())
            .await?;

        // Drop lookup tables last
        manager
            .drop_table(Table::drop().table(SerieTypes::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Artists::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Authors::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Genres::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Statuses::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Series {
    Table,
    Id,
    CoverUrl,
    SerieTypeId,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum SerieTitles {
    Table,
    Id,
    SerieId,
    Language,
    Title,
    IsAlternate,
}

#[derive(Iden)]
enum SerieSynopsis {
    Table,
    Id,
    SerieId,
    Language,
    Synopsis,
}

#[derive(Iden)]
enum Statuses {
    Table,
    Id,
    Status,
}

#[derive(Iden)]
enum Genres {
    Table,
    Id,
    Genre,
}

#[derive(Iden)]
enum Authors {
    Table,
    Id,
    Name,
}

#[derive(Iden)]
enum Artists {
    Table,
    Id,
    Name,
}

#[derive(Iden)]
enum SerieTypes {
    Table,
    Id,
    SerieType,
}

#[derive(Iden)]
enum SerieStatus {
    Table,
    SerieId,
    StatusId,
}

#[derive(Iden)]
enum SerieGenres {
    Table,
    SerieId,
    GenreId,
}

#[derive(Iden)]
enum SerieAuthors {
    Table,
    SerieId,
    AuthorId,
}

#[derive(Iden)]
enum SerieArtists {
    Table,
    SerieId,
    ArtistId,
}

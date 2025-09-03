-- Create serie_types lookup table
CREATE TABLE IF NOT EXISTS serie_types (
    id UUID NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    serie_type VARCHAR(100) NOT NULL UNIQUE
);

-- Create series table
CREATE TABLE IF NOT EXISTS series (
    id UUID NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    cover_url TEXT NOT NULL,
    serie_type_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_series_serie_type FOREIGN KEY (serie_type_id) REFERENCES serie_types(id) ON DELETE RESTRICT
);

-- Create serie_titles table
CREATE TABLE IF NOT EXISTS serie_titles (
    id UUID NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    serie_id UUID NOT NULL,
    language VARCHAR(10) NOT NULL,
    title TEXT NOT NULL,
    is_alternate BOOLEAN NOT NULL DEFAULT false,
    CONSTRAINT fk_serie_titles_serie FOREIGN KEY (serie_id) REFERENCES series(id) ON DELETE CASCADE
);

-- Create indexes for serie_titles
CREATE INDEX IF NOT EXISTS idx_serie_titles_serie_lang ON serie_titles(serie_id, language, is_alternate);
CREATE UNIQUE INDEX IF NOT EXISTS uniq_serie_titles ON serie_titles(serie_id, language, title, is_alternate);

-- Create serie_synopsis table
CREATE TABLE IF NOT EXISTS serie_synopsis (
    id UUID NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    serie_id UUID NOT NULL,
    language VARCHAR(10) NOT NULL,
    synopsis TEXT[] NOT NULL,
    CONSTRAINT fk_serie_synopsis_serie FOREIGN KEY (serie_id) REFERENCES series(id) ON DELETE CASCADE
);

-- Create indexes for serie_synopsis
CREATE INDEX IF NOT EXISTS idx_serie_synopsis_serie_lang ON serie_synopsis(serie_id, language);
CREATE UNIQUE INDEX IF NOT EXISTS uniq_serie_synopsis ON serie_synopsis(serie_id, language);

-- Create statuses lookup table
CREATE TABLE IF NOT EXISTS statuses (
    id UUID NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    status VARCHAR(50) NOT NULL UNIQUE
);

-- Create genres lookup table
CREATE TABLE IF NOT EXISTS genres (
    id UUID DEFAULT gen_random_uuid() NOT NULL PRIMARY KEY,
    genre VARCHAR(100) NOT NULL UNIQUE
);

-- Create authors lookup table
CREATE TABLE IF NOT EXISTS authors (
    id UUID NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

-- Create artists lookup table
CREATE TABLE IF NOT EXISTS artists (
    id UUID NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

-- Create serie_status junction table
CREATE TABLE IF NOT EXISTS serie_status (
    serie_id UUID NOT NULL,
    status_id UUID NOT NULL,
    PRIMARY KEY (serie_id, status_id),
    CONSTRAINT fk_serie_status_serie FOREIGN KEY (serie_id) REFERENCES series(id) ON DELETE CASCADE,
    CONSTRAINT fk_serie_status_status FOREIGN KEY (status_id) REFERENCES statuses(id) ON DELETE CASCADE
);

-- Create serie_genres junction table
CREATE TABLE IF NOT EXISTS serie_genres (
    serie_id UUID NOT NULL,
    genre_id UUID NOT NULL,
    PRIMARY KEY (serie_id, genre_id),
    CONSTRAINT fk_serie_genres_serie FOREIGN KEY (serie_id) REFERENCES series(id) ON DELETE CASCADE,
    CONSTRAINT fk_serie_genres_genre FOREIGN KEY (genre_id) REFERENCES genres(id) ON DELETE CASCADE
);

-- Create serie_authors junction table
CREATE TABLE IF NOT EXISTS serie_authors (
    serie_id UUID NOT NULL,
    author_id UUID NOT NULL,
    PRIMARY KEY (serie_id, author_id),
    CONSTRAINT fk_serie_authors_serie FOREIGN KEY (serie_id) REFERENCES series(id) ON DELETE CASCADE,
    CONSTRAINT fk_serie_authors_author FOREIGN KEY (author_id) REFERENCES authors(id) ON DELETE CASCADE
);

-- Create serie_artists junction table
CREATE TABLE IF NOT EXISTS serie_artists (
    serie_id UUID NOT NULL,
    artist_id UUID NOT NULL,
    PRIMARY KEY (serie_id, artist_id),
    CONSTRAINT fk_serie_artists_serie FOREIGN KEY (serie_id) REFERENCES series(id) ON DELETE CASCADE,
    CONSTRAINT fk_serie_artists_artist FOREIGN KEY (artist_id) REFERENCES artists(id) ON DELETE CASCADE
);
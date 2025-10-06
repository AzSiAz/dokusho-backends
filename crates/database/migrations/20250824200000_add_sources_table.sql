-- Create sources lookup table
CREATE TABLE IF NOT EXISTS sources (
    id VARCHAR(100) NOT NULL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create serie_sources junction table
CREATE TABLE IF NOT EXISTS serie_sources (
    serie_id UUID NOT NULL,
    source_id VARCHAR(100) NOT NULL,
    external_id TEXT,
    url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (serie_id, source_id),
    CONSTRAINT fk_serie_sources_serie FOREIGN KEY (serie_id) REFERENCES series(id) ON DELETE CASCADE,
    CONSTRAINT fk_serie_sources_source FOREIGN KEY (source_id) REFERENCES sources(id) ON DELETE CASCADE
);

-- Create indexes for serie_sources
CREATE INDEX IF NOT EXISTS idx_serie_sources_source ON serie_sources(source_id);
CREATE INDEX IF NOT EXISTS idx_serie_sources_external_id ON serie_sources(source_id, external_id);
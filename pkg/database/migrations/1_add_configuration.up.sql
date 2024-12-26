CREATE SCHEMA IF NOT EXISTS jobs;

CREATE TABLE configuration (
	key text,
	value jsonb,
	CONSTRAINT unique_key UNIQUE (key)
);

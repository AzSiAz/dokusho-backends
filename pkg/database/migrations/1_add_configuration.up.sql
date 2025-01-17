CREATE TABLE configuration (
	key text,
	value jsonb,
	CONSTRAINT unique_key UNIQUE (key)
);

CREATE TABLE functions (
  id SERIAL PRIMARY KEY,
  deployment_id INTEGER NOT NULL REFERENCES deployments(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  kind TEXT NOT NULL CHECK (kind IN ('plugin', 'aws', 'kn')),
  location TEXT NOT NULL,
  hash TEXT,
  blob BYTEA,
  UNIQUE (deployment_id, name)
);
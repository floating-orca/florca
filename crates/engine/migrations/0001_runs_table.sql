CREATE TABLE runs (
  id SERIAL PRIMARY KEY,
  deployment_name TEXT NOT NULL,
  entry_point TEXT NOT NULL,
  input JSONB NOT NULL,
  output JSONB,
  start_time TIMESTAMP WITH TIME ZONE NOT NULL,
  end_time TIMESTAMP WITH TIME ZONE,
  success BOOLEAN
);
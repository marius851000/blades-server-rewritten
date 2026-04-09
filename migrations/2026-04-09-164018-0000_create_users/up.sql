-- Your SQL goes here
CREATE TABLE users (
    id UUID PRIMARY KEY,
    secret_id UUID UNIQUE NOT NULL,
    data JSONB NOT NULL
)

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TYPE priority AS ENUM ('Low', 'Regular', 'Urgent');

CREATE TABLE IF NOT EXISTS tasks (
    id SERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    priority priority NOT NULL,
    completed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

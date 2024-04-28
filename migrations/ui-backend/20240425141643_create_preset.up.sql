-- Add up migration script here

CREATE TABLE preset (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    info TEXT NOT NULL
);

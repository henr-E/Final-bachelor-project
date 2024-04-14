-- Add up migration script here
ALTER TABLE twins ADD COLUMN creation_date_time INTEGER NOT NULL DEFAULT 0;

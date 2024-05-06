-- Add up migration script here

ALTER TABLE preset ADD COLUMN is_edge BOOL NOT NULL DEFAULT FALSE;

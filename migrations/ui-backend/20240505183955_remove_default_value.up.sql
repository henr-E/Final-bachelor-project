-- Add up migration script here

ALTER TABLE preset DROP COLUMN is_edge;

ALTER TABLE preset ADD COLUMN is_edge BOOL NOT NULL;

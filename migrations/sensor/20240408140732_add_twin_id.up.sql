-- Add up migration script here
ALTER TABLE sensors ADD COLUMN twin_id INTEGER NOT NULL;

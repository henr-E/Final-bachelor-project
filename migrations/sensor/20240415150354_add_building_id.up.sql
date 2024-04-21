-- Add up migration script here
ALTER TABLE sensors ADD COLUMN building_id INTEGER;

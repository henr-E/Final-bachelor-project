-- Add up migration script here
ALTER TABLE simulations ADD COLUMN parent_id INTEGER DEFAULT NULL;
ALTER TABLE simulations ADD COLUMN parent_frame INTEGER DEFAULT NULl;
AlTER TABLE simulations ADD CONSTRAINT fk_parent_id FOREIGN KEY(parent_id) REFERENCES simulations(id) ON DELETE SET NULL;

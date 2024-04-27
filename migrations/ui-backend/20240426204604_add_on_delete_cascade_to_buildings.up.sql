-- Add up migration script here
ALTER TABLE buildings DROP CONSTRAINT buildings_twin_id_fkey;

ALTER TABLE buildings
    ADD CONSTRAINT buildings_twin_id_fkey FOREIGN KEY (twin_id) REFERENCES twins(id) ON DELETE CASCADE;

-- Add up migration script here
ALTER TABLE sensor_signals
    ADD CONSTRAINT unique_quantity_alias_per_sensor
        UNIQUE (sensor_id, alias);

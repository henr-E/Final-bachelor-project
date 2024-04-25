-- Add down migration script here
ALTER TABLE sensor_signals
    DROP CONSTRAINT unique_quantity_alias_per_sensor;

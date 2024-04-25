-- Add down migration script here
ALTER TABLE sensor_signals
DROP CONSTRAINT IF EXISTS unique_quantity_per_sensor;

DROP INDEX IF EXISTS unique_sensor_per_building_idx;

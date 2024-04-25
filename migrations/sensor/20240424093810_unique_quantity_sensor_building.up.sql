-- Add up migration script here
ALTER TABLE sensor_signals
ADD CONSTRAINT unique_quantity_per_sensor UNIQUE (sensor_id, quantity);

CREATE UNIQUE INDEX unique_sensor_per_building_idx
ON sensors (building_id)
WHERE (building_id IS NOT NULL);

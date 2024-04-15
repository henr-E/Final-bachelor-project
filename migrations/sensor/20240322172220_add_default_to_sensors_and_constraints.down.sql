-- Down migration script to revert the changes

-- Remove DEFAULT gen_random_uuid() from id in the sensors table
ALTER TABLE sensors
    ALTER COLUMN id DROP DEFAULT;

-- Add DEFAULT gen_random_uuid() to sensor_id in archive_sensor_data_files table
ALTER TABLE archive_sensor_data_files
    ALTER COLUMN sensor_id SET DEFAULT gen_random_uuid();

ALTER TABLE sensor_signals
    DROP CONSTRAINT sensors_id_fk;

ALTER TABLE sensor_signals
    ADD CONSTRAINT sensor_signals_sensor_id_fkey
        FOREIGN KEY (sensor_id)
            REFERENCES sensors(id);

ALTER TABLE archive_sensor_data_files
    DROP CONSTRAINT sensors_id_fk;

ALTER TABLE archive_sensor_data_files
    ADD CONSTRAINT fk_sensor_id
        FOREIGN KEY (sensor_id)
            REFERENCES sensors(id);

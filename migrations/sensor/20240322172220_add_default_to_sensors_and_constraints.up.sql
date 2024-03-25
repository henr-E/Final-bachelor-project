-- Add up migration script here

-- Add DEFAULT gen_random_uuid() to id in the sensors table
ALTER TABLE sensors
    ALTER COLUMN id SET DEFAULT gen_random_uuid();

-- Remove DEFAULT gen_random_uuid() from sensor_id in archive_sensor_data_files table
ALTER TABLE archive_sensor_data_files
    ALTER COLUMN sensor_id DROP DEFAULT;

ALTER TABLE sensor_signals
    DROP CONSTRAINT sensor_signals_sensor_id_fkey;

ALTER TABLE sensor_signals
    ADD CONSTRAINT sensors_id_fk
        FOREIGN KEY (sensor_id)
            REFERENCES sensors(id)
            ON DELETE CASCADE;

ALTER TABLE archive_sensor_data_files DROP CONSTRAINT fk_sensor_id;

ALTER TABLE archive_sensor_data_files
    ADD CONSTRAINT sensors_id_fk
        FOREIGN KEY (sensor_id)
            REFERENCES sensors(id)
            ON DELETE CASCADE;

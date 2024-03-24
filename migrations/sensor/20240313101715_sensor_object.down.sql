-- Add down migration script here
DROP TABLE sensor_values;
DROP TABLE sensor_signals;
DROP TABLE sensors;

-- Restore archive_sensor_data_file table.
ALTER TABLE archive_sensor_data_files
    DROP CONSTRAINT fk_sensor_id;
ALTER TABLE archive_sensor_data_files
    DROP COLUMN sensor_id;
ALTER TABLE archive_sensor_data_files
    ALTER COLUMN identifier TYPE CHAR(26);
ALTER TABLE archive_sensor_data_files
    RENAME TO archive_sensor_data_file;

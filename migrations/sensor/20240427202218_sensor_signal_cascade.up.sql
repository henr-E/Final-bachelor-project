-- Add up migration script here
ALTER TABLE sensor_values
    DROP CONSTRAINT sensor_values_sensor_signal_id_fkey;

ALTER TABLE sensor_values
    ADD CONSTRAINT sensor_values_sensor_signal_id_fkey
        FOREIGN KEY (sensor_signal_id)
            REFERENCES sensor_signals(sensor_signal_id)
            ON DELETE CASCADE;

-- Add up migration script here
ALTER TABLE sensor_signals
    ALTER COLUMN prefix TYPE DECIMAL(61, 30);

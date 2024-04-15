-- Add down migration script here
ALTER TABLE sensor_signals
    ALTER COLUMN prefix TYPE DECIMAL(32, 16);

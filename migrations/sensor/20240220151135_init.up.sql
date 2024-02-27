-- Add up migration script here

-- Drop table if it already exists.
DROP TABLE IF EXISTS sensor_data_files;
-- Create the table mapping identifiers to paths
CREATE TABLE sensor_data_files (
	identifier char(26) NOT NULL PRIMARY KEY,
	time TIMESTAMPTZ NOT NULL,
	path TEXT NOT NULL,
	metadata TEXT NOT NULL
);

-- creating a timescale hypertable.
-- SELECT create_hypertable('sensor_data_files', 'time');

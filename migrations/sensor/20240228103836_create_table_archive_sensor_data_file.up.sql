-- Add up migration script here
CREATE TABLE archive_sensor_data_file(
	identifier char(26) NOT NULL PRIMARY KEY,
	time TIMESTAMPTZ NOT NULL,
	path TEXT NOT NULL,
	metadata TEXT NOT NULL
);

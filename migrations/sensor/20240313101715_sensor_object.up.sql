-- Add up migration script here
/* Sensor table containing logistical information about a sensor. */
CREATE TABLE sensors (
    id          UUID,
    name        VARCHAR NOT NULL,
    description VARCHAR,
    location    POINT   NOT NULL, -- (x, y) coordinate pair.
    user_id     INT     NOT NULL, -- TODO: Create foreign key constraint to the users table.
    PRIMARY KEY (id)
);

-- Migrate the `archive_sensor_data_file` to reflect:
-- ```sql
-- CREATE TABLE archive_sensor_data_files (
--     sensor_id  UUID        NOT NULL,
--     time       TIMESTAMPTZ NOT NULL UNIQUE,
--     identifier UUID        NOT NULL,
--     path       TEXT        NOT NULL,
--     metadata   TEXT        NOT NULL,
--     PRIMARY KEY (identifier),
--     FOREIGN KEY (sensor_id) REFERENCES sensors(id)
-- );
-- ```
ALTER TABLE archive_sensor_data_file
    RENAME TO archive_sensor_data_files;
ALTER TABLE archive_sensor_data_files
    DROP CONSTRAINT archive_sensor_data_file_pkey;
ALTER TABLE archive_sensor_data_files
    ADD sensor_id UUID NOT NULL DEFAULT gen_random_uuid();
ALTER TABLE archive_sensor_data_files
    ADD CONSTRAINT fk_sensor_id FOREIGN KEY (sensor_id) REFERENCES sensors(id);
ALTER TABLE archive_sensor_data_files
    -- NOTE: This change might fail, I think, but the IDE does not complain,
    --       nor does postgres when performing the migration.
    ALTER COLUMN identifier TYPE UUID USING identifier::UUID;

-- creating a timescale hypertable for the `sensor_data_files`table.
SELECT create_hypertable('archive_sensor_data_files', by_range('time', INTERVAL '1 hour'));

CREATE TYPE unit AS ENUM (
    'candela',
    'celsius',
    'coulomb',
    'fahrenheit',
    'farad',
    'hertz',
    'joule',
    'kelvin',
    'kilogram',
    'metre',
    'mile',
    'newton',
    'nits',
    'pascal',
    'pound',
    'volt',
    'watt'
);

CREATE TYPE quantity AS ENUM (
    'capacitance',
    'charge',
    'current',
    'energy',
    'force',
    'frequency',
    'illuminance',
    'length',
    'luminance',
    'luminousintensity',
    'mass',
    'potential',
    'power',
    'pressure',
    'rainfall',
    'resistance',
    'temperature'
);

/* Sensor signals table. Every sensor can send multiple signals to the system. */
CREATE TABLE sensor_signals (
    sensor_signal_id SERIAL,
    sensor_id        UUID     NOT NULL,
    alias            VARCHAR  NOT NULL,
    quantity         quantity NOT NULL, -- This may change to a schema level constraint.
    unit             unit     NOT NULL, -- This unit represents the unit of the original data source.
    prefix           DECIMAL(32, 16) NOT NULL,
    PRIMARY KEY (sensor_signal_id),
    FOREIGN KEY (sensor_id) REFERENCES sensors (id)
);

/*
 * The sensor values table contain transformed instances of a measurement.
 * We will only support 1 sensor_signal per timestamp.
 * A sensor may send multiple signals to the system per timestamp due to the schema definition.
 */
CREATE TABLE sensor_values (
    timestamp        TIMESTAMPTZ     NOT NULL,
    value            DECIMAL(32, 16) NOT NULL,
    sensor_signal_id INT             NOT NULL,
    PRIMARY KEY (timestamp, sensor_signal_id),
    FOREIGN KEY (sensor_signal_id) REFERENCES sensor_signals (sensor_signal_id)
);

-- Create hypertable index for `sensor_values` table.
SELECT create_hypertable('sensor_values', by_range('timestamp', INTERVAL '1 day'));

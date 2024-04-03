-- Add up migration script here
CREATE TABLE simulations
(
    id SERIAL PRIMARY KEY,
    twin_id INTEGER NOT NULL,
    FOREIGN KEY (twin_id) REFERENCES twins(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    start_date_time INTEGER NOT NULL,
    end_date_time INTEGER NOT NULL,
    creation_date_time INTEGER NOT NULL,
    frames_loaded INTEGER NOT NULL,
    status INTEGER NOT NULL
);
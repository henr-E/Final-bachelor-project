CREATE TABLE users
(
    id uuid PRIMARY KEY,
    username VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(1024) NOT NULL
);

CREATE TABLE twins
(
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    longitude FLOAT8 NOT NULL,
    latitude FLOAT8 NOT NULL,
    radius FLOAT8 NOT NULL
);

CREATE TABLE buildings
(
    id SERIAL PRIMARY KEY,
    street TEXT,
    house_number TEXT,
    postcode TEXT,
    city TEXT,
    coordinates JSONB,
    visible BOOLEAN,
    twin_id INTEGER NOT NULL,
    FOREIGN KEY (twin_id) REFERENCES twins(id)
);

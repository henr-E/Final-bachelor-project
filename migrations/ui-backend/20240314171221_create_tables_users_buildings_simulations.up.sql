CREATE TABLE buildings
(
    id SERIAL PRIMARY KEY,
    longitude FLOAT8 NOT NULL,
    latitude FLOAT8 NOT NULL
);

CREATE TABLE users
(
    id uuid PRIMARY KEY,
    username VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(1024) NOT NULL
);

CREATE TABLE twins
(
    id SERIAL PRIMARY KEY,
    name VARCHAR(256) NOT NULL,
    longitude FLOAT8 NOT NULL,
    latitude FLOAT8 NOT NULL,
    radius INT NOT NULL
);

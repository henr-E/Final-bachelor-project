CREATE TABLE simulations
(
    id           SERIAL PRIMARY KEY,
    date         DATE    NOT NULL DEFAULT CURRENT_DATE,
    name         VARCHAR NOT NULL,
    step_size_ms INT     NOT NULL DEFAULT 3600000,
    max_steps    INT     NOT NULL DEFAULT 10
);

CREATE TABLE nodes
(
    id            SERIAL PRIMARY KEY,
    node_id       INT    NOT NULL,
    simulation_id INT    NOT NULL,
    time_step     INT    NOT NULL,
    longitude     FLOAT8 NOT NULL,
    latitude      FLOAT8 NOT NULL,
    FOREIGN KEY (simulation_id) REFERENCEs simulations (id),
    UNIQUE (node_id, simulation_id, time_step)
);

CREATE TABLE edges
(
    id             SERIAL PRIMARY KEY,
    edge_id        INT     NOT NULL,
    simulation_id  INT     NOT NULL,
    time_step      INT     NOT NULL,
    from_node      INT     NOT NULL,
    to_node        INT     NOT NULL,
    component_data JSONB   NOT NULL,
    component_type VARCHAR NOT NULL,
    UNIQUE (id, simulation_id, time_step, from_node, to_node),
    FOREIGN KEY (simulation_id) REFERENCES simulations (id),
    FOREIGN KEY (from_node, simulation_id, time_step) REFERENCES nodes (node_id, simulation_id, time_step),
    FOREIGN KEY (to_node, simulation_id, time_step) REFERENCES nodes (node_id, simulation_id, time_step)
);

CREATE TABLE global_components
(
    id             SERIAL PRIMARY KEY,
    time_step      INT     NOT NULL,
    name           VARCHAR NOT NULL,
    simulation_id  INT     NOT NULL,
    component_data JSONB   NOT NULL,
    FOREIGN KEY (simulation_id) REFERENCES simulations (id)
);

CREATE TABLE node_components
(
    id             SERIAL PRIMARY KEY,
    name           VARCHAR NOT NULL,
    node_id        INT     NOT NULL,
    component_data JSONB   NOT NULL,
    FOREIGN KEY (node_id) REFERENCES nodes (id)
);

CREATE TABLE queue
(
    id            SERIAL PRIMARY KEY,
    simulation_id INT NOT NULL,
    FOREIGN KEY (simulation_id) REFERENCES simulations (id)
);

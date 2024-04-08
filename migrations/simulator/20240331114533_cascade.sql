-- Make simulation name unique
ALTER TABLE simulations ADD CONSTRAINT simulation_name UNIQUE(name);

-- Alter foreign keys to cascade on delete
ALTER TABLE nodes DROP CONSTRAINT nodes_simulation_id_fkey;
ALTER TABLE nodes ADD CONSTRAINT nodes_simulation_id_fkey
    FOREIGN KEY (simulation_id)
        REFERENCES simulations (id)
        ON DELETE CASCADE
        ON UPDATE NO ACTION;

ALTER TABLE edges DROP CONSTRAINT edges_simulation_id_fkey;
ALTER TABLE edges ADD CONSTRAINT edges_simulation_id_fkey
    FOREIGN KEY (simulation_id)
        REFERENCES simulations (id)
        ON DELETE CASCADE
        ON UPDATE NO ACTION;

ALTER TABLE global_components DROP CONSTRAINT global_components_simulation_id_fkey;
ALTER TABLE global_components ADD CONSTRAINT global_components_simulation_id_fkey
    FOREIGN KEY (simulation_id)
        REFERENCES simulations (id)
        ON DELETE CASCADE
        ON UPDATE NO ACTION;

ALTER TABLE queue DROP CONSTRAINT queue_simulation_id_fkey;
ALTER TABLE queue ADD CONSTRAINT queue_simulation_id_fkey
    FOREIGN KEY (simulation_id)
        REFERENCES simulations (id)
        ON DELETE CASCADE
        ON UPDATE NO ACTION;

ALTER TABLE node_components DROP CONSTRAINT node_components_node_id_fkey;
ALTER TABLE node_components ADD CONSTRAINT node_components_nodes_id_fkey
    FOREIGN KEY (node_id)
        REFERENCES nodes (id)
        ON DELETE CASCADE
        ON UPDATE NO ACTION;

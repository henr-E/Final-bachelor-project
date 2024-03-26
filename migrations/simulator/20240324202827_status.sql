CREATE TYPE enum_status  AS ENUM ('Pending', 'Computing', 'Finished', 'Failed');
ALTER TABLE simulations ADD COLUMN status enum_status DEFAULT 'Pending';

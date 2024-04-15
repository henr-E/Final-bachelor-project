-- Add migration script here

-- Add new values to the unit enum
ALTER TYPE unit ADD VALUE 'degrees' AFTER 'coulomb';
ALTER TYPE unit ADD VALUE 'meterspersecond' AFTER 'metre';
ALTER TYPE unit ADD VALUE 'millimetersperhour' AFTER 'meterspersecond';
ALTER TYPE unit ADD VALUE 'ohm' AFTER 'nits';
ALTER TYPE unit ADD VALUE 'okta' AFTER 'ohm';
ALTER TYPE unit ADD VALUE 'percentage' AFTER 'pascal';

-- Add new values to the quantity enum
ALTER TYPE quantity ADD VALUE 'cloudiness' AFTER 'charge';
ALTER TYPE quantity ADD VALUE 'relativehumidity' AFTER 'rainfall';
ALTER TYPE quantity ADD VALUE 'winddirection' AFTER 'timestamp';
ALTER TYPE quantity ADD VALUE 'windspeed' AFTER 'winddirection';

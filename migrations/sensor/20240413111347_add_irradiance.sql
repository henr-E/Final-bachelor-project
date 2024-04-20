-- Add migration script here

-- Add new values to the unit enum
ALTER TYPE unit ADD VALUE 'wattspersquaremetre' AFTER 'watt';

-- Add new values to the quantity enum
ALTER TYPE quantity ADD VALUE 'irradiance' AFTER 'illuminance';

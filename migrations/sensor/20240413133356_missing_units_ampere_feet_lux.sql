-- Add migration script here

-- Add new values to the unit enum
ALTER TYPE unit ADD VALUE 'ampere' BEFORE 'candela';
ALTER TYPE unit ADD VALUE 'feet' AFTER 'farad';
ALTER TYPE unit ADD VALUE 'lux' AFTER 'kilogram';

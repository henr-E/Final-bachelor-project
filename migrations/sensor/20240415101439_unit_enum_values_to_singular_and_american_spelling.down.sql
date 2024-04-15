-- Add down migration script here
ALTER TYPE unit RENAME VALUE 'meter' TO 'metre';
ALTER TYPE unit RENAME VALUE 'nit' TO 'nits';

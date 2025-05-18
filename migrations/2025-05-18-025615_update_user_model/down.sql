-- This file should undo anything in `up.sql`
ALTER TABLE users 
ADD COLUMN name varchar,
ADD COLUMN nik numeric,
ADD COLUMN phone_number numeric,
DROP COLUMN role;

DROP TYPE role;

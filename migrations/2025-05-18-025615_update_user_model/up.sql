-- Your SQL goes here
ALTER TABLE users 
DROP COLUMN name,
DROP COLUMN nik,
DROP COLUMN phone_number;

CREATE TYPE role AS ENUM ('pacilian', 'caregiver');

ALTER TABLE users
ADD role role NOT NULL DEFAULT 'pacilian';

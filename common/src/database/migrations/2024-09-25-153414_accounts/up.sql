-- Your SQL goes here
CREATE TABLE accounts (
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    login VARCHAR(12) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    session_token VARCHAR(255)
);
-- Your SQL goes here
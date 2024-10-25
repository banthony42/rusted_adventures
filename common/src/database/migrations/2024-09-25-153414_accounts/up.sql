-- Your SQL goes here
CREATE TABLE accounts (
    login VARCHAR(12) NOT NULL PRIMARY KEY,
    password VARCHAR(255) NOT NULL,
    session_token VARCHAR(255)
);
-- Your SQL goes here
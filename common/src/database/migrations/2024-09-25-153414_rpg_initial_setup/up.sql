-- Your SQL goes here
CREATE TABLE Accounts (
    login VARCHAR(12) NOT NULL PRIMARY KEY,
    password VARCHAR(255) NOT NULL,
    session_token VARCHAR(255)
);

CREATE TYPE RACES AS ENUM('Player', 'Bouftou');

CREATE TYPE CLASSES AS ENUM('Warrior');

CREATE TYPE ENTITY AS (
    name text,
    race RACES,
    m_coord point
);

CREATE TABLE Player (
    name VARCHAR(12) NOT NULL PRIMARY KEY,
    race RACES NOT NULL,
    class CLASSES NOT NULL
);

CREATE TABLE PlayerLocation (
    name VARCHAR(12) NOT NULL PRIMARY KEY,
    w_coord point NOT NULL,
    m_coord point NOT NULL
);

CREATE TABLE World (
    coord_str VARCHAR(9) NOT NULL PRIMARY KEY,
    w_coord point NOT NULL,
    entity ENTITY NOT NULL
);

-- Your SQL goes here
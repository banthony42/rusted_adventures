-- Your SQL goes here

CREATE TYPE PG_CLASSES AS ENUM('Warrior', 'Witcher');

CREATE TABLE entities (
    id SERIAL NOT NULL PRIMARY KEY,
    name VARCHAR(12) NOT NULL UNIQUE
);

CREATE TABLE characters (
    id SERIAL NOT NULL PRIMARY KEY ,
    account_id uuid NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    entity_id integer NOT NULL REFERENCES entities (id) ON DELETE CASCADE,
    class PG_CLASSES NOT NULL
);

CREATE TABLE locations (
    entity_id integer NOT NULL PRIMARY KEY REFERENCES entities (id) ON DELETE CASCADE,
    world point NOT NULL,
    map point
);

CREATE OR REPLACE FUNCTION compute_map_according_to_world() RETURNS TRIGGER AS $$
BEGIN
    UPDATE "locations"
    SET map = point(floor(NEW.world[0] / 1024), floor(NEW.world[1] / 768))
    WHERE entity_id = NEW.entity_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER compute_map_coord_according_to_world_trigger
AFTER INSERT ON "locations"
FOR EACH ROW
EXECUTE FUNCTION compute_map_according_to_world();
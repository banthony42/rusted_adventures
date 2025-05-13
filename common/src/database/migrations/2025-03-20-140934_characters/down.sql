-- This file should undo anything in `up.sql`

DROP TRIGGER IF EXISTS compute_map_coord_according_to_world_trigger ON locations;
DROP FUNCTION IF EXISTS compute_map_according_to_world;

DROP TABLE IF EXISTS characters, entities, locations;

DROP TYPE IF EXISTS PG_CLASSES;


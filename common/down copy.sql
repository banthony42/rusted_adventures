-- Active: 1750172390549@@127.0.0.1@4242@postgres
-- This file should undo anything in `up.sql`
drop table if exists characters, monsters, entities, locations, bestiary;

drop type if exists PGClass, PGBestiary;


-- Your SQL goes here
create type PGClass as ENUM(
    'Warrior',
    'Witcher'
);

create type PGBestiary as ENUM(
    'Bouftou',
    'Human'
);

create table entities(
    id serial primary key,
    uuid uuid not null unique default gen_random_uuid(),
    name varchar(16) not null unique
);

create table characters(
    id serial primary key,
    account_id uuid not null references accounts(id) on delete cascade,
    entity_id integer not null unique references entities(id) on delete cascade,
    class PGClass not null
);

create table monsters(
    id serial primary key,
    entity_id integer not null unique references entities(id) on delete cascade,
    race PGBestiary not null
);

create table locations(
    entity_id integer primary key references entities(id) on delete cascade,
    world point not null,
    map point not null,
    destination point
);


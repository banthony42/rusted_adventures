-- Your SQL goes here
-- Your SQL goes here
create type PGClass as ENUM(
    'Warrior',
    'Mage'
);

create type PGSpecies as ENUM(
    'Bouftou',
    'Crabedoeuf'
);

create table bestiary(
    id serial primary key,
    species PGSpecies not null,
    name varchar(16) not null unique
);

create table locations(
    id serial primary key,
    world point not null,
    cell point not null,
    destination point
);

create table entities(
    id serial primary key,
    location_id integer not null unique references locations(id) on delete cascade
);

create table characters(
    id serial primary key,
    account_id uuid not null references accounts(id) on delete cascade,
    entity_id integer not null unique references entities(id) on delete cascade,
    name varchar(16) not null unique,
    class PGClass not null
);

create table monsters(
    id serial,
    bestiary_id integer not null references bestiary(id) on delete cascade,
    entity_id integer not null unique references entities(id) on delete cascade,
    primary key (bestiary_id, entity_id)
);

insert into bestiary(name, species)
    values ('Bouftou', 'Bouftou'),
('Crabedoeuf', 'Crabedoeuf');


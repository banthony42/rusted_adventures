-- Your SQL goes here
create type PGClass as ENUM(
    'Warrior',
    'Mage'
);

create type PGBestiary as ENUM(
    'Bouftou',
    'Crabedoeuf'
);

create table bestiary(
    id serial primary key,
    species PGBestiary not null,
    name varchar(16) not null unique
);

create table locations(
    id serial primary key,
    world point not null,
    map point not null,
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
    bestiary_id integer not null references bestiary(id) on delete cascade,
    entity_id integer not null unique references entities(id) on delete cascade,
    primary key (bestiary_id, entity_id)
);

insert into bestiary(name, species)
    values ('Bouftou', 'Bouftou'),
('Crabedoeuf', 'Crabedoeuf');

-- Populate world with hardcoded monsters for now
insert into locations(id, world, map)
    values (1, point(0.0, 0.0), point(5.0, 5.0)),
(2, point(0.0, 0.0), point(7.0, 7.0)),
(3, point(1.0, 0.0), point(2.0, 2.0)),
(4, point(1.0, 0.0), point(6.0, 6.0));

insert into entities(id, location_id)
    values (1, 1),
(2, 2),
(3, 3),
(4, 4);

insert into monsters(bestiary_id, entity_id)
    values (1, 1),
(1, 2),
(2, 3),
(2, 4);


-- Your SQL goes here
create table accounts(
    id uuid not null primary key default gen_random_uuid(),
    login VARCHAR(12) not null unique,
    password VARCHAR(255) not null,
    session_token varchar(255)
);

-- Your SQL goes here

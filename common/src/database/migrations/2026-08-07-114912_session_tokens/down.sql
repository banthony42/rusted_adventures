-- This file should undo anything in `up.sql`
drop table if exists sessions;

alter table accounts
    add column session_token varchar(255);


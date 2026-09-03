-- Your SQL goes here
alter table accounts
    add column login_failure_count integer not null default 0,
    add column login_window_started_at timestamptz not null default now(),
    add column locked_until timestamptz,
    add column lockout_count integer not null default 0;


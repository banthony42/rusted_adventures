-- Your SQL goes here
alter table accounts
    drop if exists session_token;

create table sessions(
    id uuid not null primary key default gen_random_uuid(),
    account_id uuid not null references accounts(id) on delete cascade,
    token_hash char(64) not null unique, -- SHA-256 hex du token
    created_at timestamptz not null default now(),
    expires_at timestamptz not null,
    last_used_at timestamptz not null default now()
);


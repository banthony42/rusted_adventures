-- This file should undo anything in `up.sql`
alter table accounts
    drop column login_failure_count,
    drop column login_window_started_at,
    drop column locked_until,
    drop column lockout_count;


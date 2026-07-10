-- Deliberately named `p3_04_02_widgets`, not just `widgets` — this lesson
-- shares a Postgres database with several other lessons/the capstone in
-- this repo, and table names are global within a database, so a
-- lesson-scoped prefix avoids colliding with anyone else's tables.
--
-- No `IF NOT EXISTS` here on purpose: sqlx's migrator records every
-- applied migration (by checksum) in its own `_sqlx_migrations` tracking
-- table and simply skips anything already recorded, so re-running
-- `sqlx::migrate!(...).run(&pool)` a second time is already a safe no-op
-- without needing the SQL itself to be idempotent.
CREATE TABLE p3_04_02_widgets (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

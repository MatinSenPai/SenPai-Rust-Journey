-- The mirror image of `0001_create_accounts.up.sql`, run only by
-- `sqlx migrate revert` — never by `sqlx::migrate!(...).run(pool)`.
-- A down-migration undoes everything its up created, in reverse order
-- (here there's only one thing to undo). Reverting also deletes this
-- migration's row from `_sqlx_migrations`, so a subsequent
-- `run_migrations` will apply the up again from scratch.
DROP TABLE p3_04_04_accounts;

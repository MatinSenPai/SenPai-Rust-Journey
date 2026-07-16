-- A REVERSIBLE migration: this `.up.sql` has a paired
-- `0001_create_accounts.down.sql` that undoes it. `sqlx::migrate!` applies
-- up-migrations exactly like the single-file style from lesson 04.2; the
-- down file only ever runs via `sqlx migrate revert` (the sqlx-cli tool —
-- see README.md). One rule to know: a migrations directory must be all one
-- style — sqlx refuses to mix reversible (.up/.down) and simple (.sql)
-- migrations in the same directory.
CREATE TABLE p3_04_04_accounts (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    -- Money is integer cents, never floating point: 0.1 + 0.2 != 0.3 in
    -- binary floats, and "close enough" is not a property you want your
    -- ledger to have.
    --
    -- The CHECK is the database-enforced backstop for "a balance never goes
    -- negative." Application code checks first (to return a friendlier,
    -- typed InsufficientFunds error), but the constraint holds even if some
    -- future code path forgets to check — defense in depth, enforced at the
    -- last gate every write must pass through.
    balance_cents BIGINT NOT NULL CHECK (balance_cents >= 0)
);

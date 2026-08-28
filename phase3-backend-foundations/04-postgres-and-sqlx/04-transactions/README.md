# 04.4 — Transactions

Debit Alice, credit Bob. Two `UPDATE`s. If the process dies — or an error
takes an early return — *between* them, Alice paid and Bob never received:
money destroyed by a code path nobody wrote on purpose. That's the classic
double-write failure, and every lesson so far dodged it by only ever
needing one statement at a time. Single statements are already atomic;
**transactions make a *group* of statements atomic**: all of them commit
together, or none of them happened at all.

In Django you've done this with a context manager:

```python
from django.db import transaction

with transaction.atomic():
    alice.balance -= 250; alice.save()
    bob.balance += 250; bob.save()
# leaving the block normally → COMMIT; an exception escaping it → ROLLBACK
```

## The shape of a sqlx transaction

```rust
let mut tx = pool.begin().await?;          // BEGIN

sqlx::query("UPDATE ...")
    .execute(&mut *tx)                     // note: &mut *tx, not &pool
    .await?;

tx.commit().await?;                        // COMMIT — nothing persists before this
```

Three things worth staring at:

- **`pool.begin()`** checks one connection out of the pool and issues
  `BEGIN` on it. The returned `Transaction` *owns* that connection until
  commit, rollback, or drop — which matters because a transaction only
  exists on one connection.
- **`&mut *tx`** — every statement in the transaction must run on that same
  owned connection. Passing `&pool` instead would grab some *other* pool
  connection, silently running your query *outside* the transaction. The
  types stop you: queries inside a transaction take `&mut *tx`
  (`Transaction` derefs to the underlying `PgConnection`).
- **`tx.commit().await?`** — explicit, unlike Django's implicit
  commit-on-block-exit. Forgetting it doesn't corrupt anything: nothing
  persists, and your tests fail loudly.

## Rollback is the default, not something you remember to do

Here's the killer feature. If a `Transaction` is **dropped without
`commit`** — early `return`, a `?` propagating an error, even a panic —
sqlx issues a `ROLLBACK` before that connection returns to the pool.
Django rolls back when an exception escapes the `with` block; Rust has no
exceptions, so sqlx hangs the same guarantee on the thing Rust *does*
guarantee: `Drop` runs. Every `?` inside `transfer` is therefore secretly
also a rollback point — you cannot forget to roll back, because rollback
is what happens when you *don't* write `commit`.

## Check-then-write needs a row lock

`transfer` reads the sender's balance, checks it, then debits. Two
concurrent transfers from the same account could both read `100`, both
decide `60` is affordable, and jointly overdraw. `SELECT ... FOR UPDATE`
(Django: `select_for_update()`) locks the row until this transaction
commits or rolls back, so the second transfer waits and then sees the
post-debit balance. And if a bug ever slips past the check anyway, the
migration's `CHECK (balance_cents >= 0)` constraint is the backstop the
database enforces on every write, no matter which code path made it.
(Balances are integer cents, never floats — see the migration's comment.)

## Reversible migrations and `sqlx migrate revert`

Lesson 04.2 used single-file migrations (`0001_create_widgets.sql`) —
forward-only. This lesson's `migrations/` uses the **reversible** style:

```
0001_create_accounts.up.sql     # applied by sqlx::migrate! / sqlx migrate run
0001_create_accounts.down.sql   # applied ONLY by `sqlx migrate revert`
```

`run_migrations` treats the `.up.sql` exactly like 04.2's single file.
The down file exists for the day a deploy goes wrong: `sqlx migrate
revert` (from sqlx-cli: `cargo install sqlx-cli --no-default-features
--features postgres`) undoes the **most recently applied** migration by
running its down file and deleting its `_sqlx_migrations` row — one
migration per invocation, newest first. Django's `migrate app 0005`
"migrate backwards" is the same idea, except Django auto-derives the
reverse; here you write the `DROP TABLE` yourself. One rule: a migrations
directory is all one style — sqlx refuses to mix `.sql` and
`.up.sql`/`.down.sql` files side by side.

```sh
DATABASE_URL=postgres://taskforge:taskforge@localhost:5432/p3_04_04_transactions \
  sqlx migrate revert --source phase3-backend-foundations/04-postgres-and-sqlx/04-transactions/migrations
```

## Why this lesson gets its own database (again)

Same trap lesson 04.2 documented: `_sqlx_migrations` is per-*database*,
keyed by version number, and this migration is "version 1" like 04.2's and
the capstone's — sharing a database ends in `VersionMismatch`. One-time
setup:

```sh
sudo -u postgres psql -c "CREATE DATABASE p3_04_04_transactions OWNER taskforge;"
```

## Your task

Open `src/lib.rs`. Implement `create_account` and `transfer` (the doc
comment on `transfer` lays out the seven steps). `run_migrations` and
`balance` are given.

## Next

`cargo test -p p3-04-04-transactions` (the error-type and
validate-before-connect tests run with zero infrastructure), then:

```sh
DATABASE_URL=postgres://taskforge:taskforge@localhost:5432/p3_04_04_transactions \
  cargo test -p p3-04-04-transactions -- --ignored
```

Then `solution/SOLUTION.md` — but only after a real attempt.

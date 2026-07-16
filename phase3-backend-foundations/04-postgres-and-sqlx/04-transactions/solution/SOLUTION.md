# Solution

```rust
pub async fn transfer(
    pool: &PgPool,
    from: i64,
    to: i64,
    amount_cents: i64,
) -> Result<(), TransferError> {
    if amount_cents <= 0 {
        return Err(TransferError::AmountNotPositive(amount_cents));
    }

    let mut tx = pool.begin().await?;

    let available: i64 = sqlx::query_scalar(
        "SELECT balance_cents FROM p3_04_04_accounts WHERE id = $1 FOR UPDATE",
    )
    .bind(from)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(TransferError::AccountNotFound(from))?;

    if available < amount_cents {
        return Err(TransferError::InsufficientFunds {
            available,
            requested: amount_cents,
        });
    }

    sqlx::query("UPDATE p3_04_04_accounts SET balance_cents = balance_cents - $1 WHERE id = $2")
        .bind(amount_cents)
        .bind(from)
        .execute(&mut *tx)
        .await?;

    let credited =
        sqlx::query("UPDATE p3_04_04_accounts SET balance_cents = balance_cents + $1 WHERE id = $2")
            .bind(amount_cents)
            .bind(to)
            .execute(&mut *tx)
            .await?;
    if credited.rows_affected() == 0 {
        return Err(TransferError::AccountNotFound(to));
    }

    tx.commit().await?;
    Ok(())
}
```

## Where the rollback "is"

Count the error exits in `transfer` after `pool.begin()`: three `?`s that
can carry a `sqlx::Error`, the `ok_or` for a missing sender, the
`InsufficientFunds` return, the `rows_affected() == 0` return. **None of
them mention rollback.** They don't need to: every one of those paths
leaves the function without calling `tx.commit()`, so `tx` is dropped, and
sqlx's `Transaction` issues a `ROLLBACK` on its connection before handing
it back to the pool. This is Rust's ownership model doing transactional
bookkeeping for you — the same reasoning as a `MutexGuard` unlocking on
drop, applied to `BEGIN`/`ROLLBACK`.

That's also why `a_failed_credit_rolls_back_the_debit` is the most
important test in the lesson: the debit `UPDATE` genuinely executed inside
the transaction, and the test proves it left no trace. Django gives you
the same guarantee via `transaction.atomic()` catching exceptions at the
`with`-block boundary; sqlx hangs it on `Drop`, which fires on *every*
exit path — early return, `?`, or panic — with zero code at the exit
sites.

## Why `&mut *tx` and not `&pool`

A transaction lives on **one** connection; `BEGIN` on connection A does
nothing for a query running on connection B. `Transaction` owns its
checked-out connection, and `&mut *tx` derefs to that `PgConnection` —
which is what implements `Executor` for the "inside a transaction" case.
If you passed `&pool` by accident, the query would grab a *different*
connection and run outside the transaction (auto-committed, invisible to
your rollback). The `&mut` also means the borrow checker enforces one
statement at a time on the transaction — you physically can't interleave
two concurrent queries on it the way you can on a pool.

## `FOR UPDATE`: the check-then-act race

Without the lock: transfers T1 and T2 both read `balance = 100`, both
pass the `>= 60` check, both debit 60, final balance -20 (or, here, a
CHECK-constraint error on whichever commits second — better than
corruption, but a raw database error instead of a typed
`InsufficientFunds`). With `FOR UPDATE`, T2's `SELECT` blocks until T1
commits or rolls back, then reads the post-T1 balance and fails the check
properly. This is exactly Django's `select_for_update()`, which likewise
only works inside `transaction.atomic()` — a row lock with no transaction
to scope it would have no defined moment to release.

The CHECK constraint stays in the migration anyway. The application check
gives callers a *useful* error; the constraint guarantees the invariant
against every path — including `create_account`, which never checks, and
whatever code gets written next year.

## `rows_affected()` as an existence check

The sender's existence is proven by the `SELECT ... FOR UPDATE` returning
a row. For the recipient there's no earlier read, so the credit `UPDATE`
itself is the check: updating zero rows means no such account. That's one
round trip cheaper than SELECT-then-UPDATE, and inside the transaction
it's race-free — deciding based on the UPDATE's own result can't disagree
with what the UPDATE did.

## Why the DB tests skip `#[serial]`

Lessons 04.2/04.3 serialize their DB tests because they share mutable
state: one table that tests drop, wipe, or count. Here every test creates
fresh accounts (`BIGSERIAL` ids never collide) and asserts only on those
ids — there is no shared row to race over, so the default concurrent test
execution is safe. The concurrent `run_migrations` calls are also fine:
sqlx's migrator wraps its work in a Postgres advisory lock. The general
rule: serialize tests that share mutable state, not tests that share a
database.

## Reversible migrations in practice

`sqlx::migrate!` embeds and runs only the `.up.sql` files —
`run_migrations` behaves identically to lesson 04.2. The `.down.sql` file
is for the CLI: `sqlx migrate revert` runs the newest applied migration's
down file and deletes its `_sqlx_migrations` row, so the next
`run_migrations` re-applies the up from scratch. It reverts one migration
per invocation, newest first — there is deliberately no "revert all"
footgun. Two habits worth forming: write the down migration at the same
sitting as the up (you still remember what the up did), and remember that
a `DROP TABLE` down-migration destroys data — reverting on production is
a bigger decision than reverting on your laptop.

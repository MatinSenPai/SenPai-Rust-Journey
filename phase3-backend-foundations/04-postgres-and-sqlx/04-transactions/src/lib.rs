//! A tiny bank ledger used to teach the one database feature nothing else
//! in this module has needed yet: **multi-statement atomicity**. A transfer
//! is two `UPDATE`s that must both happen or both not happen — the classic
//! double-write problem. See `README.md` for the theory (and the Django
//! `transaction.atomic()` comparison) before touching any `todo!()`.

use sqlx::PgPool;

/// Everything that can go wrong moving money. The first three variants are
/// *domain* errors a caller can meaningfully branch on (an API layer would
/// map them to 400/404/409); `Database` wraps any underlying `sqlx::Error`.
/// The `#[from]` is what lets `transfer` use `?` on every sqlx call — and
/// inside a transaction, that early return is also what triggers rollback:
/// the `Transaction` value is dropped without `commit`, so sqlx issues a
/// `ROLLBACK` before the connection goes back to the pool.
#[derive(Debug, thiserror::Error)]
pub enum TransferError {
    #[error("transfer amount must be positive, got {0} cents")]
    AmountNotPositive(i64),
    #[error("account {0} does not exist")]
    AccountNotFound(i64),
    #[error("insufficient funds: account has {available} cents, transfer needs {requested} cents")]
    InsufficientFunds { available: i64, requested: i64 },
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Same as lesson 04.2's `run_migrations` — except this time
/// `./migrations/` holds a *reversible* pair: an
/// `0001_create_accounts.up.sql` next to a matching `.down.sql`.
/// `sqlx::migrate!` runs the `.up.sql` files exactly like the single-file
/// style; the `.down.sql` files only run via `sqlx migrate revert` (see
/// README).
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}

/// Reads one account's balance. Given, fully implemented — it's the same
/// `query_scalar` shape as lesson 04.2, plus the `fetch_optional` →
/// `ok_or` dance that turns "no row" into a typed domain error instead of
/// a generic `RowNotFound`.
pub async fn balance(pool: &PgPool, account_id: i64) -> Result<i64, TransferError> {
    sqlx::query_scalar("SELECT balance_cents FROM p3_04_04_accounts WHERE id = $1")
        .bind(account_id)
        .fetch_optional(pool)
        .await?
        .ok_or(TransferError::AccountNotFound(account_id))
}

/// Opens an account with a starting balance and returns its id.
///
/// No transaction needed here: it's a **single** statement, and single
/// statements are already atomic on their own — Postgres never applies
/// half an `INSERT`. Transactions exist for the *multi*-statement case.
/// A negative opening balance is rejected by the table's CHECK constraint
/// and surfaces as `TransferError::Database`.
pub async fn create_account(
    pool: &PgPool,
    name: &str,
    opening_balance_cents: i64,
) -> Result<i64, TransferError> {
    todo!(
        "sqlx::query_scalar(\"INSERT INTO p3_04_04_accounts (name, balance_cents) \
         VALUES ($1, $2) RETURNING id\"), bind name then opening_balance_cents, \
         .fetch_one(pool).await? (the ? converts sqlx::Error into \
         TransferError::Database via #[from]), then wrap the id in Ok"
    )
}

/// Moves `amount_cents` from account `from` to account `to` — atomically.
/// Either both the debit and the credit land, or neither does; no
/// interleaving of failures can leave money created or destroyed.
///
/// The intended shape (each step is explained in README.md):
///
/// 1. Reject `amount_cents <= 0` *before* touching the database.
/// 2. `let mut tx = pool.begin().await?;`
/// 3. `SELECT balance_cents FROM p3_04_04_accounts WHERE id = $1 FOR UPDATE`
///    against `&mut *tx` (`fetch_optional`; `None` → `AccountNotFound(from)`).
/// 4. If the balance is short, return `InsufficientFunds` — just return;
///    the dropped `tx` rolls itself back.
/// 5. Debit: `UPDATE ... SET balance_cents = balance_cents - $1 WHERE id = $2`
///    against `&mut *tx`.
/// 6. Credit: same but `+ $1` for `to`. If `.rows_affected() == 0`, the
///    recipient doesn't exist: return `AccountNotFound(to)` and let the
///    rollback undo the debit that already executed.
/// 7. `tx.commit().await?;` — nothing above persists until this line runs.
pub async fn transfer(
    pool: &PgPool,
    from: i64,
    to: i64,
    amount_cents: i64,
) -> Result<(), TransferError> {
    todo!(
        "follow the 7 numbered steps in the doc comment above; every query \
         inside the transaction executes against &mut *tx, not pool"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;

    // ------------------------------------------------------------------
    // Always-on tests: no database required, they run on plain
    // `cargo test -p p3-04-04-transactions`.
    // ------------------------------------------------------------------

    #[test]
    fn error_messages_are_human_readable() {
        assert_eq!(
            TransferError::AmountNotPositive(0).to_string(),
            "transfer amount must be positive, got 0 cents"
        );
        assert_eq!(
            TransferError::AccountNotFound(42).to_string(),
            "account 42 does not exist"
        );
        assert_eq!(
            TransferError::InsufficientFunds {
                available: 100,
                requested: 250
            }
            .to_string(),
            "insufficient funds: account has 100 cents, transfer needs 250 cents"
        );
    }

    #[test]
    fn a_sqlx_error_converts_via_from_so_question_mark_works() {
        let err = TransferError::from(sqlx::Error::RowNotFound);
        assert!(matches!(err, TransferError::Database(_)));
    }

    /// `connect_lazy` builds a pool WITHOUT opening any connection — the
    /// first real query would be the first network touch. So this test
    /// passes without a database *if and only if* `transfer` validates the
    /// amount before `pool.begin()`. Validate-then-connect, not the other
    /// way around.
    #[tokio::test]
    async fn rejects_a_non_positive_amount_before_touching_the_database() {
        let pool = PgPoolOptions::new()
            .acquire_timeout(Duration::from_secs(2))
            .connect_lazy("postgres://nobody:nothing@localhost:1/never-connected")
            .unwrap();

        let err = transfer(&pool, 1, 2, 0).await.unwrap_err();
        assert!(matches!(err, TransferError::AmountNotPositive(0)));

        let err = transfer(&pool, 1, 2, -5).await.unwrap_err();
        assert!(matches!(err, TransferError::AmountNotPositive(-5)));
    }

    // ------------------------------------------------------------------
    // Database tests: `#[ignore]`-gated like every Postgres lesson in this
    // module. Note there's no `#[serial]` here, unlike lessons 04.2/04.3 —
    // each test below only ever touches accounts it created itself (fresh
    // ids from BIGSERIAL), so there is no shared row for concurrent tests
    // to race over. Concurrent `run_migrations` calls are safe too: sqlx's
    // migrator takes a Postgres advisory lock while it works.
    // ------------------------------------------------------------------

    async fn test_pool() -> PgPool {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set to run this ignored integration test");
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .unwrap();
        run_migrations(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    #[ignore = "requires a live Postgres reachable at DATABASE_URL"]
    async fn transfer_moves_money_between_accounts() {
        let pool = test_pool().await;
        let alice = create_account(&pool, "alice", 1_000).await.unwrap();
        let bob = create_account(&pool, "bob", 0).await.unwrap();

        transfer(&pool, alice, bob, 250).await.unwrap();

        assert_eq!(balance(&pool, alice).await.unwrap(), 750);
        assert_eq!(balance(&pool, bob).await.unwrap(), 250);
    }

    #[tokio::test]
    #[ignore = "requires a live Postgres reachable at DATABASE_URL"]
    async fn insufficient_funds_fails_and_changes_nothing() {
        let pool = test_pool().await;
        let alice = create_account(&pool, "alice", 100).await.unwrap();
        let bob = create_account(&pool, "bob", 0).await.unwrap();

        let err = transfer(&pool, alice, bob, 250).await.unwrap_err();

        assert!(matches!(
            err,
            TransferError::InsufficientFunds {
                available: 100,
                requested: 250
            }
        ));
        assert_eq!(balance(&pool, alice).await.unwrap(), 100);
        assert_eq!(balance(&pool, bob).await.unwrap(), 0);
    }

    /// THE test this lesson exists for. The debit `UPDATE` genuinely
    /// executes before the credit is attempted — and then the credit finds
    /// no recipient, `transfer` returns early, the `Transaction` drops, and
    /// the already-executed debit is rolled back as if it never happened.
    /// Without the transaction, alice would be 250 cents poorer and the
    /// money would simply be gone.
    #[tokio::test]
    #[ignore = "requires a live Postgres reachable at DATABASE_URL"]
    async fn a_failed_credit_rolls_back_the_debit() {
        let pool = test_pool().await;
        let alice = create_account(&pool, "alice", 1_000).await.unwrap();

        let err = transfer(&pool, alice, i64::MAX, 250).await.unwrap_err();

        assert!(matches!(err, TransferError::AccountNotFound(id) if id == i64::MAX));
        assert_eq!(balance(&pool, alice).await.unwrap(), 1_000);
    }

    #[tokio::test]
    #[ignore = "requires a live Postgres reachable at DATABASE_URL"]
    async fn a_missing_sender_fails_cleanly() {
        let pool = test_pool().await;
        let bob = create_account(&pool, "bob", 0).await.unwrap();

        let err = transfer(&pool, i64::MAX, bob, 250).await.unwrap_err();

        assert!(matches!(err, TransferError::AccountNotFound(id) if id == i64::MAX));
        assert_eq!(balance(&pool, bob).await.unwrap(), 0);
    }

    /// The application-level check in `transfer` returns the friendly
    /// error, but the migration's `CHECK (balance_cents >= 0)` is the
    /// database-enforced backstop — here it catches a path (`create_account`)
    /// that never checks at all.
    #[tokio::test]
    #[ignore = "requires a live Postgres reachable at DATABASE_URL"]
    async fn the_check_constraint_rejects_a_negative_opening_balance() {
        let pool = test_pool().await;

        let result = create_account(&pool, "overdrawn-from-birth", -1).await;

        assert!(matches!(result, Err(TransferError::Database(_))));
    }
}

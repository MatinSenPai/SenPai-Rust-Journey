use sqlx::PgPool;

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

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}

pub async fn balance(pool: &PgPool, account_id: i64) -> Result<i64, TransferError> {
    sqlx::query_scalar("SELECT balance_cents FROM p3_04_04_accounts WHERE id = $1")
        .bind(account_id)
        .fetch_optional(pool)
        .await?
        .ok_or(TransferError::AccountNotFound(account_id))
}

pub async fn create_account(
    pool: &PgPool,
    name: &str,
    opening_balance_cents: i64,
) -> Result<i64, TransferError> {
    let id = sqlx::query_scalar(
        "INSERT INTO p3_04_04_accounts (name, balance_cents) VALUES ($1, $2) RETURNING id",
    )
    .bind(name)
    .bind(opening_balance_cents)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

pub async fn transfer(
    pool: &PgPool,
    from: i64,
    to: i64,
    amount_cents: i64,
) -> Result<(), TransferError> {
    // 1. Validate before any I/O — no reason to borrow a connection for a
    //    request that's wrong on its face.
    if amount_cents <= 0 {
        return Err(TransferError::AmountNotPositive(amount_cents));
    }

    // 2. BEGIN. From here down, every early return (`?` included) drops
    //    `tx` without commit, which rolls everything back.
    let mut tx = pool.begin().await?;

    // 3. Read the sender's balance and lock the row until commit/rollback,
    //    so a concurrent transfer can't read the same pre-debit balance.
    let available: i64 =
        sqlx::query_scalar("SELECT balance_cents FROM p3_04_04_accounts WHERE id = $1 FOR UPDATE")
            .bind(from)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or(TransferError::AccountNotFound(from))?;

    // 4. The friendly, typed version of what the CHECK constraint would
    //    otherwise enforce with a generic database error.
    if available < amount_cents {
        return Err(TransferError::InsufficientFunds {
            available,
            requested: amount_cents,
        });
    }

    // 5. Debit. This executes for real — but persists only if we commit.
    sqlx::query("UPDATE p3_04_04_accounts SET balance_cents = balance_cents - $1 WHERE id = $2")
        .bind(amount_cents)
        .bind(from)
        .execute(&mut *tx)
        .await?;

    // 6. Credit. rows_affected() == 0 means no such recipient: return the
    //    error and let the dropped transaction undo the debit above.
    let credited = sqlx::query(
        "UPDATE p3_04_04_accounts SET balance_cents = balance_cents + $1 WHERE id = $2",
    )
    .bind(amount_cents)
    .bind(to)
    .execute(&mut *tx)
    .await?;
    if credited.rows_affected() == 0 {
        return Err(TransferError::AccountNotFound(to));
    }

    // 7. COMMIT — the single moment both updates become visible, together.
    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;

    // ------------------------------------------------------------------
    // Always-on tests: no database required.
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
    // Database tests: `#[ignore]`-gated. No `#[serial]` needed — each test
    // only touches accounts it created itself.
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

    #[tokio::test]
    #[ignore = "requires a live Postgres reachable at DATABASE_URL"]
    async fn the_check_constraint_rejects_a_negative_opening_balance() {
        let pool = test_pool().await;

        let result = create_account(&pool, "overdrawn-from-birth", -1).await;

        assert!(matches!(result, Err(TransferError::Database(_))));
    }
}

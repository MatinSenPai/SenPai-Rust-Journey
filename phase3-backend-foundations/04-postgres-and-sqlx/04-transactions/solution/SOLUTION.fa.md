# پاسخ تشریحی

```rust
pub async fn transfer(pool: &PgPool, from: i64, to: i64, amount_cents: i64) -> Result<(), TransferError> {
    if amount_cents <= 0 { return Err(TransferError::AmountNotPositive(amount_cents)); }
    let mut tx = pool.begin().await?;
    let available: i64 = sqlx::query_scalar(
        "SELECT balance_cents FROM p3_04_04_accounts WHERE id = $1 FOR UPDATE",
    ).bind(from).fetch_optional(&mut *tx).await?.ok_or(TransferError::AccountNotFound(from))?;
    if available < amount_cents {
        return Err(TransferError::InsufficientFunds { available, requested: amount_cents });
    }
    sqlx::query("UPDATE p3_04_04_accounts SET balance_cents = balance_cents - $1 WHERE id = $2")
        .bind(amount_cents).bind(from).execute(&mut *tx).await?;
    let credited = sqlx::query("UPDATE p3_04_04_accounts SET balance_cents = balance_cents + $1 WHERE id = $2")
        .bind(amount_cents).bind(to).execute(&mut *tx).await?;
    if credited.rows_affected() == 0 { return Err(TransferError::AccountNotFound(to)); }
    tx.commit().await?;
    Ok(())
}
```

بعد از `begin` هیچ exit pathای rollback را صریح نمی‌نویسد. اگر `?`، `ok_or`، insufficient funds یا recipient گمشده از تابع خارج شود، `tx` بدون commit drop می‌شود و sqlx rollback می‌کند؛ همان نقش unlock-on-drop مربوط به `MutexGuard`، این بار برای `BEGIN`/`ROLLBACK`.

`&mut *tx` تضمین می‌کند هر statement روی همان connection transaction اجرا شود و borrow checker دو query هم‌زمان روی transaction را نمی‌پذیرد. بدون `FOR UPDATE` دو transfer ۶۰تایی ممکن است هر دو ۱۰۰ بخوانند و balance را منفی کنند. lock دومی را تا commit/rollback اول صبر می‌دهد؛ constraint `CHECK (balance_cents >= 0)` در migration آخرین دفاع database است.

sender با SELECT lockدار ثابت می‌شود؛ recipient با خود UPDATE و `rows_affected()`، بدون SELECT اضافه. single INSERT مربوط به create_account از قبل atomic است، پس transaction اضافی سودی ندارد. testها accountهای تازه با IDهای جدا می‌سازند و row مشترک drop/wipe نمی‌کنند؛ shared database به‌تنهایی دلیل serialکردن نیست. revert down migration را اجرا و record را حذف می‌کند؛ run بعدی up را دوباره apply می‌کند.

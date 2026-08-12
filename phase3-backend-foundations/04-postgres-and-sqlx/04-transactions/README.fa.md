# ۰۴.۴ — transactionها

از Alice debit و به Bob credit می‌کنی: دو `UPDATE`. اگر میان آن‌ها process بمیرد یا error early return کند، Alice پرداخت کرده و Bob نگرفته است. statement تکی از قبل atomic است؛ **transaction گروه statementها را atomic می‌کند**: همه commit می‌شوند یا هیچ‌کدام رخ نداده‌اند.

در Django:

```python
with transaction.atomic():
    alice.balance -= 250; alice.save()
    bob.balance += 250; bob.save()
```

## شکل transaction در sqlx

```rust
let mut tx = pool.begin().await?; // BEGIN
sqlx::query("UPDATE ...").execute(&mut *tx).await?;
tx.commit().await?; // COMMIT
```

`pool.begin()` یک connection را checkout و `BEGIN` می‌فرستد. Transaction مالک آن connection است، چون transaction روی یک connection معنا دارد. همه statementها باید با `&mut *tx` روی همان connection اجرا شوند؛ `&pool` می‌تواند connection دیگری بگیرد و query را خارج transaction auto-commit کند. `commit` صریح است؛ فراموشی آن corruption نمی‌سازد، بلکه چیزی persist نمی‌شود.

## rollback پیش‌فرض است

اگر `Transaction` بدون commit drop شود—`return`، `?` یا panic—sqlx پیش از بازگرداندن connection به pool `ROLLBACK` می‌فرستد. Django این تضمین را با exception خروجی از `with` می‌دهد؛ Rust با `Drop`. پس هر `?` پس از `begin` یک rollback point پنهان است.

## check-then-write به row lock نیاز دارد

دو transfer هم‌زمان می‌توانند balance ۱۰۰ بخوانند و هر دو ۶۰ را مجاز ببینند. `SELECT ... FOR UPDATE` تا پایان transaction row را lock می‌کند؛ دومی صبر و balance جدید را می‌بیند. constraint به‌شکل `CHECK (balance_cents >= 0)` backstop database است. مبلغ همیشه integer cents است، نه float.

## migration برگشت‌پذیر

```text
0001_create_accounts.up.sql    # run
0001_create_accounts.down.sql  # فقط sqlx migrate revert
```

`sqlx migrate revert` آخرین migration را با down file برمی‌گرداند و row آن را از `_sqlx_migrations` حذف می‌کند؛ invocation بعدی `run_migrations` up را دوباره apply می‌کند. یک directory نمی‌تواند `.sql` و `.up.sql`/`.down.sql` را mix کند.

```senpai-visual
{"kind":"database","labels":["BEGIN","debit","credit","COMMIT","خطا → ROLLBACK"]}
```

مثل انتقال پول در دو دفتر است: یا هر دو مهر می‌خورند یا هیچ‌کدام. مرز تشبیه: row lock فقط وقتی معنا دارد که در transaction scope داشته باشد.

## تمرین تو

`create_account` و `transfer` را با هفت گام doc comment کامل کن و testهای ignored را اجرا کن.

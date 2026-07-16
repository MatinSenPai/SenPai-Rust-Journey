# Checkpoint

1. In `a_failed_credit_rolls_back_the_debit`, the debit `UPDATE` really
   does execute against Postgres before the credit fails. Point to the
   exact line of your `transfer` where the rollback "happens" — trick
   question: there isn't one. Explain what actually undoes the debit, and
   when.
2. Django's `transaction.atomic()` rolls back when an exception escapes
   the `with` block. Rust has no exceptions — what two language/library
   mechanisms combine to give sqlx the equivalent guarantee, and why does
   that make `?` inside a transaction "secretly also a rollback point"?
3. Why does the balance check use `SELECT ... FOR UPDATE` instead of a
   plain `SELECT`? Describe the exact interleaving of two concurrent
   transfers that the lock prevents — and which line of the migration
   would still save you if the lock (or the check) were missing.
4. `create_account` deliberately runs *without* a transaction, while
   `transfer` requires one. What's the rule that decides, and why would
   wrapping `create_account`'s single `INSERT` in a transaction buy you
   nothing?
5. This lesson's migration is two files where 04.2's was one. What does
   `sqlx migrate revert` do with the `.down.sql` file (including to the
   `_sqlx_migrations` table), and what happens on the next
   `run_migrations` after a revert?
6. Lessons 04.2 and 04.3 needed `#[serial(...)]` on every DB test; this
   lesson's DB tests run concurrently with no serialization. What's
   different about how these tests use the database that makes that safe?

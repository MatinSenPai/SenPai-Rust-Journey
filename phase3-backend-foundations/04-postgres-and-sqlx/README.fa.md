# ماژول ۴ — PostgreSQL و `sqlx`

تا اینجا store از نوع `Mutex<HashMap<...>>` بود: درست و سریع، اما با پایان process نابود می‌شد. این ماژول آن را با PostgreSQL واقعی و `sqlx` عوض می‌کند. `sqlx` ORM کامل مانند Django نیست؛ SQL واقعی را تو می‌نویسی و library binding/decoding type-safe می‌دهد.

**راه‌اندازی:** یک PostgreSQL محلی با connection string لازم است. مقدار پیش‌فرض درس‌ها `postgres://taskforge:taskforge@localhost:5432/taskforge` است؛ همان role/database capstone. testهای database با `#[ignore]` جدا شده‌اند تا `cargo test` بدون infrastructure سبز باشد؛ برای اجرای واقعی از `cargo test -- --ignored` با `DATABASE_URL` استفاده کن.

۱. [اتصال و pooling](01-connecting-and-pooling/README.fa.md)
۲. [migrationها](02-migrations/README.fa.md)
۳. [catalog انیمه متصل به PostgreSQL](03-anime-catalog-postgres-backed/README.fa.md)
۴. [transactionها](04-transactions/README.fa.md): double-write، `pool.begin()`، `tx.commit()`، rollback-on-drop و migration برگشت‌پذیر.

# پاسخ تشریحی

```rust
pub async fn connect_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(database_url)
        .await
}

pub async fn health_check(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1").execute(pool).await.map(|_| ())
}
```

منطق کوتاه است اما configuration مهم است. وقتی پنج connection checkout شده‌اند، caller ششم روی `.fetch_one(&pool).await` داخل queue pool می‌ماند تا یکی برگردد. query رد یا drop نمی‌شود؛ فقط کندتر است. `acquire_timeout` مانع queue ابدی می‌شود.

`.connect()` eager باعث می‌شود URL بد در خود `connect_pool` دیده شود؛ testهای `connects_and_passes_a_health_check` و `a_bad_url_fails_to_connect_instead_of_hanging` روی همین رفتار تکیه دارند. بدون timeout، test URL بد با default داخلی sqlx ممکن است حدود ۳۰ ثانیه منتظر بماند.

۵۰۰ connection در هر replica concurrency نامحدود نمی‌سازد. Postgres `max_connections` خودش را دارد، هر connection memory مصرف می‌کند و چند replica می‌توانند سقف server را تمام کنند. در scale واقعی PgBouncer یا pooler managed مناسب‌تر از بالا بردن بی‌نهایت pool app است. `tokio::spawn` future با `'static` می‌خواهد و نمی‌تواند `&PgPool` stack-local را borrow کند؛ `.clone()` فقط Arc داخلی را clone می‌کند، نه connectionها را. این همان الگوی Arc/Mutex درس threadهای فاز دو است.

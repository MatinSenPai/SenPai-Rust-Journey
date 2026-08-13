# راه‌حل

```rust
pub async fn run_schema(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS p3_07_02_widgets (\
            id BIGSERIAL PRIMARY KEY, name TEXT NOT NULL, quantity INT NOT NULL)",
    )
    .execute(pool)
    .await
    .map(|_| ())
}

pub async fn create(&self, name: &str, quantity: i32) -> Result<Widget, sqlx::Error> {
    sqlx::query_as(
        "INSERT INTO p3_07_02_widgets (name, quantity) VALUES ($1, $2) \
         RETURNING id, name, quantity",
    )
    .bind(name)
    .bind(quantity)
    .fetch_one(&self.pool)
    .await
}

pub async fn get(&self, id: i64) -> Result<Option<Widget>, sqlx::Error> {
    sqlx::query_as("SELECT id, name, quantity FROM p3_07_02_widgets WHERE id = $1")
        .bind(id)
        .fetch_optional(&self.pool)
        .await
}

pub async fn list(&self) -> Result<Vec<Widget>, sqlx::Error> {
    sqlx::query_as("SELECT id, name, quantity FROM p3_07_02_widgets ORDER BY id")
        .fetch_all(&self.pool)
        .await
}
```

نکتهٔ درس setup است، نه پیچیدگی repository. `ContainerAsync<Postgres>` باید با نامی مثل `_container` در scope بماند: `_` اصلاً bind نمی‌شود و container فوراً Drop و خاموش می‌شود؛ بعد pool به database مرده وصل است. `_container` warning unused نمی‌دهد ولی عمر container را تا پایان test نگه می‌دارد.

هر test server کاملاً تازه‌ای می‌سازد، پس data اجرای قبلی وجود ندارد و cleanup لازم نیست. Docker برای containerهای موازی port آزاد جدا می‌دهد؛ bind همیشگی به ۵۴۳۲ باعث می‌شود test دوم نتواند شروع شود. `#[ignore]` فقط execution را حذف می‌کند؛ test binary و تمام APIهای `testcontainers`/`sqlx` همچنان compile می‌شوند. برای تبدیل `WidgetStore` درون‌حافظه‌ای به تست Postgres، ابتدا dependency را پشت trait repository ببر و implementationِ `sqlx` بساز؛ container داشتن به‌تنهایی store مبتنی بر `Mutex<HashMap>` را database-backed نمی‌کند.

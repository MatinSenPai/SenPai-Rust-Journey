# پاسخ تشریحی

```rust
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}

pub async fn insert_widget(pool: &PgPool, name: &str) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("INSERT INTO p3_04_02_widgets (name) VALUES ($1) RETURNING id")
        .bind(name).fetch_one(pool).await
}

pub async fn count_widgets(pool: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT COUNT(*) FROM p3_04_02_widgets")
        .fetch_one(pool).await
}
```

database بین test runها پاک نمی‌شود. اگر row 0001 قبلاً ثبت باشد، `run_migrations` table را دوباره نمی‌سازد و assertion «پس از migrate خالی است» بی‌معنا می‌شود. reset table و row tracking clean slate واقعی می‌دهد.

افزودن 0002 workflow درست است: 0001 skip می‌شود، 0002 اجرا و ثبت می‌شود. ویرایش 0001 اجراشده checksum را با record قبلی ناسازگار می‌کند و `VersionMismatch` می‌دهد؛ برای تغییر schema همیشه migration تازه بنویس، history را ویرایش نکن.

embed compile-time release را self-contained می‌کند: یک binary SQL migration خودش را همراه دارد و directory جدا یا خطر drift binary/file نداری. Django نیز table tracking دارد اما `makemigrations` از modelها migration تولید می‌کند؛ sqlx model registry برای diff ندارد و فقط SQL دستی را apply/track می‌کند.

`query_scalar` برای یک column در هر row است و مستقیم `i64` را decode می‌کند. `query_as` برای چند column و struct/tuple دارای `FromRow` است. database جدا به‌خاطر table مشترک `_sqlx_migrations` لازم است: version 1 دو crate checksumهای متفاوت دارند و prefix table داده، version tracking را namespace نمی‌کند. `#[serial]` باید روی همه‌ی testهای shareکننده‌ی database باشد؛ test بی‌tag هنوز می‌تواند هم‌زمان با test DROPکننده اجرا شود.

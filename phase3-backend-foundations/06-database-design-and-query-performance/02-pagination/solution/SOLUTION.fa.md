# راه‌حل

```rust
pub fn encode_cursor(cursor: Cursor) -> String {
    format!("{}:{}", cursor.created_at.timestamp_micros(), cursor.id)
}

pub fn decode_cursor(token: &str) -> Result<Cursor, CursorError> {
    let malformed = || CursorError::Malformed(token.to_string());
    let (ts, id) = token.split_once(':').ok_or_else(malformed)?;
    let ts: i64 = ts.parse().map_err(|_| malformed())?;
    let id: i64 = id.parse().map_err(|_| malformed())?;
    let created_at =
        DateTime::from_timestamp_micros(ts).ok_or(CursorError::TimestampOutOfRange(ts))?;
    Ok(Cursor { created_at, id })
}
```

```rust
pub async fn page_by_keyset(
    pool: &PgPool,
    after: Option<Cursor>,
    limit: i64,
) -> Result<Vec<Article>, sqlx::Error> {
    let rows: Vec<(i64, String, DateTime<Utc>)> = match after {
        Some(cursor) => sqlx::query_as(
            "SELECT id, title, created_at FROM p3_05_02_articles \
             WHERE (created_at, id) < ($1, $2) \
             ORDER BY created_at DESC, id DESC LIMIT $3",
        )
        .bind(cursor.created_at)
        .bind(cursor.id)
        .bind(limit)
        .fetch_all(pool)
        .await?,
        None => sqlx::query_as(
            "SELECT id, title, created_at FROM p3_05_02_articles \
             ORDER BY created_at DESC, id DESC LIMIT $1",
        )
        .bind(limit)
        .fetch_all(pool)
        .await?,
    };
    Ok(rows_to_articles(rows))
}
```

`page_by_offset` همان `SELECT` با `LIMIT $1 OFFSET $2` و بدون `WHERE` است.

`split_once(':')` هم ساختار را چک می‌کند؛ tokenی مثل `"1:2:3"` نیز چون بخش دوم `i64` نیست رد می‌شود. closureِ `malformed` سه مسیر خطا را یکدست نگه می‌دارد. `DateTime::from_timestamp_micros` صادقانه `Option` می‌دهد: همهٔ `i64`ها زمان قابل‌نمایش نیستند. token دست‌ساز یا خراب باید خطا شود، نه panic.

مقایسهٔ `(created_at, id) < ($1, $2)` دقیقاً این است:

```sql
created_at < $1 OR (created_at = $1 AND id < $2)
```

همان ترتیب در `ORDER BY` باعث می‌شود «پس از cursor در ترتیب پیمایش» یک شرط واحد باشد. ایندکس `(created_at, id)` هم `WHERE` و هم `ORDER BY` را با یک backward index scan پاسخ می‌دهد: `O(limit)` در برابر `O(offset + limit)`.

در تست drift، offset می‌گوید «دو ردیف از بالای جدولِ فعلی رد شو»؛ insert تعریفِ بالا را عوض می‌کند و `c` دوباره ظاهر می‌شود. cursor می‌گوید «قدیمی‌تر از `c`»، پس insert بالای `c` اثری ندارد. حذف، برای offset یک ردیف را رد می‌کند؛ keyset فقط همان ردیف حذف‌شده را دیگر نمی‌بیند.

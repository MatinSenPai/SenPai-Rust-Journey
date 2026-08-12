# پاسخ تشریحی

```rust
pub async fn create(&self, input: CreateAnime) -> Result<Anime, AnimeError> {
    validate_rating(input.rating)?;
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO p3_04_03_anime (title, status, rating) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(&input.title)
    .bind(input.status.as_str())
    .bind(input.rating)
    .fetch_one(&self.pool).await
    .map_err(|e| AnimeError::Storage(e.to_string()))?;
    Ok(Anime { id, title: input.title, status: input.status, rating: input.rating })
}

pub async fn get(&self, id: i64) -> Result<Anime, AnimeError> {
    let row: Option<(i64, String, String, Option<i16>)> =
        sqlx::query_as("SELECT id, title, status, rating FROM p3_04_03_anime WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| AnimeError::Storage(e.to_string()))?;
    let (id, title, status, rating) = row.ok_or(AnimeError::NotFound)?;
    let status = WatchStatus::parse(&status).map_err(AnimeError::Storage)?;
    Ok(Anime { id, title, status, rating })
}
```

`list` همان `get` بدون `WHERE` است و rowها را با `collect::<Result<Vec<_>, _>>()` map می‌کند تا نخستین status نامعتبر را propagate کند. `update` و `delete` اول `get` می‌زنند، پس وجود record را ثابت و `NotFound` را یکجا مدیریت می‌کنند.

هر method نسبت به نسخه‌ی HashMap یک `await` و تبدیل `sqlx::Error` به `AnimeError::Storage` دارد. Postgres enum Rust را نمی‌شناسد، پس conversion boundary صریح است. حتی اگر امروز این crate تنها writer باشد، migration آینده، `psql` دستی یا service دوم می‌تواند status نامعتبر بسازد؛ Result به‌جای panic یک ۵۰۰ قابل‌تشخیص می‌دهد.

`ORDER BY id` sorting را جایی می‌گذارد که database index و query plan را می‌شناسد و pagination را هم با ترتیب قطعی همسو می‌کند. update فعلی read-then-write است؛ دو request می‌توانند state قدیمی یکسان ببینند و دومی write اول را overwrite کند. `SELECT ... FOR UPDATE` در transaction یا `UPDATE ... WHERE version = $old_version` همراه `rows_affected()` این race را می‌بندد. tag serial مشترک نیز exclusion را میان هر دو binary تست برقرار می‌کند.

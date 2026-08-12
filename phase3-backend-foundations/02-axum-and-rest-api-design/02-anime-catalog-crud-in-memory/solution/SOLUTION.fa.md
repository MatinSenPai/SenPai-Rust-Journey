# پاسخ تشریحی

```rust
pub fn create(&self, input: CreateAnime) -> Result<Anime, AnimeError> {
    validate_rating(input.rating)?;
    let mut inner = self.inner.lock().unwrap();
    inner.next_id += 1;
    let id = inner.next_id;
    let anime = Anime { id, title: input.title, status: input.status, rating: input.rating };
    inner.items.insert(id, anime.clone());
    Ok(anime)
}
```

validation پیش از گرفتن lock انجام می‌شود؛ input نامعتبر نباید requestهای دیگر را معطل کند. افزایش سپس خواندن `next_id` باعث می‌شود ID از ۱ شروع شود.

```rust
pub fn update(&self, id: u64, input: UpdateAnime) -> Result<Anime, AnimeError> {
    validate_rating(input.rating)?;
    let mut inner = self.inner.lock().unwrap();
    let anime = inner.items.get_mut(&id).ok_or(AnimeError::NotFound)?;
    if let Some(title) = input.title { anime.title = title; }
    if let Some(status) = input.status { anime.status = status; }
    if input.rating.is_some() { anime.rating = input.rating; }
    Ok(anime.clone())
}
```

`get_mut` یک `&mut Anime` از داخل `HashMap` می‌دهد، پس fieldها درجا تغییر می‌کنند. `Option<u8>` از `Copy` است و assignment کل Option ساده است.

```rust
pub async fn create_anime(
    State(store): State<Arc<AnimeStore>>,
    Json(input): Json<CreateAnime>,
) -> Result<(StatusCode, Json<Anime>), AnimeError> {
    let anime = store.create(input)?;
    Ok((StatusCode::CREATED, Json(anime)))
}
```

`?` مستقیم `AnimeError` را propagate می‌کند. tuple به‌کمک `IntoResponse` status پیش‌فرض `200 OK` مربوط به `Json<Anime>` را به `201 Created` تبدیل می‌کند.

```rust
pub fn app(store: Arc<AnimeStore>) -> Router {
    Router::new()
        .route("/anime", get(list_anime).post(create_anime))
        .route("/anime/{id}", get(get_anime).patch(update_anime).delete(delete_anime))
        .with_state(store)
}
```

تمام منطق در `AnimeStore` است؛ اگر در handler می‌بود هر validation test به request کامل و `oneshot` نیاز داشت. برای patch سه‌حالته می‌توان `Option<Option<u8>>` یا `enum Patch<T> { Unset, SetTo(T), Clear }` داشت. با derive عادی، serde absent و null را collapse می‌کند؛ برای double option باید `serde_with::rust::double_option` یا `Deserialize` دستی به‌کار رود. success envelope به wrapper جنریک `ApiResponse<T>` در همه handlerها یا middleware نیاز دارد، اما errorها از قبل یک type مشترک دارند. یک Mutex مشترک هم increment+insert را atomic می‌کند؛ دو lock می‌توانند دو create را با ID یکسان interleave و record اول را overwrite کنند.

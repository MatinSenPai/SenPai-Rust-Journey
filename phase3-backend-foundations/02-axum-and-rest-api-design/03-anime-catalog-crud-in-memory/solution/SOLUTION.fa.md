# راه‌حل — ۳.۲.۳ عملیاتِ CRUD روی کاتالوگِ انیمه (در حافظه)

کدِ کامل `solution/src/lib.rs` است؛ همه‌ی تست‌هایِ `solution/tests/` را پاس می‌کند، از جمله `build_test.rs` برایِ پله‌ی «بساز».

## `AnimeError::into_response`

```rust
fn into_response(self) -> Response {
    let (status, message) = match self {
        AnimeError::NotFound => (StatusCode::NOT_FOUND, "anime not found".to_string()),
        AnimeError::InvalidRating(r) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            format!("rating must be between 1 and 10, got {r}"),
        ),
    };
    (status, Json(serde_json::json!({ "error": message }))).into_response()
}
```

`match` برایِ هر واریانت یک جفتِ `(status, message)` انتخاب می‌کند، و خطِ آخر توپلِ `(StatusCode, Json<Value>)` می‌سازد که از قبل `IntoResponse` دارد. `Json` هدرِ `Content-Type: application/json` را هم می‌گذارد. این تنها جایِ برنامه است که عبارتِ «not found» و عددِ `404` به هم می‌رسند. رتبه‌ی بد `422` است، نه `400`، چون درخواست پارس شد و محتوا یک قاعده را شکست (۳.۱.۳).

## ذخیره‌گاه

```rust
pub fn create(&self, input: CreateAnime) -> Result<Anime, AnimeError> {
    validate_rating(input.rating)?;
    let mut inner = self.inner.lock().unwrap();
    inner.next_id += 1;
    let anime = Anime { id: inner.next_id, title: input.title,
                        status: input.status, rating: input.rating };
    inner.items.insert(anime.id, anime.clone());
    Ok(anime)
}
```

اعتبارسنجی پیش از قفل می‌آید، به دو دلیل: درخواستِ ردشده هیچ‌وقت منتظرِ mutex نمی‌ماند، و هیچ شناسه‌ای را مصرف نمی‌کند (`a_rejected_create_does_not_use_up_an_id`). شناسه *پیش از* خوانده‌شدن زیاد می‌شود، پس از `1` شروع می‌شود. بعد از حذف هم تکرار نمی‌شود، چون `next_id` فقط بالا می‌رود و از اندازه‌ی نقشه گرفته نمی‌شود (`ids_are_never_reused_after_a_delete`).

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

مشخصات می‌گوید رتبه اول چک می‌شود، پس `update(999, rating: 20)` می‌شود `InvalidRating` و نه `NotFound` (`update_checks_the_rating_before_the_id`). `get_mut` یک `&mut Anime` داخلِ نقشه می‌دهد، پس فیلدها همان‌جا رونویسی می‌شوند. تا هر دو چک رد نشده چیزی تغییر نمی‌کند، پس هر `Err` ذخیره‌گاه را دست‌نخورده می‌گذارد.

`get` همان `.get(&id).cloned().ok_or(NotFound)` است، `delete` همان `.remove(&id).ok_or(NotFound)`، و `list` مقدارها را جمع می‌کند و بعد با `sort_by_key` رویِ شناسه مرتب می‌کند. `HashMap` ترتیبی نمی‌دهد، و تستِ بیست‌تایی یک لیستِ مرتب‌نشده را می‌گیرد.

هر متد `MutexGuard` را فقط تا برگشتنش نگه می‌دارد. ذخیره‌گاه همگام است، پس گارد هرگز در لحظه‌ی یک `.await` زنده نیست.

## هندلرها و جدولِ مسیر

```rust
pub async fn create_anime(State(store): State<Arc<AnimeStore>>, Json(input): Json<CreateAnime>)
    -> Result<(StatusCode, [(HeaderName, String); 1], Json<Anime>), AnimeError>
{
    let anime = store.create(input)?;
    let location = format!("/anime/{}", anime.id);
    Ok((StatusCode::CREATED, [(header::LOCATION, location)], Json(anime)))
}
```

اینجا `?` چیزی را تبدیل نمی‌کند: نوعِ خطایِ ذخیره‌گاه همان نوعِ خطایِ هندلر است. توپلِ سه‌قسمتی `(StatusCode, هدرها, بدنه)` است و `axum` برایش `IntoResponse` پیاده کرده. چهار هندلرِ دیگر هرکدام یک خط‌اند: `Json(store.list())`، `store.get(id).map(Json)` (یا `?` و `Ok(Json(..))`)، همین برایِ update، و `store.delete(id)?; Ok(StatusCode::NO_CONTENT)`.

```rust
Router::new()
    .route("/anime", get(list_anime).post(create_anime))
    .route("/anime/{id}", get(get_anime).patch(update_anime).delete(delete_anime))
    .with_state(store)
```

## بساز: `PUT`

`AnimeStore::replace(id, CreateAnime)` رتبه را اعتبارسنجی می‌کند، بعد شناسه را با `get_mut(...).ok_or(NotFound)?` پیدا می‌کند و کلِ مدخل را با `*anime = Anime { id, .. }` رونویسی می‌کند. هندلر همان شکلِ `update_anime` را دارد، با `Json<CreateAnime>`، و `.put(replace_anime)` به مسیرِ `/anime/{id}` زنجیر می‌شود. `PUT` رویِ شناسه‌ی گم `404` است و چیزی نمی‌سازد، چون شناسه‌ها مالِ سرورند. بدنه‌ای که یک فیلدِ لازم را ندارد اصلاً به هندلر نمی‌رسد: `Json` با `422` ردش می‌کند. همان `PUT` دو بار همان وضعیت، همان بدنه و همان حالت را می‌دهد، و `the_same_put_twice_gives_the_same_state_and_the_same_answer` هر سه را چک می‌کند. این همان خودتوانیِ `PUT` است که جدولِ ۳.۱.۳ قولش را می‌دهد.

## چالش: `If-Match`

یک شکلِ قابلِ‌اجرا: `version: u64` به `Anime` اضافه کن (در create برابرِ `1`، و با هر update یا replaceِ موفق `+= 1`)؛ `AnimeError::VersionMismatch` و یک بازوی `412 Precondition Failed` اضافه کن؛ به `update_anime` یک اکسترکتورِ `HeaderMap` بده، `If-Match` را به‌صورتِ `u64` پارس کن و به `update` بده، که پیش از هر تغییر آن را با نسخه‌ی ذخیره‌شده *داخلِ همان قفل* مقایسه می‌کند. چک و نوشتن باید زیرِ یک بار گرفتنِ قفل باشند. اگر دو فراخوانیِ جدا بودند، درخواستِ دیگری می‌توانست وسطشان بپرد، و دوباره به مشکلِ به‌روزرسانیِ گم‌شده برمی‌گشتی.

## درباره‌ی گرم‌کردن‌ها و خطاها

- **چرا `Result<Json<Anime>, AnimeError>` بازگشتِ مجاز است؟** چون `Result<T, E>: IntoResponse` وقتی هر دو نیمه آن را دارند. `impl IntoResponse for AnimeError`ِ خودت را بردار تا همان `E0277`ِ `examples/03-error-without-into-response-broken.rs` را ببینی.
- **تعمیرِ `MutexGuard`.** مثالِ ۰۲ و ۰۴: قفل را در یک بلوک بپیچ که مقدارِ لازم را برمی‌گرداند (`let now = { let mut g = ...; *g += 1; *g };`)، همان‌طور که `examples/06-guard-dropped-before-await-fix.rs` می‌کند. `drop(guard)` پیش از `.await` هم کار می‌کند، ولی شکلِ بلوک را نمی‌شود فراموش کرد. `tokio::sync::Mutex` هم کامپایل می‌شود، و فقط وقتی ابزارِ درست است که واقعاً باید قفل را از `.await` عبور بدهی.
- **مثالِ ۰۳.** `impl IntoResponse for ShowError` را با `(StatusCode::NOT_FOUND, "no such show").into_response()` اضافه کن.
- **مثالِ ۰۵.** `Ok((StatusCode::CREATED, Json(..)))` برگردان. نوعِ بازگشتیِ هندلر هم باید همراهش عوض شود، پس `use axum::http::StatusCode;` همان import است که لازم داری.

## این درس واقعاً درباره‌ی چه بود

هیچ‌کدام از کدها بلند نیست. آنچه مهم است این است که هر تصمیم کجا می‌نشیند: قاعده‌ها در ذخیره‌گاه، واژگانِ کدِ وضعیت در یک `IntoResponse`، و معناهایِ متد (`201`، `204`، `PATCH` در برابرِ `PUT`) در هندلرها و جدولِ مسیر. وقتی ماژولِ ۵ `HashMap` را با Postgres عوض می‌کند، فقط ذخیره‌گاه عوض می‌شود.

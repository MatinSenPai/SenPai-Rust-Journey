# ۰۲.۲ — CRUD catalog انیمه در حافظه

## ساختن یک resource واقعی از قطعات درس قبل

routeهای درس قبل demoهای مستقل بودند. این درس شکل معمول یک REST resource است: **یک struct، پنج operation و یک جدول route**؛ همان چیزی که DRF با `ModelViewSet` scaffold می‌کند، اما این بار دستی می‌سازی تا هر قطعه را ببینی.

| DRF روی model `Anime` | این درس |
|---|---|
| `GET /anime/` و `.list()` | `GET /anime` و `list_anime` |
| `POST /anime/` و `.create()` | `POST /anime` و `create_anime` |
| `GET /anime/{id}/` و `.retrieve()` | `GET /anime/{id}` و `get_anime` |
| `PATCH /anime/{id}/` و `.partial_update()` | `PATCH /anime/{id}` و `update_anime` |
| `DELETE /anime/{id}/` و `.destroy()` | `DELETE /anime/{id}` و `delete_anime` |
| `Anime.objects` | `AnimeStore`؛ فعلاً حافظه و ماژول چهار Postgres |

store واقعاً in-memory است: `HashMap` پشت `Mutex` که با خروج process از بین می‌رود. این عمدی است؛ در ماژول چهار همین شکل را به Postgres وصل می‌کنی تا مرز HTTP/REST و persistence را جدا ببینی. routeها، request/responseها و error handling تقریباً ثابت می‌مانند.

## منطق خالص و لبه‌ی HTTP نازک

`AnimeStore` چیزی از `axum`، `Json` یا HTTP status code نمی‌داند. یک `HashMap<u64, Anime>` پشت `Mutex` با متدهای CRUD است و `Result<Anime, AnimeError>` می‌دهد. `tests/store_test.rs` مستقیم همان منطق را test می‌کند. handler فقط extractor را باز می‌کند، store را صدا می‌زند و Result را response می‌کند. `tests/api_test.rs` کل stack را با `oneshot` می‌راند.

## یک route و چند method

```rust
Router::new()
    .route("/anime", get(list_anime).post(create_anime))
    .route("/anime/{id}", get(get_anime).patch(update_anime).delete(delete_anime))
```

`get(handler)` و همتایانش یک `MethodRouter` می‌سازند و chainشدنشان چند verb را به یک path متصل می‌کند. این همان چندverbی بودن route `ModelViewSet` است، فقط explicit.

## تبدیل domain error به HTTP response

```rust
pub enum AnimeError {
    NotFound,
    InvalidRating(u8),
}

impl IntoResponse for AnimeError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AnimeError::NotFound => (StatusCode::NOT_FOUND, "anime not found".to_string()),
            AnimeError::InvalidRating(r) => (
                StatusCode::BAD_REQUEST,
                format!("rating must be between 1 and 10, got {r}"),
            ),
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}
```

پیاده‌سازی `IntoResponse` اجازه می‌دهد handler مستقیماً `Result<Json<Anime>, AnimeError>` بدهد؛ `axum` خودش شاخه‌ی `Ok` یا `Err` را response می‌کند. همه‌ی errorها از همین حالا شکل `{"error": "..."}` دارند؛ پیش‌درآمد envelopeهای یکدست ماژول هفت.

```senpai-visual
{"kind":"database","labels":["HTTP handler","AnimeStore","Mutex<HashMap>","Anime","JSON response"]}
```

این store را مثل دفتر موقت کتابخانه ببین. مرز تشبیه: `Mutex` فقط integrity حافظه را نگه می‌دارد؛ durability، query و index database واقعی ندارد.

## تمرین تو

منطق CRUD `AnimeStore`، پنج handler و `app` را کامل کن. سپس با POST/PATCH/DELETE زیر API را امتحان کن:

```sh
cargo run -p p3-02-02-anime-catalog-crud-in-memory &
curl -X POST -H 'content-type: application/json' -d '{"title":"Frieren","status":"watching","rating":9}' http://127.0.0.1:3001/anime
curl -X PATCH -H 'content-type: application/json' -d '{"status":"completed"}' http://127.0.0.1:3001/anime/1
```

## ایست بازرسی

اول `CHECKPOINT.fa.md` و سپس `solution/SOLUTION.fa.md` را بخوان.

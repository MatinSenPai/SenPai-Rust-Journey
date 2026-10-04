# راه‌حل — ۳.۲.۱ مسیریابی، هندلرها، اکسترکتورها

## `greet`

```rust
pub async fn greet(Path(name): Path<String>) -> String {
    format!("Hello, {name}!")
}
```

استخراج در امضا انجام شده. `Path(name)` پوسته‌ی `Path<String>` را باز می‌کند، پس بدنه فقط قالب می‌زند.

## `echo`

```rust
pub async fn echo(Json(payload): Json<EchoRequest>) -> Json<EchoResponse> {
    let length = payload.message.len();
    Json(EchoResponse { message: payload.message, length })
}
```

`length` پیش از آن خوانده می‌شود که `payload.message` به پاسخ منتقل شود: اول اندازه بگیر، بعد واگذار کن. `str::len` بایت می‌شمارد، برایِ همین مشخصات می‌گوید بایت (یک حرفِ فارسی دو بایت است).

## `get_counter` و `increment_counter`

```rust
pub async fn get_counter(State(state): State<AppState>) -> Json<CounterResponse> {
    let count = *state.counter.lock().unwrap();
    Json(CounterResponse { count })
}

pub async fn increment_counter(State(state): State<AppState>) -> Json<CounterResponse> {
    let mut guard = state.counter.lock().unwrap();
    *guard += 1;
    Json(CounterResponse { count: *guard })
}
```

`.lock().unwrap()` فقط وقتی شکست می‌خورد که ریسمانِ دیگری در حینِ نگه‌داشتنِ قفل پنیک کرده باشد (mutexِ مسموم)؛ برایِ یک شمارنده‌ی درون‌حافظه‌ای این کرش قابلِ‌قبول است. `CounterResponse` یک `i64`ِ ساده را که از گارد کپی شده نگه می‌دارد، پس گارد در پایانِ تابع drop می‌شود. تا وقتی قفل در دست است هیچ `.await`ای نیست، و همین باعث می‌شود یک `std::sync::Mutex` اینجا مشکلی نداشته باشد.

## `app`

```rust
pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/", get(hello))
        .route("/greet/{name}", get(greet))
        .route("/echo", post(echo))
        .route("/counter", get(get_counter))
        .route("/counter/increment", post(increment_counter))
        .with_state(state)
}
```

هر `.route` روتر را مصرف می‌کند و برمی‌گرداند، پس کلِ جدول یک عبارت است. اگر `.with_state(state)` را جا بیندازی `E0308`ِ بخشِ «خطاهایی که خواهی دید» را می‌گیری.

## بساز: `search`

```rust
#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub q: String,
    pub limit: Option<u32>,
}

pub async fn search(Query(params): Query<SearchParams>) -> Json<SearchResponse> {
    Json(SearchResponse { q: params.q, limit: params.limit.unwrap_or(10) })
}
```

`q: String` آن را اجباری و `limit: Option<u32>` اختیاری می‌کند، پس `400`هایِ نبودنِ `q` یا `limit=many` را خودِ اکسترکتور می‌دهد. پیش‌فرض در هندلر با `unwrap_or(10)` اعمال می‌شود. با `.route("/search", get(search))` ثبتش کن؛ `SearchResponse` یک ساختارِ `Serialize` با دو فیلدِ `q` و `limit` است.

## چالش

```rust
pub async fn echo(payload: Result<Json<EchoRequest>, JsonRejection>) -> Response {
    match payload {
        Ok(Json(p)) => {
            let length = p.message.len();
            Json(EchoResponse { message: p.message, length }).into_response()
        }
        Err(rejection) => (rejection.status(), "bad echo request").into_response(),
    }
}
```

پیچیدنِ یک اکسترکتور در `Result` باعث می‌شود به‌جایِ جواب‌دادن، ردکننده‌اش را به تو بدهد. هر دو بازو به `Response` تبدیل می‌شوند تا یک نوع داشته باشند.

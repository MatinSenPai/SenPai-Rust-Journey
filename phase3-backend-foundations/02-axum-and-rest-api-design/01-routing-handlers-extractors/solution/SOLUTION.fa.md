# پاسخ تشریحی

```rust
pub async fn greet(Path(name): Path<String>) -> String {
    format!("Hello, {name}!")
}
```

پیش از اجرای این خط، `axum` مقدار `{name}` را از URL بیرون کشیده و به‌صورت `String` دارای مالکیت آماده کرده است. تمام extractor در signature رخ می‌دهد، نه در body.

```rust
pub async fn echo(Json(payload): Json<EchoRequest>) -> Json<EchoResponse> {
    let length = payload.message.len();
    Json(EchoResponse { message: payload.message, length })
}
```

پیش از moveکردن `payload.message` به response، طولش را با borrow می‌گیریم. پس از move دیگر نمی‌توانی `.len()` را بخوانی؛ همان ترتیب اندازه‌گیری‌پیش‌از‌واگذاری فاز مالکیت است.

```rust
pub async fn get_counter(State(state): State<AppState>) -> Json<CounterResponse> {
    let count = *state.counter.lock().unwrap();
    Json(CounterResponse { count })
}

pub async fn increment_counter(State(state): State<AppState>) -> Json<CounterResponse> {
    let mut guard = state.counter.lock().unwrap();
    *guard += 1;
    let count = *guard;
    drop(guard);
    Json(CounterResponse { count })
}
```

`Mutex::lock()` فقط با poisoned mutex، یعنی panic thread دیگر هنگام داشتن lock، `Err` می‌دهد. `unwrap` برای counter آموزشی قابل‌قبول است؛ production باید سیاست recovery روشن داشته باشد. binding میانی `count` یک `i64` مستقل می‌سازد و `drop(guard)` لحظه‌ی آزادشدن lock را واضح می‌کند؛ نگه‌داشتن lock بیش از نیاز، زیر بار concurrent bottleneck می‌شود.

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

builder chain router را در یک expression می‌سازد. در پاسخ checkpoint: `Json<T>` تضمین نوعی می‌دهد که body فقط با T parse‌شده وارد handler شود؛ در DRF باید `is_valid()`، branch و `Response(serializer.errors, status=400)` را دستی بنویسی. clone `AppState` فقط `Arc` را clone و reference count را زیاد می‌کند؛ همه به همان `Mutex<i64>` اشاره دارند. database queryهایی مانند `fetch_one(&pool).await` thread Tokio را هنگام انتظار آزاد می‌کنند. `Path(name)` نیز destructuring مربوط به tuple struct `Path<String>` است. route ناشناخته در هر دو framework خودکار 404 است؛ `axum` برای method اشتباه هم خودکار 405 می‌دهد، در Django class-based view مشابه است اما function-based view باید `request.method` را دستی branch کند.

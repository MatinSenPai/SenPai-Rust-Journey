# ۰۷.۱ — پوشش‌های خطای یکدست

## مسئله: هر handler شکل خطای خودش را اختراع می‌کند

در درس‌های قبلی یک ۴۰۴ شاید `{"error":"anime not found"}` بوده باشد. همان فایل شش ماه و سی endpoint بعد، با چهار نویسنده، به `{"error":"not found"}`، `{"message":"Not Found"}` و `{"errors":["bad rating"]}` تبدیل می‌شود. هیچ‌کدام به‌تنهایی غلط نیستند؛ اما frontend دیگر یک error handler ندارد، generator نمی‌تواند schema را بفهمد و پشتیبان نمی‌تواند یکنواخت `jq '.error.code'` بزند.

راه‌حل ارزان است: شکل را یک‌بار تعیین کن، یک‌جا بساز و عبور از آن را از نظر ساختاری سخت کن.

```senpai-visual
{"kind":"result","labels":["Domain failure","ApiError","error.code پایدار","پیام انسانی"]}
```

## شکل قرارداد

```json
{
  "error": {
    "code": "not_found",
    "message": "widget 42 not found"
  }
}
```

`code` برای ماشین و پایدار است: frontend می‌تواند بر اساس `validation_failed` رفتار کند، بی‌آنکه جملهٔ انگلیسی را parse کند. `message` برای انسان، log و API explorer است و می‌تواند بازنویسی یا ترجمه شود. خلاصه: **`code` قرارداد است، `message` ادبِ پاسخ**.

## یک enum، یک `IntoResponse`

```rust
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        // match self -> (StatusCode, code, message) -> one Json<ErrorBody>
    }
}
```

`thiserror::Error` به `ApiError`، `Display` و `std::error::Error` می‌دهد؛ برای logging و `?` مفید است. اما `Display` جواب HTTP نمی‌سازد. `axum::response::IntoResponse` مسئلهٔ مستقلی است: یک‌بار mappingِ خطا به `StatusCode` و `Json<ErrorBody>` را می‌نویسی و تمام handlerها فقط `Result<T, ApiError>` برمی‌گردانند.

این همان اصلِ Rust است: شکست را value کن و تبدیل آن به زبانِ caller را در یک مرز متمرکز کن. در منطق خالص `Result<T, DomainError>` است، در gRPC ممکن است `tonic::Status` باشد، و اینجا `ApiError`.

## خطاهای validation

`CreateWidget` از `validator::Validate` استفاده می‌کند. `input.validate()` یک `ValidationErrors` ساخت‌یافته می‌دهد، نه یک `String`. با `impl From<ValidationErrors> for ApiError`، handlerها فقط `input.validate()?` می‌نویسند؛ boilerplateِ `.map_err(...)` در هر handler تکثیر نمی‌شود.

`WidgetStore` یک `HashMap` پشت `Mutex` و دو handler دارد: `POST /widgets` برای validationِ نام و quantity، و `GET /widgets/{id}` برای `404`. این crate کوچک مصرف‌کنندهٔ دیگری برای store ندارد، پس خودِ `ApiError` می‌تواند error domain هم باشد. در پروژه‌ای که core را worker، CLI و HTTP هم‌زمان استفاده می‌کنند، error domain جدا و conversion در مرز HTTP انتخاب سالم‌تری است.

## تمرین

در `src/lib.rs` `IntoResponse`، conversion از `ValidationErrors`، `WidgetStore::create`/`get`، دو handler و `app` را کامل کن.

```sh
cargo run -p p3-07-01-consistent-error-envelopes &
curl -X POST -H 'content-type: application/json' \
  -d '{"name":"gizmo","quantity":12}' http://127.0.0.1:3002/widgets
curl http://127.0.0.1:3002/widgets/999
```

## قدم بعدی

`cargo test -p p3-07-01-consistent-error-envelopes`، سپس `solution/SOLUTION.fa.md`.

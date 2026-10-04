# راه‌حل — ۳.۸.۱ پاکت‌هایِ خطایِ یکدست

کدِ کامل `solution/src/lib.rs` است؛ همه‌یِ تست‌هایِ `solution/tests/` را می‌گذراند، از جمله `conflict_test.rs` برایِ پله‌یِ «بساز» و `problem_test.rs` برایِ «چالش».

## `status`، `code` و `message`

```rust
pub fn status(&self) -> StatusCode {
    match self {
        ApiError::NotFound(_) => StatusCode::NOT_FOUND,
        ApiError::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
        ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
        ApiError::UnsupportedMediaType(_) => StatusCode::UNSUPPORTED_MEDIA_TYPE,
        ApiError::InvalidBody(_) | ApiError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
        ApiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
```

`code` همین شکل است با رشته‌ها. هر دو `match` هستند و **wildcard ندارند**، پس یک گونه‌یِ تازه تا وقتی هرکدام جواب ندهد خطایِ کامپایل است (`E0004`، در درس نشان داده شد). `InvalidBody` و `Validation` هر دو `422` دارند ولی `code`ِ متفاوت: کدِ وضعیت ردهِ HTTPِ مشکل است، `code` نوعِ دقیقش.

```rust
pub fn message(&self) -> String {
    match self {
        ApiError::NotFound(m) | ApiError::BadRequest(m)
        | ApiError::UnsupportedMediaType(m) | ApiError::InvalidBody(m) => m.clone(),
        ApiError::MethodNotAllowed => "method not allowed for this route".to_string(),
        ApiError::Validation(_) => "the request body has invalid fields".to_string(),
        ApiError::Internal(_) => "something went wrong on our side".to_string(),
    }
}
```

بازوی `Internal(_)` کلِ داستانِ امنیت است: گونه صاحبِ متنِ سرور است و این تابع هرگز آن را برنمی‌گرداند. `an_internal_message_never_contains_the_inner_text` یک رمز را در متن می‌گذارد و می‌سنجد که نیست.

## `IntoResponse`

```rust
fn into_response(self) -> Response {
    if let ApiError::Internal(detail) = &self {
        eprintln!("internal error: {detail}");
    }
    let (status, code, message) = (self.status(), self.code(), self.message());
    let fields = match self { ApiError::Validation(f) => f, _ => Vec::new() };
    let body = ErrorBody { error: ErrorDetail { code, message, fields } };
    (status, Json(body)).into_response()
}
```

سه پرسش را متدها جواب می‌دهند و impl فقط سرِ هم می‌کند. خطِ لاگ اول می‌آید، تا `self` هنوز کامل است: `match self`ای که فهرستِ فیلدها را بیرون می‌برد آخر می‌آید. `ErrorDetail` رویِ `fields` اتریبیوتِ `#[serde(skip_serializing_if = "Vec::is_empty")]` دارد، پس فقط یک شکستِ اعتبارسنجی که ورودی دارد این کلید را می‌آورد. `Json` هدرِ `Content-Type: application/json` را می‌گذارد، چیزی که تست‌ها رویِ هر شکست می‌سنجند. (یک `Validation(vec![])`ِ خالی هم `fields` را حذف می‌کرد؛ کد هرگز چنین چیزی نمی‌سازد.)

## rejectionها

```rust
impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        let message = rejection.body_text();
        match rejection.status() {
            StatusCode::UNSUPPORTED_MEDIA_TYPE => ApiError::UnsupportedMediaType(message),
            StatusCode::UNPROCESSABLE_ENTITY => ApiError::InvalidBody(message),
            _ => ApiError::BadRequest(message),
        }
    }
}
```

`body_text()` همان متنی است که خودِ `axum` می‌فرستاد، پس پیام به‌اندازه‌یِ قبل آموزنده می‌ماند. match‌کردن رویِ `status()` و نه رویِ گونه‌هایِ rejection کار می‌کند حتی اگر `axum` گونه‌ای اضافه کند. wildcardِ `_` اینجا عمدی و بی‌خطر است: rejectionای که پیش‌بینی نکرده‌ایم مشکلِ درخواست است، پس `400`. `PathRejection` همین است با یک پیچ: یک `500` از `axum` یعنی *روتر* غلط پیکربندی شده (پارامترهایِ مسیر نیست، مثلاً چون `Path` در هندلری به کار رفته که هیچ routeِ دارایِ پارامترِ مسیر به آن نمی‌رسد)، که هرگز تقصیرِ کلاینت نیست، پس `Internal` می‌شود.

## `validate_new_show`

```rust
let mut fields = Vec::new();
if !(1..=100).contains(&input.title.chars().count()) {
    fields.push(FieldError { field: "title", code: "length",
        message: "title must be 1 to 100 characters".to_string() });
}
if !(1..=2000).contains(&input.episodes) {
    fields.push(FieldError { field: "episodes", code: "range",
        message: "episodes must be 1 to 2000".to_string() });
}
if fields.is_empty() { Ok(()) } else { Err(ApiError::Validation(fields)) }
```

هر دو بررسی همیشه اجرا می‌شوند، پس هر قاعده‌یِ شکسته با هم گزارش می‌شود، و `title` اول است چون اول push می‌شود. `chars().count()` و نه `len()`: `len()` بایت می‌شمارد، و یک عنوانِ فارسیِ ۱۰۰ حرفی ۲۰۰ بایت است (`title_length_counts_characters_not_bytes`).

## بساز: `Conflict`

اضافه‌کردنِ `ApiError::Conflict(String)` با `409` و `"conflict"`، و یک بازوی `message` که متن را برمی‌گرداند، چهار ویرایشِ کوچک است، و کامپایلر همه را فهرست می‌کند. بررسی به ذخیره‌گاه تعلق دارد، زیرِ همان قفلِ درج:

```rust
let mut inner = self.inner.lock().unwrap();
if inner.items.values().any(|s| s.title == input.title) {
    return Err(ApiError::Conflict(format!("a show titled \"{}\" already exists", input.title)));
}
inner.next_id += 1;
```

اگر هندلر از ذخیره‌گاه بپرسد «این عنوان هست؟» و بعد جداگانه درج کند، دو درخواستِ هم‌زمان می‌توانند هر دو «نه» ببینند و هر دو درج کنند. یک قفل، یک بررسی-و-درج، این را می‌بندد. شناسه فقط بعد از بررسی بالا می‌رود، پس یک درخواستِ ردشده یکی را مصرف نمی‌کند (`a_conflict_does_not_use_up_an_id`). `insert` حالا `Result<Show, ApiError>` برمی‌گرداند و `create_show` از `?` استفاده می‌کند، همان الگویِ `validate_new_show(&input)?` درست بالایش. پیمایشِ همه‌یِ مقدارها O(n) است، هزینه‌یِ صادقانه‌یِ «عنوانِ یکتا» در یک `HashMap` با کلیدِ شناسه؛ یک دیتابیس از یک ایندکسِ یکتا استفاده می‌کرد (ماژولِ Postgres، ۳.۵.۳ و بعد).

## چالش: `problem_response`

```rust
pub fn problem_response(&self, instance: &str) -> Response {
    let status = self.status();
    let mut body = serde_json::json!({
        "type": format!("https://api.example.com/problems/{}", self.code()),
        "title": status.canonical_reason().unwrap_or("Error"),
        "status": status.as_u16(), "detail": self.message(), "instance": instance,
    });
    if let ApiError::Validation(fields) = self { body["errors"] = serde_json::json!(fields); }
    (status, [(header::CONTENT_TYPE, "application/problem+json")], body.to_string()).into_response()
}
```

`status()`، `code()` و `message()` را بدونِ تغییر دوباره به کار می‌برد. نکته‌یِ درس همین است: چون این سه پرسش در یک جا جواب داده می‌شوند، یک قالبِ دومِ رویِ سیم یک تابع است، نه تغییر در هر هندلر. `Json(...)` `application/json` را تحمیل می‌کرد، پس بدنه یک رشته است با هدرِ صریح. `title` عبارتِ دلیلِ کدِ وضعیت است، چون RFC می‌خواهد از رخداد به رخداد پایدار بماند (متنِ خاص در `detail` می‌رود). یک `500` هنوز فقط `message()`ِ ثابت را دارد، پس problem هم چیزی فاش نمی‌کند (`an_internal_problem_leaks_nothing`).

## تست‌هایِ کلِ پشته چه چیزی را می‌گیرند که تست‌هایِ واحد نمی‌گیرند

`tests/error_test.rs` ثابت می‌کند هر `ApiError` پاکتِ درست می‌سازد. نمی‌تواند بگوید یک درخواست واقعاً به `ApiError` *می‌رسد*: یک `{id}`ِ بد اول از اکسترکتورِ `Path`ِ `axum` می‌گذرد، یک مسیرِ ناشناخته اول از روتر. `every_error_has_exactly_one_top_level_key_and_a_code_and_message` در `tests/api_test.rs` تستی است که ادعایِ درس را واقعاً ثابت می‌کند، چون شش نوعِ شکستِ مختلف، از جمله آن‌هایی که خودِ `axum` می‌سازد، می‌فرستد و برایِ همه یک شکل را می‌سنجد. خودِ قاعده‌یِ همیشگی (invariant) را تست کن، نه فقط موردها را.

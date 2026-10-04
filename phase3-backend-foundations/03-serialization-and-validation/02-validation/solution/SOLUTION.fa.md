# راه‌حل — ۳.۳.۲ اعتبارسنجی

کدِ کامل `solution/src/lib.rs` است؛ همه‌ی تست‌هایِ `solution/tests/` را می‌گذراند، از جمله `build_test.rs` برایِ پله‌ی «بساز».

## `validate_handle`

```rust
pub fn validate_handle(handle: &str) -> Result<(), ValidationError> {
    if handle.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        Ok(())
    } else {
        Err(ValidationError::new("handle_chars")
            .with_message("handle may only contain letters, digits and underscores".into()))
    }
}
```

`.all` برایِ رشته‌ی خالی `true` است، و مشخصات همین را می‌خواهد: خالی‌بودن کارِ `length` است و هر قاعده یک چیز می‌گوید. استفاده از `is_ascii_alphanumeric` (نه `is_alphanumeric`) دلیلِ ردشدنِ یک واژه‌ی فارسی است: مشخصات حروفِ *ASCII* می‌گوید، و تست یکی را امتحان می‌کند. خطا هم `code` لازم دارد (چیزی که برنامه می‌تواند رویش match کند) و هم `message` (چیزی که آدم می‌خواند).

## `flatten_errors`

```rust
fn walk(errors: &ValidationErrors, prefix: &str, out: &mut BTreeMap<String, Vec<String>>) {
    for (field, kind) in errors.errors() {
        let path = format!("{prefix}{field}");
        match kind {
            ValidationErrorsKind::Field(list) => {
                let mut msgs: Vec<String> = list.iter().map(|e| e.message.as_ref().unwrap_or(&e.code).to_string()).collect();
                msgs.sort();
                out.insert(path, msgs);
            }
            ValidationErrorsKind::Struct(inner) => walk(inner, &format!("{path}."), out),
            ValidationErrorsKind::List(items) => {
                for (i, inner) in items { walk(inner, &format!("{path}[{i}]."), out); }
            }
        }
    }
}
```

`flatten_errors` یک `BTreeMap`ِ خالی می‌سازد، `walk` را با پیشوندِ خالی صدا می‌زند و نقشه را برمی‌گرداند. بازگشت نکته‌ی اصلی است: گره‌ی `Struct` یا `List` یک `ValidationErrors`ِ دیگر در خود دارد، پس تابع خودش را با پیشوندی بلندتر صدا می‌زند (`"reviewer."`، `"notes[1]."`). فقط گره‌ی `Field` برگ است و فقط آنجا چیزی درج می‌شود. `errors()` (نه `field_errors()`) هر سه نوعِ گره را نشان می‌دهد. `e.message.as_ref().unwrap_or(&e.code)` یعنی «پیام اگر بود، وگرنه کد» بدونِ کلون‌کردنِ پیش‌ازموقع. `BTreeMap` کلیدها را رایگان مرتب نگه می‌دارد و `msgs.sort()` پیام‌هایِ درونِ هر کلید را مرتب می‌کند، پس خروجی در هر اجرا یکی است با اینکه `HashMap`ِ زیرینش به ترتیبِ تصادفی پیمایش می‌شود. درختِ خالی صفر بار حلقه می‌زند و نقشه‌ی خالی می‌دهد.

## `ApiError`، هندلرها و `app`

```rust
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::Validation(errors) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({ "errors": flatten_errors(&errors) })),
            ).into_response(),
        }
    }
}
```

توپلِ `(StatusCode, Json<_>)` از قبل `IntoResponse` دارد و `Json` هدرِ `Content-Type: application/json` را می‌گذارد، همان ترفندِ `AnimeError`ِ ۳.۲.۳. `BTreeMap<String, Vec<String>>` صفتِ `Serialize` دارد، پس `json!` آن را همان‌طور که هست می‌گیرد.

```rust
pub async fn create_review(
    State(store): State<Arc<ReviewStore>>,
    Json(input): Json<NewReview>,
) -> Result<(StatusCode, Json<StoredReview>), ApiError> {
    input.validate()?;
    Ok((StatusCode::CREATED, Json(store.add(input))))
}
```

`input.validate()?` یک `ValidationErrors` را از راهِ `From` به `ApiError` تبدیل می‌کند و زود برمی‌گردد. اعتبارسنجی پیش از `store.add` است، پس review ردشده هرگز ذخیره نمی‌شود و شناسه‌ای مصرف نمی‌کند (`a_rejected_review_is_not_stored_and_uses_up_no_id`). `list_reviews` همان `Json(store.list())` است، و `app` یک `.route("/reviews", get(list_reviews).post(create_review))` است و بعدش `.with_state(store)`.

## بساز: `ValidatedJson<T>`

```rust
async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
    let Json(value) = Json::<T>::from_request(req, state)
        .await
        .map_err(IntoResponse::into_response)?;
    value.validate().map_err(|e| ApiError::from(e).into_response())?;
    Ok(ValidatedJson(value))
}
```

اکسترکتور خودِ `from_request`ِ `Json<T>` را صدا می‌زند، پس ردهایِ `Json` دست‌نخورده بیرون می‌آیند: همین است که `400` و `415` همان می‌مانند. هر دو مسیرِ شکست به یک `Response` تبدیل می‌شوند، برای همین `type Rejection = Response`. `T: DeserializeOwned + Validate` کلِ کران است: هر نوعی که از JSON خوانده و اعتبارسنجی شود کار می‌کند، که `build_test.rs` با نوعِ `Ping` نشان می‌دهد و ربطی به review ندارد. برایِ استفاده در روتر، `create_review` می‌شود `ValidatedJson(input): ValidatedJson<NewReview>` و خطِ `validate()?` حذف می‌شود.

## طرحِ چالش

قاعده‌ی سطحِ ساختار `#[validate(schema(function = "extreme_needs_reason"))]` رویِ `NewReview` است، با `fn extreme_needs_reason(r: &NewReview) -> Result<(), ValidationError>`. در `validator` ۰٫۱۸٫۱ خطایش زیرِ کلیدِ `__all__` ثبت می‌شود، که `flatten_errors` مثلِ یک کلیدِ عادی ردش می‌کند. تغییرِ نامش به چیزی مثلِ `"review"` یک خط در شاخه‌ی `Field` است.

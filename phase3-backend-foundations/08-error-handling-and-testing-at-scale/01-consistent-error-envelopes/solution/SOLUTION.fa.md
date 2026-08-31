# راه‌حل

```rust
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            ApiError::NotFound(message) => (StatusCode::NOT_FOUND, "not_found", message),
            ApiError::Validation(message) => {
                (StatusCode::BAD_REQUEST, "validation_failed", message)
            }
            ApiError::Internal(message) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", message)
            }
        };

        (
            status,
            Json(ErrorBody {
                error: ErrorDetail {
                    code: code.to_string(),
                    message,
                },
            }),
        )
            .into_response()
    }
}
```

```rust
impl From<validator::ValidationErrors> for ApiError {
    fn from(errors: validator::ValidationErrors) -> Self {
        let mut messages: Vec<String> = errors
            .field_errors()
            .iter()
            .flat_map(|(field, field_errors)| {
                field_errors.iter().map(move |error| {
                    let message = error
                        .message
                        .clone()
                        .unwrap_or_else(|| error.code.clone())
                        .to_string();
                    format!("{field}: {message}")
                })
            })
            .collect();
        messages.sort();
        ApiError::validation(messages.join("; "))
    }
}
```

`into_response(self)` مالک error است؛ `match self` می‌تواند `String` درون variant را مستقیم move کند و نیاز به `.clone()` ندارد. در conversionِ validation، ترتیب iteration در `HashMap` تضمین نشده؛ `.sort()` جلوی تستِ گاه‌به‌گاه و اعصاب‌خردکنِ message با ترتیب متفاوت را می‌گیرد.

`WidgetStore::create` و `get` الگوی «قفل کن، کمترین کار را در lock انجام بده، clone برگردان» دارند. همهٔ پاسخ‌های خطا از یک enum رد می‌شوند؛ بنابراین اضافه‌شدن `Conflict` یعنی variant، arm در `IntoResponse`، `code` و منطق store را اضافه می‌کنی، نه اینکه JSON را در همهٔ handlerها دوباره بسازی. تستِ envelope دقیقاً تضمین می‌کند نه‌فقط مثال‌ها، بلکه invariantِ شکل پاسخ باقی مانده است.

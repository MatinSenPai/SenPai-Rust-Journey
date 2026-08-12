# پاسخ تشریحی

```rust
pub fn validation_summary(errors: &ValidationErrors) -> Vec<String> {
    let mut messages: Vec<String> = errors
        .field_errors()
        .iter()
        .flat_map(|(field, errs)| {
            errs.iter().map(move |e| {
                let message = e.message.clone().unwrap_or_else(|| e.code.clone()).to_string();
                format!("{field}: {message}")
            })
        })
        .collect();
    messages.sort();
    messages
}
```

یک field می‌تواند چند rule شکسته داشته باشد، پس `flat_map` روی fieldها و `map` روی errorهای هر field داریم. `message` سفارشی در `Option<Cow<'static, str>>` است؛ اگر نبود، `code` مانند `length` یا `range` fallback می‌شود. sort به‌خاطر بی‌قراردادبودن ترتیب `HashMap` لازم است؛ بدون آن test گاهی rating و گاهی title را اول می‌بیند.

```rust
pub fn parse_review(json: &str) -> Result<ReviewSubmission, ReviewError> {
    let submission: ReviewSubmission =
        serde_json::from_str(json).map_err(|e| ReviewError::InvalidJson(e.to_string()))?;
    submission.validate()
        .map_err(|errors| ReviewError::Invalid(validation_summary(&errors)))?;
    Ok(submission)
}
```

دو `?`، دو error type را در محل وقوع به `ReviewError` تبدیل می‌کنند. title گمشده در `from_str` رد می‌شود، زیرا field غیرdefault باید در JSON باشد. `11` یک `u8` معتبر است و فقط `.validate()` rule بازه را رد می‌کند. macro validator روی `Option<T>` generated code را با `if let Some` guard می‌کند. مزیت دو مرحله جداسازی مسئولیت و unit test ساده است؛ عیبش این است که JSON بد اصلاً به validate نمی‌رسد و یک dict مشترک از همه‌ی خطاهای shape و rule نمی‌دهد. برای cross-field rule از `#[validate(schema(function = "check_title_and_comment_differ"))]` و تابعی که کل `ReviewSubmission` را می‌گیرد استفاده کن.

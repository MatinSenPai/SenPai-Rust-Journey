# پاسخ تشریحی

```rust
impl TryFrom<u8> for Percentage {
    type Error = ValidationError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value <= 100 {
            Ok(Self(value))
        } else {
            Err(ValidationError::InvalidPercentage(value))
        }
    }
}
```

field خصوصی تضمین می‌کند هر `Percentage` قابل مشاهده از API در بازه است. blanket implementation کتابخانه‌ی استاندارد از `TryFrom<T> for U`، مسیر `TryInto<U> for T` را فراهم می‌کند؛ لازم نیست هر دو را بنویسی.

برای `EmailAddress`، ورودی `String` move می‌شود. در موفقیت همان allocation داخل newtype می‌رود و clone لازم نیست؛ در شکست error مالک رشته را پس می‌دهد تا caller آن را log، اصلاح یا در فرم نمایش دهد. هزینه فقط حمل همان allocation در error است، نه کپی اجباری.

`as` برای narrowing صریحاً bitهای بالایی را دور می‌ریزد؛ ۳۰۰ modulo ۲۵۶ برابر ۴۴ است. `TryFrom` از silent corruption جلوگیری می‌کند. truncation فقط وقتی مناسب است که semantics modulo یا bit-level واقعاً هدف باشد، نه تبدیل عادی داده‌ی کسب‌وکار.

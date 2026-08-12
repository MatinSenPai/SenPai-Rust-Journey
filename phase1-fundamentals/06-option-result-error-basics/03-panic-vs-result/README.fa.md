# ۰۶.۳ — `panic!` در برابر `Result`

این دو ابزار معنای متفاوت دارند. panic شکست بازیابی‌ناپذیر thread فعلی است؛ `Result` تصمیم را به فراخوان می‌دهد.

```rust
fn get_config_value(map: &std::collections::HashMap<String, String>, key: &str) -> String {
    map.get(key)
        .unwrap_or_else(|| panic!("missing required config key: {key}"))
        .clone()
}
```

در panic با strategy پیش‌فرض unwind، destructorهای محدوده اجرا می‌شوند؛ اما project می‌تواند `panic = "abort"` داشته باشد و آن‌وقت unwind انجام نمی‌شود. در سرور نیز نباید فرض کنی هر panic فقط همان درخواست را بی‌اثر می‌کند؛ runtime، task supervision و panic policy تعیین‌کننده‌اند.

## چه زمانی panic؟

- ناوردایی داخلی به‌دلیل bug شکسته است.
- آزمون، prototype یا script کوتاه است.
- ادامه‌ی فرایند بی‌معناست، مثلاً پیکربندی ضروری startup وجود ندارد.

## چه زمانی `Result`؟

- ورودی کاربر، پایان مهلت، نبود فایل و خطاهای عادی جهان خارج.
- فراخوان می‌تواند retry، fallback، log یا پاسخ مناسب انجام دهد.

برای فرض «نباید شکست بخورد»، `.expect("reason")` از `.unwrap()` بهتر است چون assumption شکسته‌شده را ثبت می‌کند. ورودی نامعتبر مشتری bug سرور نیست؛ یک Tuesday عادی در بک‌اند است و باید `Result` شود.

## مرز تصمیم

نبود پیکربندی startup گاهی بهتر است به خطا سطح `main` و exit کنترل‌شده تبدیل شود تا logging/telemetry کامل بماند. panic قانون ثابت برای پیکربندی نیست؛ قرارداد برنامه تعیین می‌کند.

```senpai-visual
{"kind":"result","labels":["bug invariant","panic","external error → Result"]}
```

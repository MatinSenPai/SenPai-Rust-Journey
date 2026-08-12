# ۰۶.۲ — `Result` و operator `?`

`Option` نبودن را نشان می‌دهد؛ `Result` شکست را همراه دلیل مدل می‌کند:

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

```rust
fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("cannot divide by zero".to_string())
    } else {
        Ok(a / b)
    }
}
```

فراخوان می‌تواند با match، `unwrap_or` یا propagation تصمیم بگیرد. `unwrap()` روی `Err` panic می‌کند.

```rust
fn compute(a: f64, b: f64, c: f64) -> Result<f64, String> {
    let step1 = divide(a, b)?;
    let step2 = divide(step1, c)?;
    Ok(step2)
}
```

`expr?` در حالت `Ok(value)` مقدار را می‌دهد و در حالت `Err(e)` زودهنگام خطا را از تابع فعلی return می‌کند. خطا نوع باید با خروجی تابع سازگار یا از راه `From` قابل تبدیل باشد.

در بک‌اند تقریباً هر فراخوانی پایگاه داده یا عملیات شبکه‌ای ممکن است شکست بخورد. `?` خطا را پنهان نمی‌کند؛ فقط کد تکراریِ برگرداندن خطا را کوتاه می‌کند. در مرز HTTP هنوز باید خطای دامنه را به کد وضعیت و پاسخی امن تبدیل کنی.

```senpai-visual
{"kind":"result","labels":["operation","Ok ادامه","Err بازگشت"]}
```

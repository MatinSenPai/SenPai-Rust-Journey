# ایست بازرسی

1. چرا `now: Instant` پارامتر است و `Instant::now()` درون `try_acquire` صدا زده نمی‌شود؟ testها چه چیزی را از دست می‌دهند؟
2. چرا `tokens` از نوع `f64` است، نه `u32`؟ در refillهای کوچک چه اتفاقی برای integer می‌افتد؟
3. چرا `last_refill` حتی وقتی token کامل اضافه نشده به `now` تغییر می‌کند؟
4. تفاوت بنیادی timestamp در token bucket و TTL cache چیست؟ هرکدام از چه خطری محافظت می‌کنند؟
5. `bool` در `try_acquire` کدام strategy backpressure است؟ `async fn acquire` که منتظر token می‌ماند به چه سازوکارهایی نیاز دارد؟

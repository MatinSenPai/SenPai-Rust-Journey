# پاسخ تشریحی

```rust
macro_rules! timed {
    ($work:expr) => {{
        let start = std::time::Instant::now();
        let result = $work;
        let elapsed = start.elapsed();
        (result, elapsed)
    }};
}
```

binding میانی تضمین می‌کند `$work` فقط یک بار expand و evaluate شود. اگر transcriber برای نتیجه و اندازه‌گیری دوباره `$work` را بنویسد، side effect دوبار رخ می‌دهد. hygiene نیز `start` داخلی را از binding هم‌نام caller جدا نگه می‌دارد.

در `max_of!` arm تک‌عبارتی recursion را متوقف می‌کند. arm چندتایی نخستین مقدار را با expansion باقی ورودی مقایسه می‌کند. `string_map!` نیز token `=>` را می‌پذیرد و به فراخوانی‌های `.insert` تبدیل می‌کند؛ function نمی‌تواند grammar زبان در call site را تغییر دهد.

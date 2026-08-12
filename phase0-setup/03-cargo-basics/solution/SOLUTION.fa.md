# راه‌حل — مبانی Cargo

```rust
pub fn format_greeting(name: &str, times: u32) -> String {
    (0..times)
        .map(|_| format!("Hello, {name}!"))
        .collect::<Vec<_>>()
        .join("\n")
}
```

`0..times` یک `Range` قابل iteration است. چون index لازم نداریم، closure از `_` استفاده می‌کند. `format!` همان نقش کلی f-string پایتون را دارد. سپس greetingها در `Vec` جمع و با newline متصل می‌شوند.

نسخه‌ی صریح‌تر نیز درست است:

```rust
pub fn format_greeting(name: &str, times: u32) -> String {
    let mut result = String::new();
    for i in 0..times {
        if i > 0 {
            result.push('\n');
        }
        result.push_str(&format!("Hello, {name}!"));
    }
    result
}
```

نسخه‌ی iterator اصطلاحی‌تر و نسخه‌ی loop برای شروع آشکارتر است؛ هیچ‌کدام به‌خودی‌خود غلط نیستند. در فاز دو، زنجیره‌ی `map → collect → join` را عمیق‌تر می‌بینی.

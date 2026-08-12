# راه‌حل — سلام Rust

```rust
pub fn describe_journey(language: &str, day: u32) -> String {
    format!("Day {day} of learning {language}: still compiling.")
}
```

`format!` مانند `println!` placeholderها را در کامپایل بررسی می‌کند، اما به‌جای چاپ یک `String` برمی‌گرداند. syntax جدید متغیر را با `{day}` مستقیم capture می‌کند؛ شکل `format!("Day {}", day)` نیز معتبر و اصطلاحی است.

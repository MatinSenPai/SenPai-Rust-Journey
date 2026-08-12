# راه‌حل

```rust
pub fn chained_division(a: f64, b: f64, c: f64) -> Result<f64, String> {
    let step1 = safe_divide(a, b)?;
    let step2 = safe_divide(step1, c)?;
    Ok(step2)
}
```

شکل تقریبی نخستین `?` چنین است:

```rust
let step1 = match safe_divide(a, b) {
    Ok(v) => v,
    Err(e) => return Err(e),
};
```

```rust
pub fn parse_positive(s: &str) -> Result<u32, String> {
    s.parse::<u32>().map_err(|e| e.to_string())
}
```

`parse` یک `Result<u32, ParseIntError>` می‌دهد، ولی امضای تابع خطای `String` می‌خواهد. `map_err` خطا نوع را صریح تبدیل می‌کند. در برنامه‌ی بزرگ‌تر معمولاً enum خطای typed از `String` بهتر است، چون کد منبع و category را حفظ می‌کند.

# راه‌حل

```rust
pub fn increment_n_times(times: u32) -> u32 {
    let mut count = 0;
    for _ in 0..times {
        count += 1;
    }
    count
}

pub fn parse_and_double(input: &str) -> i32 {
    let input: i32 = input.parse().expect("input should be a valid integer");
    let input = input * 2;
    input
}
```

`count` همان مقدار منطقی است که بار‌ها تغییر می‌کند. `input` یک pipeline از `&str` به `i32` و سپس مقدار دوبرابر است. `parse()` یک `Result` می‌دهد؛ `expect` برای تمرین ساده قابل قبول است، اما ورودی نامطمئن بک‌اند باید با `Result` مدیریت شود.

clippy ممکن است اتصال نهایی را زائد بداند؛ اینجا فقط برای تمرین shadowing حفظ شده و هر `#[allow]` باید دلیل محلی داشته باشد.

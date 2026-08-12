# راه‌حل

```rust
pub fn required_config(vars: &HashMap<String, String>, key: &str) -> String {
    vars.get(key)
        .cloned()
        .unwrap_or_else(|| panic!("missing required config key: {key}"))
}
```

closure فقط هنگام نبود key اجرا و نام دقیق آن را گزارش می‌کند.

```rust
pub fn average_of_nonempty(nums: &[f64]) -> f64 {
    assert!(
        !nums.is_empty(),
        "average_of_nonempty called with an empty slice — caller bug"
    );
    let sum: f64 = nums.iter().sum();
    sum / nums.len() as f64
}
```

`assert!` ناوردایی تابع را بیان می‌کند. `required_config` درباره‌ی environment و `average_of_nonempty` درباره‌ی قرارداد داخلی است؛ هر دو در طراحی این تمرین «ادامه ناممکن» هستند. در مقابل، `parse_user_age` ورودی عادی و نامطمئن بیرونی است و باید `Result` بدهد.

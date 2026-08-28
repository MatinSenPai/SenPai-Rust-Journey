# راه‌حل — ۱.۶.۳ `Result` و علامتِ سؤال

```rust
pub fn safe_divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("cannot divide by zero".to_string())
    } else {
        Ok(a / b)
    }
}

pub fn parse_quantity(s: &str) -> Result<u32, String> {
    s.parse::<u32>().map_err(|e| e.to_string())
}

pub fn chained_division(a: f64, b: f64, c: f64) -> Result<f64, String> {
    let step1 = safe_divide(a, b)?;
    let step2 = safe_divide(step1, c)?;
    Ok(step2)
}

pub fn half_or_zero(input: &str) -> f64 {
    input.parse::<f64>().map(|value| value / 2.0).unwrap_or(0.0)
}

pub fn parsed_and_divided(input: &str, divisor: f64) -> Option<f64> {
    input
        .parse::<f64>()
        .ok()
        .and_then(|value| safe_divide(value, divisor).ok())
}
```

پنج تابع، پنج جنبه‌ی همین درس: ساختنِ دستی، `.parse()`، `?`، و دو ترکیب‌گر که یک `Result` را عمداً به چیزِ ساده‌تری تبدیل می‌کنند.

## `safe_divide` — همان چیزی که در «مفهوم» دیدی

```rust
if b == 0.0 {
    Err("cannot divide by zero".to_string())
} else {
    Ok(a / b)
}
```

دو راهِ خروج، هر دو صریح. هیچ `panic!`ی نیست، هیچ `None`ی نیست که دلیل را دور بریزد — فقط یک `if` معمولی که یکی از دو بازوی enum را می‌سازد. این تابع پایه‌ی بقیه‌ی درس است؛ سه تابعِ دیگر صدایش می‌زنند.

## `parse_quantity` — `.parse()` به‌علاوه‌ی `.map_err()`

```rust
s.parse::<u32>().map_err(|e| e.to_string())
```

`.parse::<u32>()` خودش یک `Result<u32, ParseIntError>` می‌دهد — نصفِ کار از قبل تمام است. مشکل این است که امضای تابع `Result<u32, String>` قول داده، نه `Result<u32, ParseIntError>`. `.map_err()` دقیقاً همین شکاف را پر می‌کند: رویِ `Ok` دست نمی‌زند، و رویِ `Err` تابعی که بهش دادی را صدا می‌زند — اینجا `.to_string()`، که `Display` را روی `ParseIntError` صدا می‌زند و همان جمله‌ی آشنا را برمی‌گرداند.

نوشتنِ یک `match` هم درست بود:

```rust
match s.parse::<u32>() {
    Ok(n) => Ok(n),
    Err(e) => Err(e.to_string()),
}
```

ولی وقتی کاری که با `Ok` باید کرد «هیچ‌کاری» است و کاری که با `Err` باید کرد «یک تابع رویش صدا بزن» است، `.map_err()` همان جمله را کوتاه‌تر می‌گوید.

## `chained_division` — `?` دوبار

```rust
let step1 = safe_divide(a, b)?;
let step2 = safe_divide(step1, c)?;
Ok(step2)
```

هر `?` یک `match` کاملِ چهارخطی را جایگزین کرده. اگر `safe_divide(a, b)` شکست بخورد، خطِ دومِ تابع هرگز اجرا نمی‌شود — همان `Err` بلافاصله برمی‌گردد. تست‌ها هر دو راهِ شکست را جدا بررسی می‌کنند تا مطمئن شوند این عبورِ زودهنگام واقعاً کار می‌کند، نه فقط اینکه تابع در حالتِ موفق درست است.

## `half_or_zero` — `.map()` و `.unwrap_or()` پشتِ سرِ هم

```rust
input.parse::<f64>().map(|value| value / 2.0).unwrap_or(0.0)
```

سه قدم، یک خط: پارس کن، اگر موفق شد نصفش کن، اگر هر جایش شکست خورد `0.0` بده. نوشتنِ این با `match` هم می‌شد، ولی وقتی زنجیره‌ی تبدیل‌ها این‌قدر کوتاه است، ترکیب‌گرها دقیقاً همان چیزی را می‌گویند که تابع انجام می‌دهد — نه بیشتر.

نکته‌ی ظریف: `.map()` قبل از `.unwrap_or()` می‌آید، نه بعدش. اگر جایشان عوض شود (`.unwrap_or(0.0).map(...)`)، اصلاً کامپایل نمی‌شود — `.unwrap_or()` رویِ یک `Result<f64, _>` یک `f64` ساده تحویل می‌دهد، و `f64` متدی به اسمِ `.map()` ندارد.

## `parsed_and_divided` — `.ok()` دوبار، و `.and_then()` بینشان

```rust
input
    .parse::<f64>()
    .ok()
    .and_then(|value| safe_divide(value, divisor).ok())
```

اینجا دو مرحله‌ی شکست‌پذیر پشتِ سرِ همند: پارس کردن، بعد تقسیم. هر دو ممکن است شکست بخورند، و آخرِ کار فقط `Some`/`None` می‌خواهیم — دلیلِ دقیقِ شکست دیگر مهم نیست.

`.ok()` اولی، `Result<f64, ParseIntError>` را به `Option<f64>` تبدیل می‌کند. `.and_then()` فقط وقتی آن `Some` باشد بستارش را صدا می‌زند؛ بستار خودش `safe_divide` را صدا می‌زند و با `.ok()`ِ دومی، `Result<f64, String>`اش را هم به `Option<f64>` تبدیل می‌کند. نتیجه دقیقاً همان نوعی است که `.and_then()` انتظار دارد بستار برگرداند، پس دو تا `Option` تودرتو نمی‌شوند.

## این درس واقعاً درباره‌ی چه بود

- **`Result<T, E>` یک enumِ معمولی است** با دو بازو، یکی‌شان یک دلیل حمل می‌کند — همان چیزی که `Option` نداشت.
- **`?` یک `match` را فشرده می‌کند**، نه چیزِ جدیدی. `chained_division` همان کاری را می‌کند که با دست هم می‌شد نوشت، فقط کوتاه‌تر.
- **ترکیب‌گرها برای وقتی‌اند که رفتارت رویِ `Ok`/`Err` را می‌شود در یک عبارت گفت** — `.map`، `.map_err`، `.and_then`، `.unwrap_or`، `.ok()`. وقتی رفتار پیچیده‌تر می‌شود (شرط‌های تودرتو، چند خط منطق)، `match` هنوز خواناتر است.
- **`.ok()` یک درِ خروجِ عمدی است**: وقتی داری از دنیای «چرا شکست خورد» به دنیای «آیا اصلاً چیزی هست» می‌روی، دقیقاً همین متد است که آن مرز را می‌کشد.

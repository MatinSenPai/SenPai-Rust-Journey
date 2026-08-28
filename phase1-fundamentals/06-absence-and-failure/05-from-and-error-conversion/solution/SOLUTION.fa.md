# راه‌حل — ۱.۶.۵ `From` و تبدیلِ خطا

```rust
use std::num::ParseIntError;

#[derive(Debug, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, PartialEq)]
pub enum ColorError {
    BadChannel(ParseIntError),
    TooDark,
}

impl From<ParseIntError> for ColorError {
    fn from(err: ParseIntError) -> Self {
        ColorError::BadChannel(err)
    }
}

pub fn parse_color(s: &str) -> Result<Color, ColorError> {
    let mut parts = s.split(',');
    let r_str = parts.next().unwrap_or("");
    let g_str = parts.next().unwrap_or("");
    let b_str = parts.next().unwrap_or("");

    let r = r_str.parse::<u8>()?;
    let g = g_str.parse::<u8>()?;
    let b = b_str.parse::<u8>()?;

    if (r as u32) + (g as u32) + (b as u32) < 30 {
        return Err(ColorError::TooDark);
    }

    Ok(Color { r, g, b })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Celsius(pub f64);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fahrenheit(pub f64);

impl From<Celsius> for Fahrenheit {
    fn from(value: Celsius) -> Self {
        Fahrenheit(value.0 * 9.0 / 5.0 + 32.0)
    }
}

pub fn all_fahrenheit(values: Vec<Celsius>) -> Vec<Fahrenheit> {
    let mut out = Vec::with_capacity(values.len());
    for value in values {
        out.push(value.into());
    }
    out
}
```

دو تای این‌ها یک `impl From` نوشتند. دو تای دیگر از همان `impl` استفاده کردند — یکی با `?` روی یک خطا، یکی با `.into()` روی یک تبدیلِ بدونِ خطا.

## `impl From<ParseIntError> for ColorError` — خودِ راز

```rust
impl From<ParseIntError> for ColorError {
    fn from(err: ParseIntError) -> Self {
        ColorError::BadChannel(err)
    }
}
```

سه خط. همه‌ی کاری که می‌کند این است که بگوید «اگر یک `ParseIntError` دیدی، آن را داخلِ `BadChannel` بگذار.» خودِ این تابع هیچ‌جا صدا زده نمی‌شود — `?` است که وقتِ لازم پیدایش می‌کند و صدایش می‌زند، دقیقاً همان چیزی که در «مفهوم» دیدی.

## `parse_color` — سه `?` خالی، به‌لطفِ همان `impl`

```rust
let r = r_str.parse::<u8>()?;
let g = g_str.parse::<u8>()?;
let b = b_str.parse::<u8>()?;
```

سه فراخوانیِ `.parse::<u8>()`، هر سه `ParseIntError` می‌دهند، هر سه با همان یک `impl From` بالا به `ColorError` تبدیل می‌شوند. اگر این `impl` نبود، همین سه خط به یک `match` یا یک `.map_err(ColorError::BadChannel)` روی هرکدام نیاز داشت — دقیقاً همان تفاوتی که در «راهِ دستی» دیدی.

فیلدهای گم‌شده هم رایگان جواب می‌دهند: `s.split(',')` روی `"1,2"` فقط دو تکه می‌دهد؛ `parts.next().unwrap_or("")` برای سومی یک رشته‌ی خالی می‌گذارد، و `"".parse::<u8>()` هم خودش با `IntErrorKind::Empty` شکست می‌خورد. یک راهِ جدا برای «فیلد کم است» لازم نبود — همان راهِ «عدد نامعتبر است» کافی بود.

قاعده‌ی تاریکی بعد از هر سه `?` می‌آید، نه قبلش:

```rust
if (r as u32) + (g as u32) + (b as u32) < 30 {
    return Err(ColorError::TooDark);
}
```

باید مطمئن باشی هر سه عدد معتبرند قبل از اینکه جمع‌شان معنی داشته باشد. جمعِ سه `u8` می‌تواند تا ۷۶۵ برسد که در خودِ `u8` جا نمی‌شود، برای همین هرکدام قبل از جمع به `u32` تبدیل می‌شود — [۱.۱.۲](../../../01-foundations/02-scalar-types-and-overflow/README.fa.md) — و `TooDark` مستقیم ساخته می‌شود، بدونِ هیچ `From`ی، چون این خطا از تبدیلِ چیزی نمی‌آید؛ خودِ این تابع تشخیصش می‌دهد.

## `impl From<Celsius> for Fahrenheit` — همان صفت، بدونِ خطا

```rust
impl From<Celsius> for Fahrenheit {
    fn from(value: Celsius) -> Self {
        Fahrenheit(value.0 * 9.0 / 5.0 + 32.0)
    }
}
```

`From` قولِ «همیشه موفق» می‌دهد؛ این تبدیل هم همیشه موفق است — هیچ عددِ اعشاری‌ای وجود ندارد که این فرمول رویش شکست بخورد. هیچ `Result`ی هم در امضا نیست، چون هیچ چیزی برای گزارش‌دادن نیست.

## `all_fahrenheit` — `.into()` به‌جایِ دوباره‌نویسیِ فرمول

```rust
let mut out = Vec::with_capacity(values.len());
for value in values {
    out.push(value.into());
}
out
```

`value.into()` دقیقاً همان `Fahrenheit::from(value)` است، فقط از جهتِ مقصد خوانده‌شده. کامپایلر مقصد را از امضایِ خودِ `all_fahrenheit` می‌داند: نوعِ بازگشتی `Vec<Fahrenheit>` است، پس هر `push` باید یک `Fahrenheit` بدهد، پس `.into()` می‌داند کدام `From` را صدا بزند — همانی که چند خط بالاتر نوشتیم.

نوشتنِ `Fahrenheit(value.0 * 9.0 / 5.0 + 32.0)` دوباره، همین‌جا هم، تست‌ها را پاس می‌کرد — ولی آن‌وقت فرمول در دو جا زندگی می‌کرد. تستِ `all_fahrenheit_matches_the_same_conversion_used_alone` دقیقاً همین را می‌سنجد: نتیجه‌ی `all_fahrenheit` باید با صدازدنِ مستقیمِ همان `From` یکی باشد. `Vec::with_capacity(values.len())` هم یک یادآوری از [۱.۲.۳](../../../02-ownership-and-memory/03-clone-and-copy/README.fa.md) است: طول از قبل معلوم است، پس چرا وکتور را چندبار دوباره‌تخصیص بدهیم؟

## این درس واقعاً درباره‌ی چه بود

- **`?` رویِ خطا `From::from` صدا می‌زند، بعد `Err`ِ تبدیل‌شده را برمی‌گرداند.** همین یک جمله، کلِ رفتارِ `?` را کامل می‌کند.
- **یک `impl From` را یک‌بار بنویس، همه‌جا کار می‌کند** — چه پشتِ `?` روی یک خطا، چه پشتِ `.into()` روی یک تبدیلِ بدونِ خطا.
- **`Into` را کسی دستی نمی‌نویسد.** پیاده‌سازیِ فراگیرِ کتابخانه‌ی استاندارد از هر `From` یک `Into` می‌سازد.
- **نه هر enum‌سازی از `From` می‌آید.** `TooDark` را خودت مستقیم می‌سازی، چون خطایش از تبدیلِ چیزی نیست.
- **یک `.into()` باید مقصدش را از جایی بگیرد** — نوعِ بازگشتی، نوعِ پارامتر، یا نوعِ صریحِ یک `let`. اگر هیچ‌کدام نگویند، `E0282` می‌گیری.

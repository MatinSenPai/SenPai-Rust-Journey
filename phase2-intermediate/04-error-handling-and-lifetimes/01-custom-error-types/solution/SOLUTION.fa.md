# پاسخ تشریحی

```rust
impl From<std::num::ParseIntError> for ConfigError {
    fn from(source: std::num::ParseIntError) -> Self {
        ConfigError::InvalidNumber {
            field: "max_retries".to_string(),
            source,
        }
    }
}
```

```rust
let max_retries_raw = fields
    .get("max_retries")
    .ok_or_else(|| ConfigError::MissingField("max_retries".to_string()))?;
let max_retries: u32 = max_retries_raw.parse()?;
```

خط مهم `max_retries_raw.parse()?` است. `.parse::<u32>()` یک `Result<u32, ParseIntError>` می‌سازد، در حالی که خروجی اعلام‌شده‌ی `parse_config` برابر `Result<Config, ConfigError>` است. معمولاً `?` با این دو نوع خطای متفاوت compile نمی‌شد. اما چون `impl From<ParseIntError> for ConfigError` وجود دارد، کامپایلر تبدیل را خودکار وارد می‌کند. تقریباً می‌توانی `?` را این‌طور بخوانی: «اگر `Err(e)` بود، `return Err(ConfigError::from(e))`». تمام سازوکار lookup همین trait است.

حالا `timeout_secs` را ببین:

```rust
let timeout_secs: u32 = timeout_secs_raw
    .parse()
    .map_err(|source| ConfigError::InvalidNumber {
        field: "timeout_secs".to_string(),
        source,
    })?;
```

خطای زیربنایی و variant مقصد همان‌اند، اما این call site نمی‌تواند به `From` تکیه کند؛ چون `From` موجود نام `max_retries` را ثابت نوشته است. تابع `From::from` فقط خود `ParseIntError` را می‌گیرد و نمی‌داند کدام `.parse()` آن را ساخته. برای ثبت field درست باید context را همان call site بدهیم؛ پس `.map_err` صریح لازم است. این پاسخ پرسش دوم است.

اگر config بزرگ‌تری با تعداد زیادی field عددی داشتیم، ثابت‌کردن نام یک field داخل `From` دیگر تصمیم خوبی نبود. آن `From` را حذف می‌کردیم و برای سازگاری در همه‌ی call siteها `.map_err(...)` می‌نوشتیم. `From` وقتی ارزش دارد که تبدیل غالب و بدون ابهام باشد. به‌محض نیاز به context وابسته به محل، تبدیل صریح صادقانه‌تر است.

پاسخ پرسش سوم: `impl std::error::Error for ConfigError {}` فقط چون supertraitهای `Error`، یعنی `Debug + Display`، از قبل برقرارند با بدنه‌ی خالی compile می‌شود. `#[derive(Debug)]` اولی و `impl Display` دستی دومی را فراهم می‌کند. اگر `#[derive(Debug)]` را حذف کنی، خط `impl Error` با اشاره به bound گمشده‌ی `Debug` رد می‌شود.

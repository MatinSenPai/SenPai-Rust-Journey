# راه‌حل — ۲.۵.۲ زنجیره‌ی منشأ و `Box<dyn Error>`

```rust
impl From<std::io::Error> for ConfigError {
    fn from(source: std::io::Error) -> Self {
        ConfigError::Io(source)
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ConfigError::Io(e) => Some(e),
            ConfigError::MissingField(_) => None,
            ConfigError::InvalidNumber { source, .. } => Some(source),
        }
    }
}

pub fn load_config(path: &str) -> Result<Config, ConfigError> {
    let contents = std::fs::read_to_string(path)?;
    parse_config_str(&contents)
}

pub fn error_chain(err: &dyn Error) -> Vec<String> {
    let mut chain = vec![err.to_string()];
    let mut cause = err.source();
    while let Some(source) = cause {
        chain.push(source.to_string());
        cause = source.source();
    }
    chain
}

pub fn effective_max_retries(
    path: &str,
    override_max_retries: Option<&str>,
) -> Result<u32, Box<dyn Error>> {
    match override_max_retries {
        Some(text) => Ok(text.parse::<u32>()?),
        None => Ok(load_config(path)?.max_retries),
    }
}
```

## `From<std::io::Error>` — همان یک‌خطی که `?` را به کار می‌اندازد

```rust
fn from(source: std::io::Error) -> Self {
    ConfigError::Io(source)
}
```

هیچ‌چیزِ باهوشانه‌ای در کار نیست — فقط `source` را به همان گونه‌ای که برایش ساخته شده منتقل کن. دلیلِ وجودِ این `impl` خودِ `load_config` است: بدونِ آن، `std::fs::read_to_string(path)?` اصلاً کامپایل نمی‌شد، چون `?` هیچ راهی نداشت یک `io::Error` را خودش تنها به `ConfigError` تبدیل کند. این دقیقاً همان مکانیزمی‌ست که ۲.۵.۱ برایِ `ParseIntError` به‌کار برد، این‌بار برایِ همان شکستی که یک فایلِ واقعی وارد می‌کند.

## `source()` — یک خط به‌ازایِ هر گونه، مطابقِ چیزی که واقعاً در خودش پیچیده

```rust
fn source(&self) -> Option<&(dyn Error + 'static)> {
    match self {
        ConfigError::Io(e) => Some(e),
        ConfigError::MissingField(_) => None,
        ConfigError::InvalidNumber { source, .. } => Some(source),
    }
}
```

`Io` و `InvalidNumber` هرکدام یک خطایِ دیگر را به‌عنوانِ فیلد در خودشان دارند، پس بازوهایشان یک ارجاع به آن برمی‌گردانند: `Some(e)`، `Some(source)`. `MissingField` فقط یک `String` دارد — هیچ خطایی زیرش نیست که به‌اش اشاره کند — پس بازویش `None` است. اگر برایِ `MissingField` هم `Some(...)` می‌نوشتی، کامپایلر اصلاً قبولش نمی‌کرد: از یک فیلدِ ساده‌یِ `String` نمی‌شود یک `&(dyn Error + 'static)` قرض گرفت. اشتباهِ برعکس — نوشتنِ `None` برایِ `Io` — کاملاً کامپایل می‌شد و فقط بی‌سروصدا اطلاعاتی را دور می‌ریخت که فراخواننده می‌توانست ازش استفاده کند؛ هیچ‌چیز این اشتباه را برایت نمی‌گیرد، و دقیقاً به همین دلیل تطبیقِ هر گونه با شکلِ واقعی‌اش اهمیت دارد.

## `load_config` — دو تبدیل، هرکدام از پشتِ یک `?`

```rust
pub fn load_config(path: &str) -> Result<Config, ConfigError> {
    let contents = std::fs::read_to_string(path)?;
    parse_config_str(&contents)
}
```

`?` اول واقعاً کار می‌کند: یک `io::Error` را از راهِ `From`ِ بالا به `ConfigError` تبدیل می‌کند. خطِ دوم نه به `?` نیاز دارد نه به هیچ تبدیلی — `parse_config_str` از قبل `Result<Config, ConfigError>` برمی‌گرداند، دقیقاً همان نوعی که این تابع هم برمی‌گرداند، پس نتیجه‌اش همین‌طوری پس داده می‌شود.

## `error_chain` — همان حلقه‌ی «پیمایشِ زنجیره»، این‌بار برگردانده‌شده به‌جایِ چاپ‌شده

```rust
pub fn error_chain(err: &dyn Error) -> Vec<String> {
    let mut chain = vec![err.to_string()];
    let mut cause = err.source();
    while let Some(source) = cause {
        chain.push(source.to_string());
        cause = source.source();
    }
    chain
}
```

`chain` دقیقاً با یک عضو شروع می‌شود: پیامِ خودِ `err`. بعد حلقه همان شکلِ درس با `println!` است — `source()` را صدا بزن، و تا وقتی `Some` است، پیامِ آن لایه را اضافه کن و از خودش `source()`ِ بعدی را بخواه. خودِ شرطِ حلقه اثباتِ پایانش هم هست: به‌محضِ اینکه یک `source()` مقدارِ `None` بدهد متوقف می‌شود، و این همیشه اتفاق می‌افتد چون هیچ‌چیزی تویِ `ConfigError`ِ این درس نمی‌تواند به خودش اشاره کند.

## `effective_max_retries` — override قبل از اینکه `path` اصلاً لمس شود تصمیم را می‌گیرد

```rust
pub fn effective_max_retries(
    path: &str,
    override_max_retries: Option<&str>,
) -> Result<u32, Box<dyn Error>> {
    match override_max_retries {
        Some(text) => Ok(text.parse::<u32>()?),
        None => Ok(load_config(path)?.max_retries),
    }
}
```

خودِ `match` همه‌چیز را از همان اول تصمیم می‌گیرد. تویِ بازویِ `Some(text)`، `path` اصلاً هیچ‌جا نمی‌آید — `text.parse::<u32>()?` یا موفق می‌شود یا یک `ParseIntError` را پرتاب می‌کند، و کل داستان همین است. تویِ بازویِ `None`، `load_config(path)?` یا موفق می‌شود یا یک `ConfigError` را پرتاب می‌کند. از یک تابع، دو نوعِ خطایِ کاملاً متفاوت بیرون می‌آید، و هر دو از پشتِ `?`ِ خودشان به `Box<dyn Error>` تبدیل می‌شوند — تنها جایی که این تصمیم گرفته می‌شود نوعِ برگشتی‌ست؛ هیچ‌کدام از دو بازو حتی اسمِ `Box` را نمی‌آورد.

## این درس واقعاً درباره‌ی چه بود

- **امضایِ `source()` پیشنهاد نیست.** `Option<&(dyn Error + 'static)>` دقیقاً همان چیزی‌ست که trait اعلام کرده، و `impl`ی که طولِ‌عمر را جور دیگری حذف می‌کند — گره‌زدنش به `&self` به‌جایِ `'static` — امضایِ متفاوتی‌ست که کامپایلر رد می‌کند.
- **یک زنجیره فقط چند بار پی‌درپیِ `source()` صدا زدن است، نه بیشتر.** `error_chain` همان حلقه‌ی پنج‌خطیِ «مفهوم» است، این‌بار با یک تستی اثبات‌شده که واقعاً دو حلقه‌ی واقعی را پیمایش می‌کند.
- **`Box<dyn Error>` دقیقاً همان‌جایی حقش را ادا می‌کند که دو نوعِ خطایِ بی‌ربط تویِ یک تابع به هم می‌رسند.** `effective_max_retries` هیچ‌وقت مجبور نشد `ConfigError` و `ParseIntError` را تویِ یک enumِ مشترک یکی کند — نوعِ برگشتی خودش این کار را کرد.
- **راحتی و هزینه، دو رویِ یک سکه‌اند.** پاک‌شدنِ نوع همان چیزی‌ست که گذاشت دو خطایِ بی‌ربط یک نوعِ برگشتی را شریک شوند؛ دقیقاً همان هم دلیلِ این است که هیچ‌چیزِ پایین‌دستی، بدونِ اول `downcast_ref()` زدن، نمی‌تواند رویِ نتیجه `match` کند.

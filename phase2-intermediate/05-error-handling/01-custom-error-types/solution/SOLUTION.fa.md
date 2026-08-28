# راه‌حل — ۲.۵.۱ نوع‌های خطای سفارشی و `std::error::Error`

```rust
impl std::fmt::Display for EntryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntryError::BlankName => write!(f, "entry is missing a name"),
            EntryError::BadScore(source) => write!(f, "invalid score: {source}"),
            EntryError::ScoreTooHigh(score) => {
                write!(f, "score {score} is above the maximum of 9999")
            }
        }
    }
}

impl From<ParseIntError> for EntryError {
    fn from(source: ParseIntError) -> Self {
        EntryError::BadScore(source)
    }
}

pub fn parse_entry(line: &str) -> Result<LeaderboardEntry, EntryError> {
    let (name, score_str) = line.split_once(':').unwrap_or((line, ""));
    let name = name.trim();
    if name.is_empty() {
        return Err(EntryError::BlankName);
    }
    let score: u32 = score_str.trim().parse()?;
    if score > 9999 {
        return Err(EntryError::ScoreTooHigh(score));
    }
    Ok(LeaderboardEntry {
        name: name.to_string(),
        score,
    })
}
```

هیچ‌کدام از این‌ها چیزی فراتر از آنچه در «مفهوم» دیدی نمی‌خواست — همان سه تکه، این‌بار برایِ یک دامنه‌ی دیگر.

## `Display` — سه بازو، یکی به‌ازایِ هر گونه

```rust
match self {
    EntryError::BlankName => write!(f, "entry is missing a name"),
    EntryError::BadScore(source) => write!(f, "invalid score: {source}"),
    EntryError::ScoreTooHigh(score) => {
        write!(f, "score {score} is above the maximum of 9999")
    }
}
```

مشخصات هر سه رشته را دقیق گفته بود. بازویِ `BadScore` پیامِ `Display` خودِ `ParseIntError`ِ زیربنایی را با `{source}` مستقیم تویِ پیامِ خودش می‌چیند — همان ترفندی که `ReviewError` در بدنه‌ی درس استفاده کرد، پس هیچ اطلاعاتی از شکستِ زیربنایی گم نمی‌شود، حتی در سطحِ پیامی که یک آدم می‌خواند.

## `impl std::error::Error for EntryError {}` — از قبل داده شده، و چرا به هیچ‌چیز نیاز نداشت

این یکی از اول تویِ اسکلت بود، دست‌نخورده. `Debug` (بالایِ فایل مشتق‌شده) و `Display` (همین الان نوشته‌شده) دو ابرصفتِ `Error`اند، و تا این خط می‌رسد، هر دو از قبل هستند — پس چیزی برایِ بدنه نمی‌ماند. `source()` پیش‌فرضش را نگه می‌دارد، `None` برمی‌گرداند؛ `EntryError` یک خطایِ ریشه‌ای است، هیچ‌وقت خودش نتیجه‌ی یک خطایِ دیگر نیست.

## `From<ParseIntError> for EntryError` — یک خط، یک گونه

```rust
EntryError::BadScore(source)
```

همان شکلی که ۱.۶.۵ یاد داد: خطایِ بیرونی را در همان یک گونه‌ای بپیچ که معنایش «این جورِ خاصِ شکستِ بیرونی» است. چیزِ دیگری برایِ تصمیم‌گیری نمانده — `BadScore` همیشه فقط یک معنا دارد.

## `parse_entry` — سه چک، به همان ترتیبی که مشخصات گفته بود

```rust
let (name, score_str) = line.split_once(':').unwrap_or((line, ""));
let name = name.trim();
if name.is_empty() {
    return Err(EntryError::BlankName);
}
let score: u32 = score_str.trim().parse()?;
if score > 9999 {
    return Err(EntryError::ScoreTooHigh(score));
}
```

`.split_once(':').unwrap_or((line, ""))` همان چیزی است که رفتارِ «یک خط بدونِ `:` مثلِ نیمه‌ی امتیازِ خالی» را می‌سازد — دقیقاً همان‌طور که کامنتِ مستندساز قول داده بود، بدونِ یک شاخه‌ی جداگانه برایش. `score_str.trim().parse()?` جایی است که بلوکِ `From` بالا واقعاً به‌کار می‌آید: یک `?` خالی، بدونِ هیچ `.map_err(...)`ای. چکِ `> 9999` فقط بعدِ یک پارسِ موفق اجرا می‌شود، برایِ همین `ScoreTooHigh` همیشه یک `u32`ِ واقعی و از قبل معتبر را حمل می‌کند، نه چیزی که فقط شبیهِ یکی بود.

## این درس واقعاً درباره‌ی چه بود

- **شکلِ خودِ صفت تعیین می‌کند یک `impl` چقدر می‌تواند کوچک باشد.** `Error: Debug + Display` به‌علاوه‌ی یک متدِ پیش‌فرض یعنی `impl Error` برایِ یک خطایِ ریشه‌ای اغلب خالی است — کار از قبل تویِ `Debug`/`Display` انجام شده، همان چیزی که به‌هرحال می‌نوشتی.
- **یک `String` و یک `Box<dyn Error>` هر دو ساختار را پاک می‌کنند؛ یک enum نه.** `EntryError` قابلِ `match` است. هیچ‌کدام از آن دو، لااقل بدونِ کارِ اضافه، این‌طور نیستند — کارِ اضافه‌یِ `Box<dyn Error>` دقیقاً موضوعِ [۲.۵.۲](../02-error-source-chains/README.fa.md) است.
- **نه هر گونه‌ای به یک `impl From` نیاز دارد.** `BlankName` و `ScoreTooHigh` هر دو مستقیم ساخته می‌شوند، همان‌جا که اطلاعاتش را از قبل داری؛ فقط `BadScore`، که یک نوعِ بیرونی را می‌پیچد، از یکی سود می‌برد.

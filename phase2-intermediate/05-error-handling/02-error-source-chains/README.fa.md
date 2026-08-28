# ۲.۵.۲ — زنجیره‌ی منشأ و `Box<dyn Error>`

## در یک نگاه

بعد از این درس می‌توانی:

- برایِ یک گونه‌ی خطا که خطایِ دیگری را در دلِ خودش پیچیده، `source()` را با امضایِ دقیقِ خودِ trait — `Option<&(dyn Error + 'static)>` — پیاده‌سازی کنی، نه هر امضایی که به نظرت درست می‌آید.
- یک زنجیره‌ی خطا را — از شکستِ سطحِ‌بالا تا ریشه‌ی نهایی‌اش — با یک حلقه دنبال کنی و هر حلقه‌اش را چاپ کنی.
- بینِ یک enum سفارشی (مالِ ۲.۵.۱) و `Box<dyn Error>` برایِ نوعِ برگشتیِ یک تابع انتخاب کنی، و در یک جمله بگویی هرکدام چه چیزی به‌ات می‌دهد و چه چیزی از دستت می‌گیرد.

**زمان:** حدود ۶۰ دقیقه · **پیش‌نیاز:**
[۲.۵.۱ — نوع‌های خطای سفارشی و `std::error::Error`](../01-custom-error-types/README.fa.md)،
[۲.۳.۷ — ارسالِ ایستا در برابرِ پویا، و ایمنیِ شیء](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.fa.md)

---

## چرا اهمیت دارد

۲.۵.۱ بهت یاد داد یک enum خطایِ سفارشی بسازی و `std::error::Error` رویش پیاده کنی — و همان‌جا یک متد را دست‌نخورده گذاشتی: `source()`. پیاده‌سازیِ پیش‌فرضش فقط `None` برمی‌گرداند، و درس فقط یک جمله دربارهٔ‌اش گفت: به‌اش نیاز داری وقتی خطاات یک خطایِ دیگر را در دلش پیچیده باشد. امروز همان جمله را کامل می‌کنیم.

فکرش را بکن: یک برنامه‌ی واقعی یک فایلِ کانفیگ را از رویِ دیسک می‌خواند. جایی که خطا واقعاً رخ می‌دهد معمولاً چند لایه پایین‌تر از جایی است که کاربر یا لاگ آن را می‌بیند — یک تابعِ کتابخانه‌ی استاندارد شکست خورده، یا سیستم‌عامل گفته «فایل پیدا نشد». اگر خطایِ سطحِ‌بالاییِ تو («نتوانستم کانفیگ را بارگذاری کنم») این ریشه را پنهان کند، هرکسی که بعداً بخواهد دیباگ کند — یا لاگِ ساختاریافته بنویسد، یا فقط رویِ یک نوعِ خاص از خطایِ زیربنایی دوباره تلاش کند — کورکورانه کار می‌کند. `source()` دقیقاً همین را حل می‌کند: راهی برایِ اینکه یک خطا به خطایِ پایین‌ترش اشاره کند، تا زنجیره‌ای از شکستِ سطحِ‌بالا تا علتِ ریشه‌ای شکل بگیرد.

یک سؤالِ دومِ دیگر هم هست که ۲.۵.۱ جوابش را نداد. enum سفارشیِ خودت عالی‌ست وقتی فراخواننده باید بینِ انواعِ شکست فرق بگذارد — رویِ یک timeout دوباره تلاش کند، رویِ یک خطایِ اعتبارسنجی از کاربر مقدارِ دیگری بخواهد. ولی خیلی وقت‌ها — مخصوصاً نزدیکِ لبه‌ی برنامه: یک `main`، یک اسکریپت، یک تابعِ سطحِ‌بالا که فقط می‌خواهد هرچه خطا شد را لاگ کند و خارج شود — فراخواننده اصلاً نمی‌خواهد `match` کند. برایِ آن حالت، یک enum سفارشی صرفاً وزنِ اضافه است. امروز `Box<dyn Error>` را می‌بینی — همان جعبه‌ی «هر خطایی»، ساخته‌شده دقیقاً برایِ آن لحظه — همراه با قیمتِ واقعی‌اش.

---

## مفهوم

### دوباره‌بینیِ `source()`: این‌بار واقعی

۲.۵.۱ فقط این را گفت: `std::error::Error` یک متد دارد به اسمِ `source()`، با یک پیاده‌سازیِ پیش‌فرض که `None` برمی‌گرداند، و مجبور نبودی بازنویسی‌اش کنی. امضایِ کاملش را اما هنوز ندیده بودی:

```rust
fn source(&self) -> Option<&(dyn Error + 'static)>
```

سه تکه، هرکدام یک دلیل. یک `Option`، چون خیلی از خطاها اصلاً چیزی را در خودشان نپیچیده‌اند — یک `MissingField` خودش ریشه‌ی مشکل است، نه لایه‌ای رویِ یک خطایِ پایین‌تر. یک ارجاع، چون `source()` مالکیتِ خطایِ زیربنایی را نمی‌گیرد، فقط نشانش می‌دهد؛ خودِ `self` هنوز مالکِ آن است. و یک شیءِ صفتی — `dyn Error` — چون خطایی که `self` در خودش پیچیده می‌تواند از هر نوعی باشد؛ `source()` نمی‌تواند از قبل بداند دقیقاً کدام نوعِ ملموس، دقیقاً همان [ایده‌ی پاک‌شدنِ نوع که ۲.۳.۷](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.fa.md) رویِ `dyn Summarize` نشانت داد.

آن `'static` آخر تنها چیزی‌ست که شاید غافلگیرت کند. معنایش این نیست که مقدار برایِ همیشه زنده می‌ماند — معنایش این است که نوعِ زیرِ آن شیءِ صفتی نباید خودش یک ارجاعِ قرضیِ کوتاه‌عمر را نگه داشته باشد؛ باید مالکِ داده‌ی خودش باشد. تقریباً هر خطایی که می‌نویسی همین‌طوری‌ست — یک `String`، یک `io::Error` که خودش چیزی را قرض نگرفته — پس تویِ عمل به‌ندرت واقعاً محدودت می‌کند؛ فقط باید صریح نوشته شود، چون Rust اینجا چیزی را از رویِ حدس استنتاج نمی‌کند.

حالا همین را رویِ یک enum واقعی پیاده کن. `ConfigError` را یک قدم جلوتر از ۲.۵.۱ می‌بریم: این‌بار کانفیگ از یک فایلِ واقعی رویِ دیسک می‌آید، پس یک گونه‌ی تازه لازم است — `Io`، برایِ وقتی خودِ فایل باز نمی‌شود:

```rust
enum ConfigError {
    Io(io::Error),
    MissingField(String),
    InvalidNumber { field: String, source: ParseIntError },
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
```

`Io` و `InvalidNumber` هرکدام یک خطایِ دیگر را در خودشان پیچیده‌اند، پس `Some` برمی‌گردانند. `MissingField` خودش تهِ خط است — هیچ خطایِ پایین‌تری وجود ندارد که به‌اش اشاره کند — پس `None`. این دقیقاً همان تمایزی‌ست که بخشِ بعد رویش راه می‌رود.

```senpai-visual
{"kind":"borrowing","labels":["ConfigError::Io owns the io::Error","source() returns &","caller borrows it, never takes ownership"]}
```

اجرایش کن و ببین:

```rust
let bad_path = ConfigError::Io(io::Error::new(io::ErrorKind::NotFound, "no such file"));
let missing = ConfigError::MissingField("name".to_string());
let invalid = ConfigError::InvalidNumber {
    field: "max_retries".to_string(),
    source: "not-a-number".parse::<u32>().unwrap_err(),
};

report("bad_path", &bad_path);
report("missing", &missing);
report("invalid", &invalid);
```

```text
bad_path: could not read config file: no such file
  source: Some(no such file)
missing: missing required field: name
  source: None
invalid: invalid number for field 'max_retries': invalid digit found in string
  source: Some(invalid digit found in string)
```

### پیمایشِ زنجیره: یک حلقه که تا `None` می‌رود

حالا که `source()` واقعی‌ست، می‌توانی از خطایِ سطحِ‌بالا شروع کنی و لایه به لایه پایین بروی — یک زنجیره. الگویش همیشه یکی‌ست: خطا را چاپ کن، بعد تا وقتی `source()` چیزی برمی‌گرداند، همان را هم چاپ کن:

```rust
println!("error: {top}");
let mut cause = top.source();
while let Some(err) = cause {
    println!("caused by: {err}");
    cause = err.source();
}
```

این را رویِ یک خطایِ واقعی امتحان کن — نه یک `io::Error` دست‌ساز، بلکه یکی که واقعاً از تلاش برایِ خواندنِ یک فایلِ ناموجود بیرون می‌آید:

```text
error: could not read config file: The system cannot find the path specified. (os error 3)
caused by: The system cannot find the path specified. (os error 3)
```

پیامِ سیستم‌عامل دو بار دیده می‌شود — یک‌بار چون `Display`ِ خودِ `ConfigError::Io` آن را با `{e}` در دلِ پیامِ یک‌خطی‌اش جا داده، یک‌بار دیگر چون همین‌جا صریحاً `source()` را هم پیمایش کردی. این طبیعی‌ست، نه تکرارِ اضافه: `Display` و `source()` برایِ دو مخاطبِ متفاوت‌اند. `Display` برایِ یک انسان است که یک خط می‌خواند؛ `source()` برایِ کدی‌ست که می‌خواهد هر لایه را جدا بازرسی یا لاگ کند — مثلاً یک سیستمِ لاگِ ساختاریافته که هر لایه را در یک فیلدِ جدا می‌نویسد، نه در یک رشته‌ی تخت.

```senpai-visual
{"kind":"result","labels":["top-level ConfigError","source()","io::Error — root cause","source()","None — chain ends"]}
```

### `Box<dyn Error>`: یک جعبه‌ی «هر خطایی»

فرض کن یک تابع دو کارِ متفاوت انجام می‌دهد: اندازه‌ی یک فایلِ کانفیگ را می‌خواند (که می‌تواند با `ConfigError` شکست بخورد) و یک عددِ retry را از یک رشته پارس می‌کند (که می‌تواند با `ParseIntError` شکست بخورد — یک نوعِ کاملاً بی‌ربط، مستقیم از کتابخانه‌ی استاندارد). اگر امضایِ تابع `Result<_, ConfigError>` باشد، خطِ دومِ آن اصلاً کامپایل نمی‌شود — `?` نمی‌داند چطور یک `ParseIntError` را به `ConfigError` تبدیل کند، چون هیچ `From`ی برایش ننوشته‌ای. راهِ حل نوشتنِ یک `From<ParseIntError> for ConfigError` است — ولی این کار را برایِ *هر* ترکیبی از نوعِ خطا در *هر* تابعی تکرار کن، و enum سفارشی‌ات تبدیل می‌شود به یک لیستِ بلندِ `impl From` که هیچ ربطی به منطقِ واقعیِ برنامه‌ات ندارد. خروجیِ دقیقِ این خطا در «خطاهایی که خواهی دید» است.

`Box<dyn Error>` این مشکل را از ریشه کنار می‌زند. [۲.۳.۷ دقیقاً همین ایده را](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.fa.md) — یک شیءِ صفتی که نوعِ ملموسِ زیرش پاک شده — رویِ `Summarize` نشانت داد؛ همان `dyn Trait` و همان پاک‌شدنِ نوع، این‌بار رویِ `Error`. کتابخانه‌ی استاندارد یک `From<E>` نوشته که برایِ **هر** نوعِ `E` که `Error` را پیاده کرده کار می‌کند، به شرطِ اینکه نوعِ برگشتی‌ات `Box<dyn Error>` باشد — نه یک enum ثابت. یعنی `?` رویِ هر خطایی که `std::error::Error` را پیاده کرده باشد، بدونِ هیچ `impl From`ِ دستی، همین‌جوری کار می‌کند:

```rust
fn file_size(path: &str) -> Result<u64, ConfigError> {
    let contents = std::fs::read_to_string(path)?;
    Ok(contents.len() as u64)
}

fn run(path: &str, retry_count_text: &str) -> Result<(u64, u32), Box<dyn Error>> {
    let size = file_size(path)?;
    let retries: u32 = retry_count_text.parse()?;
    Ok((size, retries))
}
```

```text
file was 21 bytes, retry count was 5
```

`ConfigError` و `ParseIntError` هیچ‌کدام از وجودِ دیگری خبر ندارند — هیچ `impl From` بینشان نیست — و هر دو مستقیم به `Box<dyn Error>` تبدیل شدند. این همان چیزی‌ست که `Box<dyn Error>` واقعاً می‌فروشد: **یک** نوعِ برگشتی که هر نوعِ خطایی را که `Error` را پیاده کرده پوشش می‌دهد.

اینجا از `Box` عمداً فقط همین‌قدر استفاده می‌شود که لازم است، نه بیشتر: یک جعبه‌ی هیپ که هر خطایی را به یک اشاره‌گرِ هم‌اندازه تبدیل می‌کند تا نوعِ برگشتی‌ات ثابت بماند. داستانِ کاملِ `Box` — تخصیصِ هیپ، مالکیت، کِی به‌تنهایی سراغش بروی — مالِ [۲.۶.۱](../../06-smart-pointers/01-box-and-heap-allocation/README.fa.md) است، دقیقاً همان‌طور که [۲.۳.۷](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.fa.md) هم آن را زودتر، فقط برایِ همین یک نقش، امانت گرفت.

### هزینه‌یِ واقعی: دیگر نمی‌توانی `match` کنی

این راحتی رایگان نیست. وقتی نوعِ برگشتی‌ات `Box<dyn Error>` است، فراخواننده دیگر نمی‌داند پشتِ آن جعبه دقیقاً کدام enum یا ساختار نشسته — دقیقاً همان چیزی که پاک‌شدنِ نوع یعنی. `match err { ConfigError::Io(_) => ..., ... }` رویِ یک `Box<dyn Error>` اصلاً کامپایل نمی‌شود؛ کامپایلر نوعِ pattern را با نوعِ چیزی که رویش match می‌کنی مقایسه می‌کند، و آن‌ها دیگر یکی نیستند. خروجیِ دقیقش هم در «خطاهایی که خواهی دید» است.

تنها راهِ فرار `downcast_ref::<T>()` است: یک حدس — «شاید این یک `ConfigError` باشد» — که اگر درست باشد نوعِ ملموس را برمی‌گرداند، و اگر غلط باشد فقط `None` می‌دهد. جامع نیست، مثلِ `match`: کامپایلر مجبورت نمی‌کند هر حالتِ ممکن را پوشش بدهی، و اگر نوعی را که حدس نزده‌ای اتفاق بیفتد، بی‌سروصدا از قلم می‌افتد:

```rust
fn describe(err: &(dyn Error + 'static)) -> &'static str {
    if err.downcast_ref::<ConfigError>().is_some() {
        "a ConfigError"
    } else if err.downcast_ref::<ParseIntError>().is_some() {
        "a ParseIntError"
    } else {
        "something else"
    }
}
```

```text
a: config problem: missing field: name (a ConfigError)
b: invalid digit found in string (a ParseIntError)
```

یک نکته‌ی فنی: `downcast_ref()` فقط رویِ `dyn Error + 'static` تعریف شده — دقیقاً همان کراندی که `source()` هم داشت — پس امضایِ `describe` باید صریح آن را بنویسد، نه `dyn Error` خام. همان قاعده، دو جا.

```senpai-visual
{"kind":"concept","labels":["ConfigError","ParseIntError","Box<dyn Error>","print or log: works everywhere","match on variant: needs downcast_ref, one guess at a time"]}
```

### enum سفارشی یا `Box<dyn Error>`؟

هیچ‌کدام همیشه درست نیست — سؤال این است که فراخواننده واقعاً به چه چیزی نیاز دارد:

| محور | enum سفارشی (۲.۵.۱) برنده است وقتی... | `Box<dyn Error>` برنده است وقتی... |
|---|---|---|
| رفتارِ فراخواننده | باید بینِ گونه‌ها `match` کند و کارِ متفاوتی انجام بدهد | فقط می‌خواهد چاپ کند، لاگ کند، یا بالا پرتاب کند |
| تعدادِ نوعِ خطایِ زیربنایی | معمولاً چندتا، همه از قبل شناخته‌شده | می‌تواند هر تعداد نوعِ نامرتبط باشد |
| جایگاه در برنامه | لایه‌ی منطقِ اصلی، جایی که تصمیم‌گیری واقعی اتفاق می‌افتد | نزدیکِ لبه — `main`، یک اسکریپت، یک هندلرِ سطحِ‌بالا |
| هزینه | یک `impl From` به‌ازایِ هر تبدیل | نوعِ ملموس گم می‌شود؛ فقط با حدس و `downcast_ref` برمی‌گردد |

قاعده‌ی کوتاه: پیش‌فرض همان enum سفارشیِ ۲.۵.۱ است، هرجا فراخواننده واقعاً باید تصمیم بگیرد. `Box<dyn Error>` را برایِ همان نقطه‌هایی نگه دار که خطا فقط قرار است دیده شود، نه هندل شود. و یک نکته برایِ جلوتر: هر دویِ این راه‌ها — نوشتنِ دستیِ `Display` و `impl From` برایِ یک enum، یا زندگی با `Box<dyn Error>`ِ بی‌نام — دقیقاً همان کارِ تکراری‌ست که دو کریتِ `thiserror` و `anyhow` برایِ خودکارسازی‌اش ساخته شده‌اند؛ [۲.۵.۳](../03-thiserror-and-anyhow/README.fa.md) نشانت می‌دهد چطور.

---

## دست‌به‌کد

```sh
cargo run -p p2-05-02-error-source-chains --example 01-hand-written-source
cargo run -p p2-05-02-error-source-chains --example 02-walking-the-chain
cargo run -p p2-05-02-error-source-chains --example 03-box-dyn-error-unifies-return-types
cargo run -p p2-05-02-error-source-chains --example 04-the-cost-and-downcasting
```

بعد سه‌تایِ خراب:

```sh
cargo run -p p2-05-02-error-source-chains --example 05-source-wrong-lifetime-broken --features broken
cargo run -p p2-05-02-error-source-chains --example 06-cannot-convert-without-box-broken --features broken
cargo run -p p2-05-02-error-source-chains --example 07-cannot-match-a-boxed-error-broken --features broken
```

بعد این‌ها را امتحان کن:

۱. در `01-hand-written-source.rs`، یک گونه‌ی تازه اضافه کن — مثلاً `ConfigError::Empty` برایِ یک فایلِ کانفیگِ کاملاً خالی — و `source()`ش را هم بنویس. باید `Some` باشد یا `None`؟
۲. در `02-walking-the-chain.rs`، به‌جایِ یک مسیرِ ناموجود، یک فایل واقعاً بساز که وجود دارد ولی خواندنش اجازه ندارد (یا هر راهِ دیگری برایِ گرفتنِ یک `io::Error` واقعیِ متفاوت پیدا کن) و ببین پیامِ چاپ‌شده چطور فرق می‌کند.
۳. در `04-the-cost-and-downcasting.rs`، یک نوعِ خطایِ سومی اضافه کن که `describe` اصلاً حدسش را نمی‌زند و ببین چه چاپ می‌شود.

---

## خطاهایی که خواهی دید

### تطبیق‌نداشتنِ امضایِ `impl` — بدونِ کدِ خطا

```text
error: `impl` item signature doesn't match `trait` item signature
   --> phase2-intermediate\05-error-handling\02-error-source-chains\examples\05-source-wrong-lifetime-broken.rs:26:5
    |
 26 |     fn source(&self) -> Option<&dyn Error> {
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ found `fn(&'1 ConfigError) -> Option<&'1 (dyn std::error::Error + '1)>`
    |
   ::: C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\error.rs:111:5
    |
111 |     fn source(&self) -> Option<&(dyn Error + 'static)> {
    |     -------------------------------------------------- expected `fn(&'1 ConfigError) -> Option<&'1 (dyn std::error::Error + 'static)>`
    |
    = note: expected signature `fn(&'1 ConfigError) -> Option<&'1 (dyn std::error::Error + 'static)>`
               found signature `fn(&'1 ConfigError) -> Option<&'1 (dyn std::error::Error + '1)>`
    = help: the lifetime requirements from the `impl` do not correspond to the requirements in the `trait`
    = help: verify the lifetime relationships in the `trait` and `impl` between the `self` argument, the other inputs and its output
```

**کامپایلر به چه اعتراض دارد:** نوشتنِ `Option<&dyn Error>` (بدونِ `+ 'static`) یک امضایِ متفاوت از چیزی‌ست که trait اعلام کرده. وقتی `'static` را ننویسی، Rust برایِ آن ارجاع همان قاعده‌ی حذفِ طولِ‌عمرِ معمولیِ متدها را به‌کار می‌برد و آن را به طولِ‌عمرِ `&self` گره می‌زند — نه به `'static`. خودِ کامپایلر همین را با دو امضایِ کنارِ هم نشان می‌دهد: `impl`ی که نوشتی در برابرِ چیزی که trait می‌خواهد.

**راه‌حل:** دقیقاً همان چیزی را بنویس که trait اعلام کرده:

```rust
fn source(&self) -> Option<&(dyn Error + 'static)> {
    Some(&self.inner)
}
```

**چرا این راه‌حل است:** امضایِ یک متدِ trait، وقتی `impl`ش می‌کنی، اختیاری نیست — باید دقیقاً همان باشد (با حذفِ طولِ‌عمرِ یکسان یا با طولِ‌عمرِ صریحِ یکسان). این جا trait صراحتاً `'static` نوشته، پس `impl` هم باید همان را بنویسد؛ نمی‌شود با یک امضایِ «تقریباً همان» جایش را گرفت.

### `E0271` — بدونِ `From`، `?` نمی‌تواند تبدیل کند

```text
error[E0271]: type mismatch resolving `<u32 as FromStr>::Err == ConfigError`
  --> phase2-intermediate\05-error-handling\02-error-source-chains\examples\06-cannot-convert-without-box-broken.rs:26:27
   |
26 |     let value: u32 = text.parse()?;
   |                           ^^^^^ expected `ConfigError`, found `ParseIntError`

For more information about this error, try `rustc --explain E0271`.
```

**کامپایلر به چه اعتراض دارد:** `text.parse()` رویِ شکست یک `ParseIntError` می‌دهد. تابع `Result<u32, ConfigError>` برمی‌گرداند، و `?` برایِ تبدیلِ خطا دنبالِ `impl From<ParseIntError> for ConfigError` می‌گردد. چون چنین `impl`ی نوشته نشده، کامپایلر دو نوع را کنارِ هم می‌گذارد و می‌گوید یکی نیستند.

**راه‌حل:** یا یک `impl From<ParseIntError> for ConfigError` بنویس، یا — دقیقاً موضوعِ این درس — نوعِ برگشتی را `Box<dyn Error>` کن تا اصلاً نیازی به آن `impl` نباشد:

```rust
fn read_retries(text: &str) -> Result<u32, Box<dyn Error>> {
    let value: u32 = text.parse()?;
    Ok(value)
}
```

**چرا این راه‌حل است:** `From<E> for Box<dyn Error>` را کتابخانه‌ی استاندارد یک‌بار، برایِ هر `E: Error`، نوشته. با تغییرِ نوعِ برگشتی، دیگر خودت مجبور نیستی این تبدیل را برایِ هر جفت نوعی از نو بنویسی.

### `E0308` — یک `Box<dyn Error>` را نمی‌شود مثلِ enumِ اصلی‌اش `match` کرد

```text
error[E0308]: mismatched types
  --> phase2-intermediate\05-error-handling\02-error-source-chains\examples\07-cannot-match-a-boxed-error-broken.rs:37:9
   |
36 |     match err {
   |           --- this expression has type `Box<dyn std::error::Error>`
37 |         ConfigError::Io(_) => println!("io"),
   |         ^^^^^^^^^^^^^^^^^^ expected `Box<dyn Error>`, found `ConfigError`
   |
   = note: expected struct `Box<dyn std::error::Error>`
                found enum `ConfigError`

error[E0308]: mismatched types
  --> phase2-intermediate\05-error-handling\02-error-source-chains\examples\07-cannot-match-a-boxed-error-broken.rs:38:9
   |
36 |     match err {
   |           --- this expression has type `Box<dyn std::error::Error>`
37 |         ConfigError::Io(_) => println!("io"),
38 |         ConfigError::MissingField(_) => println!("missing"),
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `Box<dyn Error>`, found `ConfigError`
   |
   = note: expected struct `Box<dyn std::error::Error>`
                found enum `ConfigError`

For more information about this error, try `rustc --explain E0308`.
```

**کامپایلر به چه اعتراض دارد:** `err` نوعش `Box<dyn Error>` است، ولی هر دو بازویِ `match` الگوهایی از نوعِ `ConfigError` نوشته‌اند. یک pattern باید همان نوعِ چیزی را داشته باشد که رویش match می‌کنی؛ این دو یکی نیستند، دقیقاً همان‌طور که «هزینه‌یِ واقعی» گفت.

**راه‌حل:** یا اصلاً `Box<dyn Error>` نساز (نوعِ برگشتی‌ات را `ConfigError` نگه دار، اگر فراخواننده واقعاً باید `match` کند)، یا اول `downcast_ref::<ConfigError>()` کن و رویِ نتیجه‌اش کار کن:

```rust
if let Some(config_err) = err.downcast_ref::<ConfigError>() {
    match config_err {
        ConfigError::Io(_) => println!("io"),
        ConfigError::MissingField(_) => println!("missing"),
    }
}
```

**چرا این راه‌حل است:** `downcast_ref()` دقیقاً چیزی را برمی‌گرداند که `match` به‌اش نیاز دارد — یک `&ConfigError` واقعی، نه `&Box<dyn Error>`. ولی دقت کن: این فقط برایِ حالتی کار می‌کند که از قبل حدس زده باشی نوعِ زیرین `ConfigError` است؛ برخلافِ `match` رویِ enum خودش، اینجا هیچ‌چیز کامپایلر را مجبور نمی‌کند هر نوعِ ممکن را پوشش بدهی.

---

## تمرین

### گرم‌کردن

<details>
<summary>یک <code>impl Error for MyError</code> می‌نویسی و <code>source()</code> را بازنویسی نمی‌کنی. <code>my_error.source()</code> چه برمی‌گرداند؟</summary>

`None` — پیاده‌سازیِ پیش‌فرضِ trait همیشه همین را می‌دهد، همان چیزی که ۲.۵.۱ نشانت داد.

</details>

<details>
<summary>این کامپایل می‌شود؟</summary>

```rust
impl std::error::Error for MyError {
    fn source(&self) -> Option<&dyn std::error::Error> {
        None
    }
}
```

</details>

<details>
<summary>پاسخ</summary>

نه. امضا با چیزی که trait اعلام کرده یکی نیست — طولِ‌عمرِ حذف‌شده به `&self` گره خورده، نه `'static`. دقیقاً همان خطایی که بالا دیدی.

</details>

<details>
<summary>یک تابع <code>Result&lt;T, Box&lt;dyn Error&gt;&gt;</code> برمی‌گرداند. فراخواننده می‌تواند بدونِ هیچ کارِ اضافه بفهمد دقیقاً کدام نوعِ ملموسِ خطا رخ داده؟</summary>

نه — نوعِ ملموس پاک شده. تنها راه `downcast_ref::<T>()` است، و آن هم فقط وقتی جواب می‌دهد که از قبل حدس زده باشی `T` چیست.

</details>

<details>
<summary><code>ConfigError::Io</code> یک <code>io::Error</code> را در خودش پیچیده، و <code>ConfigError::MissingField</code> هیچ‌چیزی را نپیچیده. <code>source()</code>ِ هرکدام باید چه برگرداند؟</summary>

`Io` باید `Some(&io_error)` برگرداند؛ `MissingField` باید `None` برگرداند، چون خودش تهِ زنجیره است — هیچ خطایِ پایین‌تری وجود ندارد.

</details>

<details>
<summary>این چه چاپ می‌کند؟ (فرض کن <code>err</code> یک <code>ConfigError::Io</code> است که خودِ <code>io::Error</code>ش هیچ <code>source()</code>ی ندارد)</summary>

```rust
let mut cause = err.source();
let mut count = 0;
while let Some(c) = cause {
    count += 1;
    cause = c.source();
}
println!("{count}");
```

</details>

<details>
<summary>پاسخ</summary>

```text
1
```

حلقه یک‌بار اجرا می‌شود — رویِ همان `io::Error`ی که `err` در خودش پیچیده — و چون آن `io::Error` خودش `source()`ی ندارد، حلقه همان‌جا تمام می‌شود.

</details>

### تعمیر

هر سه مثالِ خراب را درست کن — نه با یک ترفندِ نحوی، بلکه با تغییرِ همان تصمیمی که مثال خرابش کرده:

۱. `examples/05-source-wrong-lifetime-broken.rs` — امضایِ `source()` را دقیقاً همان چیزی کن که trait می‌خواهد.
۲. `examples/06-cannot-convert-without-box-broken.rs` — نوعِ برگشتیِ `read_retries` را طوری عوض کن که `?` بدونِ نوشتنِ یک `impl From` تازه کار کند.
۳. `examples/07-cannot-match-a-boxed-error-broken.rs` — قبل از `match`، اول نوعِ ملموس را با `downcast_ref()` پس بگیر.

### پیاده‌سازی

پنج جا در `src/lib.rs` — `parse_config_str` کاملاً نوشته شده (مهارتِ ۲.۵.۱ است، نه این درس)؛ این پنج‌تا مالِ امروزند:

```sh
cargo test -p p2-05-02-error-source-chains
```

- `impl From<std::io::Error> for ConfigError`
- `source()` — امضایِ دقیقِ trait، `Some`/`None` بر اساسِ گونه
- `load_config` — می‌خواند، بعد پارس می‌کند
- `error_chain` — پیامِ خودِ خطا، بعد هر لایه‌یِ `source()` تا `None`
- `effective_max_retries` — دو نوعِ خطایِ ناهم‌ریشه، یک `Box<dyn Error>`

مشخصاتِ دقیقِ هرکدام — از جمله رفتارِ دقیقِ هر حالت — در کامنتِ مستنداتِ بالایِ خودِ تابع است.

### بساز

یک نوعِ خطایِ کوچکِ خودت طراحی کن که یک خطایِ دیگر را در خودش می‌پیچد — هرچیزی، مثلاً یک خطایِ «بارگذاریِ manifest» که یک `std::env::VarError` یا یک `serde`-مانند parse error را در خودش دارد (لازم نیست واقعاً `serde` را بیاوری؛ خودت یک خطایِ ساده‌ی دیگر بساز). `source()`ش را درست بنویس، یک بار واقعاً به‌اش بربخور (نه دست‌ساز — واقعاً کاری کن که رخ بدهد)، و زنجیره‌اش را با الگویِ همین درس چاپ کن. در یک کامنت بنویس: کدام گونه‌ی این enum باید `None` برگرداند، و چرا.

### چالش (اختیاری)

از تابعِ `effective_max_retries`ت یک `Box<dyn Error>` بگیر و با `downcast_ref()` رویِ هر دو نوعِ ممکن (`ConfigError` و `ParseIntError`) match کن — دقیقاً الگویِ `describe` در «هزینه‌ی واقعی»، این‌بار مالِ خودت.

بعد کمی جلوتر را نگاه کن: enum سفارشیِ ۲.۵.۱ به یک `impl Display` دستی و یک مشتِ `impl From` نیاز داشت؛ `Box<dyn Error>`ِ امروز آن‌ها را دور زد ولی نوعِ ملموس را قربانی کرد. دو کریت — `thiserror` و `anyhow` — دقیقاً برایِ رفعِ همین دو دردسر ساخته شده‌اند: یکی برایِ enum سفارشی‌ها بویلرپلیتِ `Display`/`From` را خودکار می‌کند، دیگری تجربه‌ی کار با `Box<dyn Error>` را ارگونومیک‌تر می‌کند. بدونِ نصبِ چیزی، مستنداتِ `docs.rs` هرکدام را نگاه کن و حدس بزن کدام‌یک کدام مشکل را حل می‌کند — [۲.۵.۳](../03-thiserror-and-anyhow/README.fa.md) جوابت را می‌گوید.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| `source()` | متدِ صفتِ `Error` که خطایِ زیربنایی را برمی‌گرداند: `Option<&(dyn Error + 'static)>` | هر گونه‌ای که یک خطایِ دیگر را در خودش می‌پیچد |
| زنجیره‌ی خطا (error chain) | پیمایشِ پی‌درپیِ `source()` از یک خطایِ سطحِ‌بالا تا ریشه‌ی نهایی | لاگِ ساختاریافته، پیامِ کاملِ خطا برایِ دیباگ |
| `Box<dyn Error>` | ظرفِ نوع‌پاک‌شده‌یِ «هر خطایی»؛ یک نوعِ برگشتی برایِ هر خطایی که `Error` را پیاده کرده | نزدیکِ لبه‌ی برنامه، جایی که فقط قرار است خطا دیده شود |
| بازگرداندنِ نوع (`downcast_ref()`) | برگرداندنِ نوعِ ملموس از یک `dyn Error + 'static`، وقتی از قبل حدس می‌زنی چیست | تنها راهِ فرار از هزینه‌یِ `Box<dyn Error>`، نه یک جایگزینِ `match` |

### الان می‌دانی

- `source()` امضایِ ثابتی دارد — `Option<&(dyn Error + 'static)>` — و نوشتنِ چیزی «تقریباً همان» کامپایل نمی‌شود.
- گونه‌ای که خطایِ دیگری را در خودش پیچیده `Some` برمی‌گرداند؛ گونه‌ای که خودش ریشه‌ی مشکل است `None` برمی‌گرداند.
- پیمایشِ `source()` تا `None` یک زنجیره می‌سازد؛ همین الگو پشتِ هر ابزارِ لاگِ خطایِ ساختاریافته‌ای‌ست که تا حالا دیده‌ای.
- `Box<dyn Error>` هر نوعِ خطایی را که `Error` را پیاده کرده، بدونِ نوشتنِ هیچ `impl From`ی، پشتِ یک نوعِ برگشتیِ واحد جمع می‌کند.
- آن راحتی رایگان نیست: پشتِ `Box<dyn Error>`، دیگر نمی‌توانی `match` کنی؛ فقط `downcast_ref()` داری، حدس‌به‌حدس، نه جامع.
- انتخاب بینِ enum سفارشی و `Box<dyn Error>` به این برمی‌گردد که فراخواننده واقعاً باید بینِ گونه‌ها تصمیم بگیرد، یا فقط باید خطا را ببیند.

### بعداً کامل‌تر می‌بینی

- **`Box<T>` و تخصیصِ هیپ، کامل** — این درس فقط از `Box` به‌عنوانِ ابزاری برایِ یک‌اندازه‌کردنِ نوعِ برگشتی استفاده کرد؛ داستانِ کاملش — [۲.۶.۱ — `Box` و تخصیصِ هیپ](../../06-smart-pointers/01-box-and-heap-allocation/README.fa.md).
- **`thiserror` و `anyhow`** — بویلرپلیتِ دستیِ همین درس (`impl Display`، `impl From`، انتخاب بینِ enum و `Box<dyn Error>`) دقیقاً همان چیزی‌ست که این دو کریت خودکارش می‌کنند — [۲.۵.۳ — `thiserror` در برابرِ `anyhow`](../03-thiserror-and-anyhow/README.fa.md).
- **طراحیِ یک رده‌بندیِ کاملِ خطا برایِ یک سرویس** — امروز فقط یک `ConfigError` کوچک دیدی؛ طراحیِ سلسله‌مراتبِ خطا برایِ یک سرویسِ واقعی، لایه به لایه — [۲.۵.۴ — طراحیِ رده‌بندیِ خطا برایِ یک سرویس](../04-error-taxonomy-for-a-service/README.fa.md).
- **`Send`/`Sync` رویِ `Box<dyn Error>`** — `Box<dyn Error>`ِ خام به‌تنهایی قولِ عبور از مرزِ یک ریسه را نمی‌دهد؛ برایِ آن به `Box<dyn Error + Send + Sync>` نیاز داری — [۲.۸.۴ — `Send` و `Sync`](../../08-concurrency/04-send-and-sync/README.fa.md).

### می‌توانی توضیح بدهی؟

- چرا `Option<&dyn Error>` (بدونِ `'static`) امضایِ درستی برایِ `source()` نیست؟
- یک enum با سه گونه داری، دوتا خطایِ دیگر را در خودشان می‌پیچند و یکی نه. `source()`ِ هرکدام چه باید برگرداند، و چرا؟
- `Box<dyn Error>` چطور به `?` اجازه می‌دهد بینِ نوع‌هایِ خطایِ کاملاً بی‌ربط حرکت کند، بدونِ هیچ `impl From`ی؟
- چرا `match err { ConfigError::Io(_) => ... }` رویِ یک `Box<dyn Error>` کامپایل نمی‌شود، ولی رویِ یک `ConfigError` خام می‌شود؟
- `downcast_ref()` را با `match` مقایسه کن: هرکدام چه تضمینی می‌دهد که دیگری نمی‌دهد؟
- برایِ یک تابعِ واقعی که خودت می‌شناسی (از یک پروژه، یا از فازِ ۱)، بگو enum سفارشی برایش بهتر بود یا `Box<dyn Error>` — و چرا.

---

## بیشتر

- [مستنداتِ `std::error::Error`](https://doc.rust-lang.org/std/error/trait.Error.html) — امضایِ کاملِ `source()`، از زبانِ خودِ کتابخانه‌ی استاندارد.
- [ماژولِ `std::error`](https://doc.rust-lang.org/std/error/index.html) — همان‌جایی که `impl<E: Error> From<E> for Box<dyn Error>` واقعاً نوشته شده.
- [کتابِ Rust — فصلِ ۹، مدیریتِ خطا](https://doc.rust-lang.org/book/ch09-00-error-handling.html) — دیدِ کلی‌ترِ همین موضوع، از زبانِ خودِ تیمِ Rust.
- [مستنداتِ `Box<T>`](https://doc.rust-lang.org/std/boxed/struct.Box.html) — همان ابزاری که [۲.۳.۷](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.fa.md) برایِ باندازه‌کردنِ اشیایِ صفتی به‌کار برد، امروز رویِ `dyn Error`.

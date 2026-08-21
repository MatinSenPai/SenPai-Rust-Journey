# ۱.۷.۱ — پروژه‌ی کوچکِ راهنمایی‌شده

## در یک نگاه

بعد از این درس می‌توانی:

- بگویی چرا امضای هر متدِ `Watchlist` — `add`، `find`، `titles` — یک تصمیمِ مالکیت است، نه فقط یک انتخابِ نحوی، و آن تصمیم را به یکی از درس‌های پیش وصل کنی.
- یک واریانتِ تازه به یک `enum` اضافه کنی و بگذاری کامپایلر خودش فهرستِ کاملِ جاهایی که باید عوض شوند را بدهد.
- یک عنوانِ فارسی را به‌درستی به تعدادِ معینی نویسه — نه بایت — کوتاه کنی، بی هیچ ریسکِ panic.
- یک `enum` خطا با `From` بسازی و بگذاری `?` خودش تبدیلش کند، به‌جای نوشتنِ `match` دستی روی هر خطا.

**زمان:** حدود ۹۰ دقیقه · **پیش‌نیاز:**
[۱.۵.۲ — الگوی newtype](../../05-your-own-types/02-tuple-structs-and-newtype/README.fa.md) ·
[۱.۵.۳ — Enumها به‌عنوانِ داده](../../05-your-own-types/03-enums-as-data/README.fa.md) ·
[۱.۵.۴ — `match` در عمق](../../05-your-own-types/04-match-in-depth/README.fa.md) ·
[۱.۴.۴ — برشِ امنِ متن](../../04-text-and-strings/04-slicing-text-safely/README.fa.md) ·
[۱.۶.۳ — `Result` و عملگرِ `?`](../../06-absence-and-failure/03-result-and-question-mark/README.fa.md)

و راستش، تقریباً هر چیزی که از ۱.۱.۱ تا این‌جا خوانده‌ای.

---

## چرا اهمیت دارد

سی درس گذشته، سی تکه بودند. `enum`، `struct`، `&`، `String` در برابرِ `&str`، `Option`، `Result` — هرکدام در درسِ خودش، جدا از بقیه، با مثال‌های خودش.

این درس هیچ مفهومِ تازه‌ای نمی‌آموزد. کارش نشان‌دادنِ این است که آن سی تکه، وقتی کنارِ هم بگذاری‌شان، یک برنامه‌ی واقعی می‌سازند — نه یک برنامه‌ی اسباب‌بازی که فقط یک ایده را نشان می‌دهد، یک چیزی که واقعاً می‌شود از آن استفاده کرد.

برنامه یک **فهرستِ تماشا (watchlist)** است: کتابخانه‌ای در حافظه برای انیمه و سریال، که می‌شود بهش عنوان اضافه کرد، جست‌وجویش کرد، امتیاز داد، و خلاصه‌اش را خواند. همان چیزی که هزاران بار در MyAnimeList یا Notion خودت ساخته‌ای — این‌بار در Rust، و این‌بار با عنوان‌های فارسی، چون همین‌جاست که درسِ متن (فاز ۱.۴) بلافاصله جواب می‌دهد.

پنج مرحله دارد، هر مرحله یک زیربخشِ «مفهوم»، و هر مرحله کدی که واقعاً اجرا می‌کنی:

1. **داده** — یک `enum` که هر حالتش داده‌ی متفاوتی حمل می‌کند، و یک newtype که نمی‌شود غلط ساختش.
2. **مخزن** — یک `struct` با یک `Vec` داخلش، و سه متد که هرکدام یک تصمیمِ مالکیتِ متفاوت می‌گیرند.
3. **خواندنش** — یک `match` جامع که هر چهار حالت را می‌پوشاند، و چیزی که وقتی حالتِ پنجمی اضافه شود اتفاق می‌افتد.
4. **متنِ درست‌وحسابی** — یک خطِ خلاصه با `format!`، ستون‌های هم‌تراز، و کوتاه‌کردنِ عنوان به تعدادِ نویسه.
5. **شکست‌خوردنِ درست** — یک `Result` برای تنها عملیاتی که واقعاً می‌تواند شکست بخورد، و `?`ای که خودش تبدیلِ خطا را انجام می‌دهد.

نه کدِ جدیدی برای یاد گرفتن، نه قاعده‌ی جدیدی برای حفظ کردن. فقط این‌که ببینی سی درسِ گذشته، دستِ کم برای این یک برنامه، کافی بودند.

---

## مفهوم

### داده: یک enum با چهار شکل

فهرستِ تماشا یک چیز باید بتواند نگه دارد: این‌که هر عنوان الان کجای مسیر است. دقیقاً همان `Entry`ای که در [۱.۵.۳](../../05-your-own-types/03-enums-as-data/README.fa.md) ساختی، این‌بار با اسم‌های این درس:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
    Watching { episode: u32 },
    Finished { rating: Rating },
    Planned,
    Dropped { at: u32 },
}
```

چهار واریانت، سه شکلِ متفاوت — یکی بی‌داده، دو تا با داده‌ی نام‌دار. یک عنوان که «دارد تماشا می‌شود» بدونِ قسمت معنایی ندارد؛ یک عنوانِ «تمام‌شده» بدونِ امتیاز معنایی ندارد. `Status` این را با نوعش تضمین می‌کند، نه با یک بررسیِ زمانِ اجرا — همان بحثِ «حالتِ نامعتبر را نانوشتنی کن» از ۱.۵.۳.

### Rating: newtype‌ای که نمی‌شود غلط ساختش

امتیاز باید از ۰ تا ۱۰ باشد. می‌شد یک `u8` ساده گذاشت و در هر جایی که استفاده می‌شود بررسی کرد — ولی این یعنی هر جا که فراموش کنی، یک باگ خفته. همان کاری که [۱.۵.۲](../../05-your-own-types/02-tuple-structs-and-newtype/README.fa.md) با `Percent` کرد، این‌بار با `Rating`:

```rust
struct Rating(u8);

impl Rating {
    fn new(value: u8) -> Rating {
        if value > 10 {
            Rating(10)
        } else {
            Rating(value)
        }
    }

    fn value(self) -> u8 {
        self.0
    }
}
```

```text
Rating::new(9):   9
Rating::new(255): 10
```

فیلد خصوصی است، `new` تنها دری‌ست که می‌شود از آن وارد شد، و `new` به‌جای رد کردنِ ورودیِ غلط، آن را کلمپ می‌کند. نتیجه: از این نقطه به بعد، هرجا `Rating` در دست داری، دیگر لازم نیست دوباره بررسی کنی که کمتر از ۱۰ هست یا نه — نوع خودش این را ضمانت کرده. `self`، نه `&self`، روی `value` — چون `Rating` هشت بایتی و `Copy` است، دقیقاً همان استدلالِ ۱.۵.۲.

```text
$ cargo run -p p1-07-01-guided-mini-project --example 01-the-data
Entry { title: "Cowboy Bebop", status: Watching { episode: 9 } }
Entry { title: "Frieren", status: Finished { rating: Rating(9) } }
Entry { title: "حمله به تایتان", status: Dropped { at: 12 } }
Entry { title: "Bocchi the Rock!", status: Planned }

Rating::new(9):   9
Rating::new(255): 10
```

### مخزن: سه متد، سه تصمیمِ مالکیت

`Watchlist` فقط یک `Vec<Entry>` است. جالب‌ترین چیز درباره‌اش کدِ داخلش نیست — امضای متدهایش است:

```rust
struct Watchlist {
    entries: Vec<Entry>,
}

impl Watchlist {
    fn add(&mut self, entry: Entry) { /* ... */ }
    fn find(&self, title: &str) -> Option<&Entry> { /* ... */ }
    fn titles(&self) -> Vec<&str> { /* ... */ }
}
```

هر سه‌تا، یک تصمیمِ مالکیت‌اند که ۱.۲ و ۱.۳ برایش قاعده ساختند:

```senpai-visual
{"kind":"ownership","labels":["add(entry): مالکیت می‌گیرد","list صاحبش می‌شود","find(&self): فقط نگاه می‌کند","Option<&Entry> برمی‌گرداند","titles(&self): فقط نگاه می‌کند","Vec<&str> برمی‌گرداند"]}
```

**`add(&mut self, entry: Entry)`** — پارامتر `Entry` است، نه `&Entry`. فهرست قرار است این ورودی را تا هر وقت که خودش زنده است نگه دارد؛ این فقط وقتی ممکن است که فهرست **مالکِ** آن باشد. اگر یک ارجاع می‌گرفت، طول عمرِ فهرست به کسی که `add` را صدا زده گره می‌خورد — دقیقاً برعکسِ چیزی که می‌خواهیم. `&mut self` هم چون `push` کردن روی `self.entries` تغییرش می‌دهد.

**`find(&self, title: &str) -> Option<&Entry>`** — جست‌وجو معمولاً یعنی یک فیلد را بخوانی یا چیزی چاپ کنی. برگرداندنِ یک `Entry` مالکانه یعنی هر بار جست‌وجو، یک `String` کلون شود — همان حلقه‌ی «بی‌فایده»ی ۱.۲.۳، این‌بار داخلِ متدِ خودت. `&self` هم برای همین: `find` فقط نگاه می‌کند، چیزی را تغییر نمی‌دهد.

**`titles(&self) -> Vec<&str>`** — هر عنوان از قبل داخلِ یک `Entry` در `self.entries` مالک است. کلون کردنِ هرکدام فقط برای فهرست‌کردنشان، دوباره همان کلونِ بی‌فایده است. `Vec<&str>` یک وکتور از نگاه‌های قرضی‌ست، نه رونوشت‌ها — و چون همه‌شان از `&self` می‌آیند، بازگشتشان بدونِ نوشتنِ هیچ طول‌عمری کامپایل می‌شود؛ همان قاعده‌ی حذفِ طول‌عمر که در قرض‌گرفتن‌ها دیده بودی.

```text
$ cargo run -p p1-07-01-guided-mini-project --example 02-the-store
titles: ["Cowboy Bebop", "حمله به تایتان"]
found:     Entry { title: "Cowboy Bebop", status: Watching { episode: 9 } }
not found: Ghost in the Shell
still borrowed from list: true
```

آن خطِ آخر خودش گویاست: چیزی که `find` برگردانده هنوز از `list` قرض گرفته شده، و کامپایلر مطمئن می‌شود که `list` تا وقتی آن قرض زنده است، جابه‌جا یا نابود نمی‌شود — بدونِ این‌که تو یک کلمه درباره‌ی طولِ عمر نوشته باشی.

و اگر الان وسوسه شدی یک `HashMap<String, Entry>` بسازی تا `find` سریع‌تر شود: حق داری، ولی آن ابزار در فاز ۲ می‌آید. برای یک فهرستِ چند صد تایی، حلقه روی یک `Vec` دقیقاً به همین اندازه درست است.

### خواندنش: یک match جامع

حالا که فهرست را ساختیم، وقتِ نمایشِ آن است — و اینجاست که `match` روی `Status` وارد می‌شود:

```rust
fn describe(entry: &Entry) -> String {
    let detail = match entry.status {
        Status::Watching { episode } => format!("watching, episode {episode}"),
        Status::Finished { rating } => format!("finished, {rating}/10"),
        Status::Planned => "planned".to_string(),
        Status::Dropped { at } => format!("dropped at episode {at}"),
    };
    format!("{} — {detail}", entry.title)
}
```

```text
$ cargo run -p p1-07-01-guided-mini-project --example 03-reading-it-back
Cowboy Bebop — watching, episode 9
Frieren — finished, 9/10
Bocchi the Rock! — planned
حمله به تایتان — dropped at episode 12
```

چهار واریانت، چهار بازو، بدونِ `_`. این دقیقاً همان جامعیتِ [۱.۵.۴](../../05-your-own-types/04-match-in-depth/README.fa.md) است: کامپایلر شمرده که `Status` چند واریانت دارد و ثابت کرده هر چهارتا پوشش داده شده‌اند. حالا فرض کن محصول یک حالتِ پنجم می‌خواهد — «نگه‌داشته‌شده». یک واریانت اضافه می‌کنی، هیچ‌جای دیگر را دست نمی‌زنی، و کامپایلر خودش فهرستِ کاملِ جاهایی که باید عوض شوند را می‌دهد. بخشِ «خطاهایی که خواهی دید» دقیقاً همین را نشان می‌دهد.

### متنِ درست‌وحسابی: format! و کوتاه‌کردنِ امن

یک خلاصه‌ی خوب دو چیز لازم دارد: ستون‌هایی که هم‌تراز بمانند، و عنوان‌هایی که کوتاه شوند بدونِ این‌که چیزی بشکند.

هم‌تراز کردن، خودش رایگان است — پدینگِ `{:<width$}` در Rust بر اساسِ تعدادِ **نویسه** حساب می‌شود، نه بایت:

```text
$ cargo run -p p1-07-01-guided-mini-project --example 04-text-done-properly
Cowboy Beb…  watching
حمله به تا…  dropped
Frieren      finished
Bocchi the…  planned

4 entries — 1 watching, 1 finished, 1 planned, 1 dropped
```

ردیفِ فارسی درست به همان اندازه‌ی ردیف‌های انگلیسی جا می‌افتد — چیزی که در یک زبانِ دیگر که پدینگ را بر اساسِ بایت حساب می‌کند، ممکن نبود.

کوتاه‌کردن داستانِ دیگری‌ست. اینجا همان تابعِ [۱.۴.۴](../../04-text-and-strings/04-slicing-text-safely/README.fa.md) است، بدونِ هیچ تغییری:

```rust
fn truncate_to_chars(text: &str, max_chars: usize) -> &str {
    let mut seen = 0;
    for (index, _) in text.char_indices() {
        if seen == max_chars {
            return &text[..index];
        }
        seen += 1;
    }
    text
}
```

اندیسی که برمی‌گردد، خودش از `char_indices` آمده، پس همیشه یک مرزِ نویسه است — این برش هرگز panic نمی‌کند، نه روی «Cowboy Bebop»، نه روی «حمله به تایتان». همان تابع را با `&title[..10]` عوض کن (کدِ `card_title` ۱.۴.۴ را به خاطر بیاور) و روی این ردیفِ سوم — که ۲۵ بایت است ولی فقط ۱۴ نویسه — یا panic می‌گیری یا یک نویسه را نصفه می‌بری.

بیضی (…) فقط وقتی اضافه می‌شود که واقعاً چیزی بریده شده — با شمردنِ `.chars().count()`، نه `.len()`. «Frieren» هفت نویسه است، کمتر از سقفِ ده‌تایی، پس بدونِ بیضی می‌ماند؛ بقیه بریده و علامت‌دار می‌شوند.

و شمارشِ خط آخر — «۴ entry، ۱ watching، ...» — با چهار شمارنده‌ی جدا نوشته شده، نه یک `HashMap<Status, u32>`. برای چهار حالتِ شناخته‌شده، یک `Vec` (یا این‌جا حتی ساده‌تر: چهار متغیر) به همان اندازه خوانا است؛ کلیدِ باز-انتها همان‌جایی‌ست که `HashMap` فاز ۲ به کار می‌آید.

### شکست‌خوردنِ درست: تنها عملیاتی که می‌تواند شکست بخورد

از میانِ همه‌ی متدهای `Watchlist`، فقط یکی واقعاً می‌تواند شکست بخورد: **امتیاز دادن به عنوانی که در فهرست نیست.** `add`، `find`، `titles` — هیچ‌کدام `Result` برنمی‌گردانند، چون هیچ‌کدام نمی‌توانند شکست بخورند.

```rust
enum WatchlistError {
    NotFound(String),
    InvalidRating(String),
}

fn rate(&mut self, title: &str, rating: Rating) -> Result<(), WatchlistError> {
    for entry in &mut self.entries {
        if entry.title == title {
            entry.status = Status::Finished { rating };
            return Ok(());
        }
    }
    Err(WatchlistError::NotFound(title.to_string()))
}
```

ولی در دنیای واقعی، امتیاز از یک ورودیِ متنی می‌آید — و همان‌جا یک راهِ شکستِ دوم پیدا می‌شود: متن ممکن است اصلاً عدد نباشد. `.parse::<u8>()` در آن حالت `ParseIntError` برمی‌گرداند، نه `WatchlistError`. این‌جا `From` وارد می‌شود:

```rust
impl From<std::num::ParseIntError> for WatchlistError {
    fn from(err: std::num::ParseIntError) -> WatchlistError {
        WatchlistError::InvalidRating(err.to_string())
    }
}

fn rate_from_text(&mut self, title: &str, rating_text: &str) -> Result<(), WatchlistError> {
    let raw: u8 = rating_text.trim().parse()?;
    self.rate(title, Rating::new(raw))
}
```

آن `?` روی خطِ `.parse()` دقیقاً همین را می‌گوید: «اگر این `Err` بود، آن را با `From` تبدیل کن و همین الان از این تابع برگردان». هیچ `match`ی، هیچ بلوکِ برگشتِ زودهنگامی نوشته نشده — `?` خودش هم بازکردنِ مقدار را انجام می‌دهد و هم تبدیلِ نوعِ خطا را، چون `WatchlistError` می‌داند چطور از یک `ParseIntError` ساخته شود.

```text
$ cargo run -p p1-07-01-guided-mini-project --example 05-failing-well
Ok(())
Err(NotFound("Cowboy Bebop"))

Ok(())
Err(InvalidRating("invalid digit found in string"))
Err(InvalidRating("invalid digit found in string"))
```

خطِ آخر نکته‌ی ظریف‌تری دارد: عنوانِ «Cowboy Bebop» هم در فهرست نیست *و* متنِ امتیاز هم عدد نیست — و جوابی که می‌گیری `InvalidRating` است، نه `NotFound`. `parse` قبل از جست‌وجوی عنوان اجرا می‌شود؛ یک ورودیِ بد، همیشه به‌عنوانِ ورودیِ بد گزارش می‌شود، نه به‌عنوانِ عنوانِ گم‌شده.

---

## دست‌به‌کد

```sh
cargo run -p p1-07-01-guided-mini-project --example 01-the-data
cargo run -p p1-07-01-guided-mini-project --example 02-the-store
cargo run -p p1-07-01-guided-mini-project --example 03-reading-it-back
cargo run -p p1-07-01-guided-mini-project --example 04-text-done-properly
cargo run -p p1-07-01-guided-mini-project --example 05-failing-well
```

بعد آن یکیِ خراب:

```sh
cargo run -p p1-07-01-guided-mini-project --example 06-a-new-status --features broken
```

بعد این‌ها را امتحان کن:

۱. در `02-the-store`، خطِ کامنت‌شده را که `bebop` را دوباره چاپ می‌کند واقعاً بردار. کدامیک از خطاهای ۱.۲.۲ را می‌گیری؟
۲. در `04-text-done-properly`، `MAX_CHARS` را از ۱۰ به ۵ کاهش بده. کدام عنوان‌ها حالا با بیضی می‌آیند که قبلاً نمی‌آمدند؟
۳. در `05-failing-well`، ترتیبِ دو خطِ آخرِ `main` را عوض کن — یعنی اول تیترِ نامعتبر را امتحان کن، بعد عنوانِ ناموجود را. آیا جواب‌ها فرق می‌کنند؟ چرا باید یا نباید فرق کنند؟

---

## خطاهایی که خواهی دید

### `E0004` — واریانتی که اضافه شد، در هر جایی که match رویش هست

`examples/06-a-new-status.rs` دقیقاً همان `Status`ی است که بالا ساختیم، به‌علاوه‌ی یک واریانتِ تازه — `OnHold { since: u32 }` — و همان دو تابعی که رویش `match` می‌زنند: `describe` و `status_tag`. هیچ‌جای دیگری دست نخورده.

```text
error[E0004]: non-exhaustive patterns: `Status::OnHold { .. }` not covered
  --> examples\06-a-new-status.rs:26:24
   |
26 |     let detail = match entry.status {
   |                        ^^^^^^^^^^^^ pattern `Status::OnHold { .. }` not covered
   |
note: `Status` defined here
  --> examples\06-a-new-status.rs:12:6
   |
12 | enum Status {
   |      ^^^^^^
...
17 |     OnHold { since: u32 },
   |     ------ not covered
   = note: the matched value is of type `Status`
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern or an explicit pattern as shown
   |
30 ~         Status::Dropped { at } => format!("dropped at episode {at}"),
31 ~         Status::OnHold { .. } => todo!(),
   |

error[E0004]: non-exhaustive patterns: `&Status::OnHold { .. }` not covered
  --> examples\06-a-new-status.rs:36:11
   |
36 |     match status {
   |           ^^^^^^ pattern `&Status::OnHold { .. }` not covered
   |
note: `Status` defined here
  --> examples\06-a-new-status.rs:12:6
   |
12 | enum Status {
   |      ^^^^^^
...
17 |     OnHold { since: u32 },
   |     ------ not covered
   = note: the matched value is of type `&Status`
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern or an explicit pattern as shown
   |
40 ~         Status::Dropped { .. } => "dropped",
41 ~         &Status::OnHold { .. } => todo!(),
   |

For more information about this error, try `rustc --explain E0004`.
error: could not compile `p1-07-01-guided-mini-project` (example "06-a-new-status") due to 2 previous errors
```

**کامپایلر به چه اعتراض دارد:** دو تابع، دو خطا، یک دلیل. `Status` حالا پنج واریانت دارد و هر دو `match` فقط چهارتا را پوشش می‌دهند. کامپایلر برنامه را اجرا نکرد تا این را بفهمد — فقط تعدادِ واریانت‌ها را با تعدادِ بازوها مقایسه کرد. دقت کن پیام دقیقاً می‌گوید کدام واریانت جا افتاده — `OnHold` — نه فقط «یک‌جایی چیزی جا افتاده».

و یک تفاوتِ ظریف بینِ آن دو خطا: اولی می‌گوید `Status::OnHold` (بدونِ `&`)، چون `entry.status` یک کپیِ مقدار است — `Status` مشتقِ `Copy` است، پس `match` روی یک رونوشت کار می‌کند. دومی می‌گوید `&Status::OnHold`، چون `status` خودِ پارامتر، از نوعِ `&Status` است. همان یادداشتِ «the matched value is of type `&Progress`» که در [۱.۵.۴](../../05-your-own-types/04-match-in-depth/README.fa.md) دیدی، این‌بار روی نوعِ خودت.

**راه‌حل:** به هر دو `match` یک بازوی `Status::OnHold { since } => ...` اضافه کن (یا در دومی، `Status::OnHold { .. } => ...`، چون آن‌جا به `since` نیازی نیست).

**چرا این راه‌حل است:** این دقیقاً همان معامله‌ای‌ست که [۱.۵.۴](../../05-your-own-types/04-match-in-depth/README.fa.md) وعده داده بود. اگر هرکدام از این دو `match` به‌جای آخرین بازو `_ => ...` داشت، هیچ‌کدام از این دو خطا اصلاً نمی‌آمد — کد بی‌سروصدا کامپایل می‌شد، و «نگه‌داشته‌شده» بی‌آن‌که کسی تصمیمی گرفته باشد، همان برچسبِ پیش‌فرض را می‌گرفت. نام‌بردنِ هر واریانت، به‌جای `_`، همین دو خطای پرحرف را به قیمتِ آسودگیِ ذهنی می‌خرد.

---

## تمرین

### گرم‌کردن

<details>
<summary><code>Rating::new(15).value()</code> چه عددی برمی‌گرداند؟</summary>

`۱۰`. `new` مقدار را کلمپ می‌کند، رد نمی‌کند — هیچ چیزی که به آن بدهی نمی‌تواند بیشتر از `MAX_RATING` بسازد.

</details>

<details>
<summary>بعد از <code>list.add(bebop);</code> آیا <code>println!("{bebop:?}")</code> کامپایل می‌شود؟</summary>

نه. `add` مالکیتِ `bebop` را گرفته — همان `E0382`ی که در ۱.۲.۲ دیدی، این‌بار روی نوعِ خودت.

</details>

<details>
<summary>یک <code>Status</code> پنج واریانتی داری و یک <code>match</code> با چهار بازوی نام‌دار و بدونِ <code>_</code>. کامپایل می‌شود؟</summary>

نه. کامپایلر شمرده که یک واریانت پوشش داده نشده و `E0004` می‌دهد — چه واریانتِ پنجم را استفاده کنی چه نه.

</details>

<details>
<summary>چرا <code>&text[..n]</code> روی یک عنوانِ فارسی می‌تواند panic کند، ولی <code>truncate_to_chars(text, n)</code> هرگز نمی‌تواند؟</summary>

اولی یک اندیسِ بایتیِ دلخواه است که ممکن است وسطِ یک نویسه‌ی چندبایتی بیفتد. دومی فقط اندیس‌هایی را برمی‌گرداند که خودِ `char_indices` داده — و آن‌ها همیشه مرزِ نویسه‌اند.

</details>

<details>
<summary><code>rate_from_text("Cowboy Bebop", "nine")</code> وقتی «Cowboy Bebop» اصلاً در فهرست نیست، کدام خطا را می‌دهد؟</summary>

`InvalidRating`، نه `NotFound`. متن قبل از جست‌وجوی عنوان پارس می‌شود؛ پارسِ ناموفق، تابع را زودتر با `?` تمام می‌کند.

</details>

<details>
<summary>درست یا غلط: هر متدِ <code>Watchlist</code> یک <code>Result</code> برمی‌گرداند.</summary>

غلط. فقط `rate` و `rate_from_text`. `add`، `find` و `titles` هیچ‌کدام نمی‌توانند شکست بخورند، پس هیچ‌کدام `Result` ندارند.

</details>

### تعمیر

`examples/06-a-new-status.rs` را با `--features broken` اجرا کن، دو خطای `E0004` را بخوان، بعد هر دو را درست کن:

۱. با اضافه‌کردنِ یک بازوی `Status::OnHold { .. } => ...` نام‌دار به هر دو `match`.
۲. بعد، فقط برای دیدنِ فرق، آخرین بازوی یکی از آن دو را با `_ => ...` عوض کن و یک واریانتِ ششم اضافه کن. کدام `match` این‌بار ساکت می‌ماند؟ همان سؤالی که ۱.۵.۴ هم پرسیده بود.

### پیاده‌سازی

پنج متد در `src/lib.rs`:

```sh
cargo test -p p1-07-01-guided-mini-project
```

تایپ‌ها — `Rating`، `Status`، `Entry`، `WatchlistError` — از قبل کامل نوشته شده‌اند؛ کارِ تو فقط متدهای `Watchlist` است. هر متد یک doc comment دارد که رفتارش را دقیق مشخص می‌کند — نه چیزی که تست‌ها چک می‌کنند ولی توضیح ندهد.

دو نکته که تست‌ها رویشان حساسند: `titles` باید به ترتیبِ افزوده‌شدن باشد، نه هر ترتیبِ دیگری؛ و `rate_from_text` باید متن را قبل از جست‌وجوی عنوان پارس کند — یک متنِ نامعتبر، حتی روی عنوانی که وجود ندارد، باید `InvalidRating` بدهد نه `NotFound`.

### بساز

روی `Watchlist` یک متدِ تازه بنویس: `fn render_all(&self) -> String`، که مرحله‌ی ۳ (توصیفِ هر عنوان با `match`) و مرحله‌ی ۴ (ستون‌های هم‌تراز و کوتاه‌شده) را در یک متدِ واحد روی خودِ نوع ترکیب کند — به‌جای دو تابعِ آزاد که یک `&Watchlist` می‌گیرند.

هیچ تستِ ازپیش‌نوشته‌شده‌ای برایش نیست؛ خودت تصمیم بگیر خروجی دقیقاً چه شکلی باشد، امضایش را بنویس، و روی چند عنوانِ فارسی و انگلیسی امتحانش کن.

### چالش (اختیاری)

یک `fn remove(&mut self, title: &str) -> Option<Entry>` بنویس که عنوان را از فهرست حذف می‌کند و **مالکیتِ** آن را به صدازننده برمی‌گرداند — برخلافِ `find` که فقط نگاهش می‌دهد.

فکر کن: کدام متدِ `Vec` برای این کار مناسب‌تر است، `remove` یا `swap_remove`؟ کدام‌یک ترتیبِ بقیه‌ی عنوان‌ها را حفظ می‌کند، و آیا برایت مهم است؟ و اگر دو عنوان دقیقاً یک اسم داشته باشند — چیزی که `find` هرگز نپرسید — این متد باید کدامشان را بردارد؟

نگاهِ رو به جلو: `titles`ی که این درس نوشتی یک حلقه‌ی دستی است. وقتی به ۲.۲ (iteratorها) رسیدی، برگرد و با `self.entries.iter().map(|e| e.title.as_str()).collect()` بازش‌نویسی کن، و ببین چند خط کم شد.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به کارت می‌آید |
|---|---|---|
| newtype clamp | سازنده‌ای که ورودیِ نامعتبر را رد نمی‌کند، محدودش می‌کند | `Rating::new` |
| تصمیمِ مالکیت در امضا | `T` در برابرِ `&T` در برابرِ `&mut T` در پارامتر یا خروجی | `add`، `find`، `titles` |
| match جامع | کامپایلر ثابت می‌کند هر واریانت پوشش داده شده | `describe`، `status_tag` |
| `E0004` | یک واریانت در یک `match` جا افتاده | هر جا `enum` اضافه می‌شود |
| کوتاه‌کردن به نویسه | برشی که همیشه از `char_indices` می‌آید، هرگز panic نمی‌کند | عنوان‌های فارسی و انگلیسی |
| `enum` خطا + `From` | `?` خودش تبدیلِ نوعِ خطا را انجام می‌دهد | `rate_from_text` |

### الان می‌دانی

- یک `enum` با واریانت‌های داده‌دار، حالتِ نامعتبر را نانوشتنی می‌کند؛ یک `match` جامع، فراموش‌کردنِ یک حالت را غیرقابلِ کامپایل.
- امضای هر متد یک تصمیمِ مالکیت است: پارامتر مالکانه یعنی «نگهش می‌دارم»، پارامتر یا خروجیِ قرضی یعنی «فقط نگاه می‌کنم».
- پدینگِ رشته در Rust بر اساسِ نویسه است، ولی برشِ رشته بر اساسِ بایت — و این دو، دو قاعده‌ی کاملاً جدا هستند.
- کوتاه‌کردنِ امن از اندیس‌هایی می‌آید که خودِ `char_indices` داده، نه از یک عددِ دلخواه.
- `Result` فقط برای عملیاتی نوشته می‌شود که واقعاً می‌تواند شکست بخورد؛ بقیه، بدونِ آن، ساده‌ترند.
- یک `impl From<X> for Y` یعنی `?` می‌تواند خودش یک `X` را به `Y` تبدیل کند، بدونِ هیچ `match` دستی.

### بعداً کامل‌تر می‌بینی

- **`From` و تبدیلِ خطا، به‌طورِ کامل** — [۱.۶.۵](../../06-absence-and-failure/05-from-and-error-conversion/README.fa.md)
- **کلیدهای باز-انتها که این درس با `Vec` دور زد** — [فاز ۲ — `HashMap`](../../../phase2-intermediate/01-collections/01-vec-and-hashmap/README.fa.md)
- **`titles` به‌عنوانِ یک خط، به‌جای یک حلقه** — [فاز ۲ — Iteratorها](../../../phase2-intermediate/02-iterators-and-closures/02-iterator-adapters/README.fa.md)
- **وقتی یک واریانت خیلی بزرگ‌تر از بقیه شود** — [فاز ۲ — `Box`](../../../phase2-intermediate/05-smart-pointers/01-box-and-heap-allocation/README.fa.md)
- **مرورِ کاملِ فاز، به‌جای یک برنامه‌ی تکی** — [۱.۷.۲ — مرورِ فاز](../02-phase-review/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا `add` مالکیتِ `Entry` را می‌گیرد ولی `find` فقط قرضش می‌دهد؟
- اضافه‌کردنِ یک واریانتِ تازه به یک `enum`، چه اتفاقی برای یک `match` جامعِ ازپیش‌نوشته‌شده می‌افتد؟
- چرا کوتاه‌کردنِ یک عنوانِ فارسی به N بایت می‌تواند panic کند، ولی به N نویسه هرگز؟
- چرا از پنج متدِ `Watchlist`، فقط دو تا `Result` برمی‌گردانند؟
- `?` روی یک `ParseIntError`، داخلِ تابعی که `Result<_, WatchlistError>` برمی‌گرداند، چه کاری خودش انجام می‌دهد؟

---

## بیشتر

- [کتابِ Rust — پیاده‌سازیِ یک پروژه‌ی کوچک](https://doc.rust-lang.org/book/ch12-00-an-io-project.html) — همین ایده، در مقیاسِ یک ابزارِ خط‌فرمان.
- [`std::str::CharIndices`](https://doc.rust-lang.org/std/str/struct.CharIndices.html) — مستنداتِ همان تابعی که کوتاه‌کردنِ امن را ممکن کرد.
- [`std::convert::From`](https://doc.rust-lang.org/std/convert/trait.From.html) — مستنداتِ چیزی که `?` پشتِ‌صحنه صدا می‌زند.

# ۱.۵.۴ — `match` از نزدیک

## در یک نگاه

بعد از این درس می‌توانی:

- یک `E0004` را بخوانی و بگویی کامپایلر کدام واریانت را جا افتاده دیده است.
- یک `match` بنویسی که در همان قدمی که واریانت را تشخیص می‌دهد، داده‌اش را هم بیرون بکشد.
- بین یک بازوی نام‌برده، یک نگهبان، و `_` انتخاب کنی — و بگویی `_` چه چیزی از تو می‌گیرد.
- یک واریانت به یک شمارشی اضافه کنی و از کامپایلر فهرستِ کاملِ جاهایی را بگیری که باید عوض شوند.

**زمان:** حدود ۶۰ دقیقه · **پیش‌نیاز:**
[۱.۵.۳ — شمارشی‌ها به‌عنوان داده](../03-enums-as-data/README.fa.md) ·
[۱.۱.۳ — نوع‌های مرکب و واسازی](../../01-foundations/03-compound-types-and-destructuring/README.fa.md)

---

## چرا اهمیت دارد

درسِ قبل نصفِ کار را انجام داد: با یک شمارشی (enum) داده‌ات دیگر نمی‌تواند غلط باشد. یک `Completed` امتیاز دارد، یک `Dropped` ندارد، و ساختنِ چیزی که هم «در حالِ خواندن» باشد و هم امتیاز داشته باشد اصلاً قابلِ نوشتن نیست.

ولی داده‌ی درست کافی نیست. کدی که آن داده را می‌خواند هم می‌تواند حالتی را فراموش کند.

در پایتون این را هزار بار دیده‌ای:

```python
if status == "reading":
    ...
elif status == "finished":
    ...
else:
    pass          # و این «else» همان جایی است که باگ زندگی می‌کند
```

فردا یک وضعیتِ `paused` اضافه می‌شود. این تابع کامپایل می‌شود — چون چیزی برای کامپایل شدن نیست — تست‌ها سبز می‌مانند، و شش هفته بعد کسی گزارش می‌دهد که سریال‌های متوقف‌شده در داشبورد اصلاً نمایش داده نمی‌شوند. برای پیدا کردنِ هر جایی که باید عوض می‌شد، ابزارت `grep` است.

`match` در Rust همان `if/elif` نیست با نحوِ زیباتر. تفاوتش یک کلمه است: **جامع بودن (exhaustiveness)**. کامپایلر ثابت می‌کند که هر حالتِ ممکن یک بازو دارد، و اگر نداشته باشد برنامه ساخته نمی‌شود. یعنی اضافه کردنِ یک واریانت، هر جایی را که باید عوض شود به یک خطای کامپایل تبدیل می‌کند — نه به یک تیکت در آینده.

> پایتونِ ۳.۱۰ به بعد هم `match` دارد، و با `enum.Enum` و `typing.assert_never` می‌شود چیزی شبیهِ همین بررسی ساخت. ولی آن‌جا اختیاری است، در زمانِ type-check اتفاق می‌افتد، و فقط اگر واقعاً `mypy` را اجرا کنی. این‌جا خودِ کامپایلر است و راهِ خاموش کردنش وجود ندارد.

این درس، به‌علاوه‌ی درسِ قبل، همان چیزی است که باعث می‌شود آدم‌ها بگویند «در Rust حالتِ نامعتبر قابلِ بیان نیست». نصفش شمارشی بود؛ نصفِ دیگرش این‌جاست.

---

## مفهوم

### `match` یک عبارت است

اولین چیزی که باید در ذهنت جا بیفتد این است که `match` یک دستور (statement) نیست؛ یک **عبارت (expression)** است. یعنی یک مقدار تولید می‌کند و می‌شود آن مقدار را جایی گذاشت — دقیقاً مثلِ `if` در [۱.۱.۴](../../01-foundations/04-functions-and-expressions/README.fa.md).

```rust
let mood = match 3 {
    0 => "nothing on the shelf",
    1..=5 => "a manageable pile",
    _ => "too many open tabs",
};
```

```text
mood:  a manageable pile
```

آن `;` آخر مالِ `let` است، نه مالِ `match`. و چون یک عبارت است، جای دیگری هم می‌شود گذاشتش — مثلاً وسطِ یک `format!`، بدونِ هیچ متغیرِ موقتی:

```rust
println!(
    "check: {}",
    match band(7).len() {
        0 => "empty",
        1..=4 => "short",
        _ => "long enough",
    }
);
```

```text
check: short
```

یک قاعده‌ی مهم از همین‌جا می‌آید: **همه‌ی بازوها باید مقداری از یک نوع بدهند.** اگر یکی `String` بدهد و یکی `&str`، برنامه ساخته نمی‌شود — همان قاعده‌ی «هر دو شاخه‌ی `if` هم‌نوع‌اند» که در ۱.۱.۵ دیدی، فقط با تعدادِ بیشتری شاخه. خطایش `E0308` است و در بخشِ خطاها می‌بینی‌اش.

### هر بازو یک الگو است

سمتِ چپِ هر `=>` یک **الگو (pattern)** است — همان چیزی که در [۱.۱.۳](../../01-foundations/03-compound-types-and-destructuring/README.fa.md) در سمتِ چپِ `let` نوشتی. آن‌جا گفتیم «همین ماشین پشتِ `match` هم هست، فقط با شکل‌های جالب‌تر». این‌ها آن شکل‌ها:

```rust
fn band(stars: u8) -> String {
    match stars {
        0 => "unrated".to_string(),
        1..=3 => "weak".to_string(),
        4..=6 => "watchable".to_string(),
        7 | 8 => "good".to_string(),
        9 | 10 => "top shelf".to_string(),
        _ => "not a score".to_string(),
    }
}
```

```text
  0  -> unrated
  2  -> weak
  5  -> watchable
  8  -> good
 10  -> top shelf
200  -> not a score
```

چهار شکل در شش خط:

| الگو | اسمش | یعنی |
|---|---|---|
| `0` | لیترال | دقیقاً این مقدار |
| `1..=3` | الگوی بازه‌ای (range pattern) | از ۱ تا ۳، هر دو سر شامل |
| `7 \| 8` | جایگزین (alternative) | این یا آن، در یک بازو |
| `_` | جانشین (wildcard) | هر چیزِ دیگری، و اسمی هم نمی‌خواهم |

`..=` یعنی بازه‌ی بسته؛ همان `..=` که در [۱.۱.۵](../../01-foundations/05-control-flow/README.fa.md) در حلقه‌ها دیدی. و آن `_` این‌جا دقیقاً یعنی «۱۱ تا ۲۵۵»، چون بقیه‌ی مقدارهای یک `u8` همان‌هاست.

### کامپایلر حالت‌ها را برایت می‌شمارد

حالا همان `match` روی یک شمارشیِ چهارواریانتی — با یک بازوی جاافتاده:

```rust
match progress {
    Progress::NotStarted => "not started".to_string(),
    Progress::Reading { chapter } => format!("chapter {chapter}"),
    Progress::Finished { rating } => format!("finished, {rating}/10"),
}
```

```text
error[E0004]: non-exhaustive patterns: `&Progress::Dropped { .. }` not covered
  --> examples\05-a-missing-arm.rs:15:11
   |
15 |     match progress {
   |           ^^^^^^^^ pattern `&Progress::Dropped { .. }` not covered
   |
note: `Progress` defined here
  --> examples\05-a-missing-arm.rs:7:6
   |
 7 | enum Progress {
   |      ^^^^^^^^
...
11 |     Dropped { at: u32 },
   |     ------- not covered
   = note: the matched value is of type `&Progress`
```

این مهم‌ترین خطای این درس است، پس خط به خط بخوانش:

- `non-exhaustive patterns` — «الگوهایت همه را نمی‌پوشانند». این نامِ رسمیِ همان چیزی است که دنبالش بودیم.
- `` `&Progress::Dropped { .. }` not covered `` — و **دقیقاً می‌گوید کدام**. لازم نیست خودت شمارشی را با `match` مقایسه کنی؛ کامپایلر این کار را کرده.
- ``note: `Progress` defined here`` با فلشی به خطِ ۱۱ — تعریفِ واریانتِ گمشده را هم نشانت می‌دهد. روی یک شمارشیِ دوازده‌واریانتی این تفاوتِ بینِ ده ثانیه و ده دقیقه است.
- ``the matched value is of type `&Progress` `` — یادآوری می‌کند که داری روی یک ارجاع تطبیق می‌دهی، نه روی خودِ مقدار. کمی جلوتر به این برمی‌گردیم.

نکته‌ی اصلی: کامپایلر برنامه را **اجرا** نکرد تا این را بفهمد. تعدادِ واریانت‌ها را با پوششِ بازوها مقایسه کرد و یک ثابت‌کردنِ ساده انجام داد. این کار در پایتون ممکن نیست، چون آن‌جا `status` می‌تواند هر رشته‌ای باشد و «همه‌ی حالت‌ها» عددِ مشخصی ندارد.

### این جامع بودن به چه دردی می‌خورد

فرض کن سه ماه بعد محصول می‌گوید کاربر باید بتواند یک سریال را «موقتاً متوقف» کند. یک واریانت اضافه می‌کنی — و هیچ کارِ دیگری نمی‌کنی:

```rust
enum Progress {
    NotStarted,
    Reading { chapter: u32 },
    Finished { rating: u8 },
    Dropped { at: u32 },
    Paused { at: u32 },
}
```

```text
error[E0004]: non-exhaustive patterns: `&Progress::Paused { .. }` not covered
  --> examples\06-a-new-variant.rs:19:11
error[E0004]: non-exhaustive patterns: `&Progress::Paused { .. }` not covered
  --> examples\06-a-new-variant.rs:28:11
error[E0004]: non-exhaustive patterns: `&Progress::Paused { .. }` not covered
  --> examples\06-a-new-variant.rs:37:11
error: could not compile `p1-05-04-match-in-depth` (example "06-a-new-variant") due to 3 previous errors
```

این خطِ اولِ هر خطا به‌علاوه‌ی حرفِ آخرِ کامپایلر است؛ هر سه‌تا همان جزئیاتی را دارند که بالاتر خواندی، و اجرای مثال کاملشان را نشان می‌دهد.

سه خطا، سه خطِ متفاوت، سه تابعی که باید درباره‌شان تصمیم بگیری. **این فهرستِ کارها را کامپایلر نوشت، نه تو.** و تا هر سه انجام نشود، برنامه ساخته نمی‌شود.

همین یک قابلیت است که سرعتِ تغییر دادنِ یک سیستمِ بزرگ را عوض می‌کند. در پایتون این تغییر یعنی `grep` زدن و امیدوار بودن؛ این‌جا یعنی سه بار `cargo build` و تمام.

### `_` قولی است که شاید نخواهی بدهی

در فایلِ `06-a-new-variant` یک تابعِ چهارم هم هست و اسمش در فهرستِ خطاها نیست — چون آخرین بازویش `_` است. همین شکل، این‌بار از `04-tuples-and-options` که کامپایل و اجرا می‌شود:

```rust
fn tag(progress: &Progress) -> String {
    match progress {
        Progress::Finished { rating } => format!("finished, {rating}/10"),
        _ => "not finished".to_string(),
    }
}
```

```text
not finished
not finished
finished, 9/10
```

`_` به کامپایلر گفت «هر چیزِ دیگری را من قبول دارم» — پس واریانتِ تازه بی‌سروصدا شد `"not finished"` و از تو پرسیده نشد که آیا این درست است یا نه. برای یک `u8` این عالی است؛ برای یک شمارشی معمولاً یعنی خطای فردا را همین الان خریده‌ای.

قاعده‌ی عملی: **روی یک شمارشیِ خودت، تا می‌توانی واریانت‌ها را با اسم بنویس.** اگر واقعاً چند واریانت رفتارِ یکسان دارند، `A | B | C` بنویس نه `_` — این‌طوری اضافه شدنِ `D` باز هم خطا می‌دهد.

و اگر می‌خواهی خودِ ابزار مراقبت باشد، clippy یک لینتِ اختیاری دارد:

```text
warning: wildcard match will also match any future added variants
  --> examples\04-tuples-and-options.rs:50:9
   |
50 |         _ => "not finished".to_string(),
   |         ^ help: try: `Progress::NotStarted | Progress::Reading { .. }`
```

### بیرون کشیدنِ داده‌ی واریانت

تا این‌جا فقط تشخیص دادیم کدام واریانت است. کارِ دومِ الگو این است که داده‌ی داخلش را هم بیرون بکشد — در همان قدم:

```rust
fn describe(progress: &Progress) -> String {
    match progress {
        Progress::NotStarted => "not started".to_string(),
        Progress::Reading { chapter, of } => format!("chapter {chapter} of {of}"),
        Progress::Finished { rating } => format!("finished, {rating}/10"),
        Progress::Dropped { at, .. } => format!("dropped at chapter {at}"),
    }
}
```

```text
not started
chapter 12 of 40
finished, 9/10
finished, 6/10
dropped at chapter 3
```

`chapter` و `of` اسم‌های تازه‌اند که از فیلدهای همان واریانت مقدار گرفته‌اند. هیچ تستِ جداگانه‌ای برای «آیا `Reading` است؟» ننوشتی و بعدش هم هیچ `.chapter` نخواندی. **یک قدم، دو کار.**

و آن `..` در بازوی آخر یعنی «بقیه‌ی فیلدهای این واریانت، هر چه هستند». `Dropped` این‌جا یک فیلدِ `reason` هم دارد که لازمش نداشتیم. `..` می‌تواند هر طرفی بایستد:

```rust
fn why_dropped(progress: &Progress) -> String {
    match progress {
        Progress::Dropped { reason, .. } => format!("gave up because the {reason}"),
        _ => "not dropped".to_string(),
    }
}
```

```text
gave up because the art changed
not dropped
```

### الگوها تودرتو می‌شوند

یک الگو دقیقاً به همان عمقی می‌رود که داده می‌رود. این یک ساختار است که داخلش یک شمارشی است که داخلش یک لیترال:

```rust
match entry {
    Entry {
        title,
        progress: Progress::Finished { rating: 10 },
    } => format!("{title}: a perfect score"),
    Entry {
        title,
        progress: Progress::NotStarted,
    } => format!("{title}: untouched"),
    Entry { title, .. } => format!("{title}: somewhere in the middle"),
}
```

```text
Vinland Saga: a perfect score
Berserk: untouched
Vagabond: somewhere in the middle
```

بازوی اول می‌گوید «یک `Entry` که `progress`اش `Finished` است و `rating`اش دقیقاً ۱۰ است — و ضمناً `title`اش را به من بده». همه‌ی این‌ها یک الگوست. اگر بخواهی همین را با `if` بنویسی، سه شرطِ تودرتو و یک دسترسی به فیلد می‌شود.

### `@` — هم بسنج، هم نگه دار

گاهی می‌خواهی یک مقدار در یک بازه باشد **و** خودِ مقدار را هم داشته باشی. اگر فقط `9..=10` بنویسی، شرط را داری ولی عدد را نه. `@` هر دو را می‌دهد:

```rust
fn shelf(progress: &Progress) -> String {
    match progress {
        Progress::Finished { rating: r @ 9..=10 } => format!("hall of fame ({r}/10)"),
        Progress::Finished { rating } => format!("read once ({rating}/10)"),
        other => format!("still open — {}", describe(other)),
    }
}
```

```text
still open — not started
still open — chapter 12 of 40
hall of fame (9/10)
read once (6/10)
still open — dropped at chapter 3
```

`r @ 9..=10` را این‌طور بخوان: «اگر در بازه‌ی ۹ تا ۱۰ بود، اسمش را بگذار `r`». و بازوی آخر نشان می‌دهد که یک اسمِ خالی هم خودش یک الگوست: `other` هر چیزی را می‌گیرد و اسم هم بهش می‌دهد — دقیقاً مثلِ `_`، فقط با یک اسم.

### بازوها به ترتیب امتحان می‌شوند

`match` بازوها را از بالا به پایین امتحان می‌کند و **اولین بازویی که تطبیق بخورد برنده است**. بقیه حتی نگاه هم نمی‌شوند.

```senpai-visual
{"kind":"concept","labels":["مقدار","بازوی ۱","بازوی ۲","اولین تطبیق برنده"]}
```

یعنی ترتیب، بخشی از معنای برنامه است. این دو تابع بازوهای یکسانی دارند و فقط ترتیبشان فرق می‌کند:

```rust
match (chapter, unread) {
    (0, n) => format!("never opened, {n} waiting"),
    (c, n) if n > 20 => format!("chapter {c}, {n} behind — good luck"),
    (c, n) => format!("chapter {c}, {n} to go"),
    (_, 0) => "all caught up".to_string(),
}
```

```text
all caught up
never opened, 7 waiting
chapter 12, 31 behind — good luck
chapter 12, 3 to go

never opened, 0 waiting
never opened, 7 waiting
chapter 12, 31 behind — good luck
chapter 12, 3 to go
```

بالا نسخه‌ی اصلی است و پایین نسخه‌ی جابه‌جاشده، با همان ورودی‌ها. خطِ اول عوض شده: `(_, 0)` حالا زیرِ یک بازوی همه‌گیر افتاده و هرگز نوبتش نمی‌شود.

کامپایلر این را می‌بیند و هشدار می‌دهد — `warning: unreachable pattern`، که در بخشِ خطاها کاملش هست. هشدار است نه خطا، ولی تقریباً همیشه یعنی برنامه‌ات کاری را که فکر می‌کنی نمی‌کند. قاعده‌ی عملی: **خاص‌ترین بازو بالا، عام‌ترین پایین.**

### نگهبان‌ها، برای چیزی که الگو نمی‌تواند بگوید

الگو دربار‌ه‌ی *شکل* حرف می‌زند. نمی‌تواند بگوید «این دو عدد با هم برابرند». برای آن یک **نگهبان (guard)** لازم است: یک `if` که بعد از تطبیقِ الگو سنجیده می‌شود.

```rust
fn is_at_the_end(chapter: u32, of: u32) -> String {
    match (chapter, of) {
        (c, total) if c == total => "waiting for the next chapter".to_string(),
        (c, total) => format!("{} chapters left", total - c),
    }
}
```

```text
waiting for the next chapter
28 chapters left
```

نگهبان بازوها را انتخاب‌پذیرتر می‌کند، ولی یک هزینه دارد که باید بدانی: **کامپایلر نگهبان‌ها را در شمارشِ جامع بودن حساب نمی‌کند.** دو بازوی `s if s >= 8` و `s if s < 8` روی هم هر `u8` ممکن را می‌پوشانند و کامپایلر باز هم قبول نمی‌کند — چون برای اثباتش باید مقایسه‌ی عددی را حل می‌کرد، و این کاری نیست که بکند. `examples/07-guards-do-not-count.rs` همین است و خودِ خطا هم صریح می‌گویدش؛ در بخشِ خطاها.

### دو مقدار با هم

یک `match` می‌تواند روی یک تاپل کار کند، و آن‌وقت دو (یا سه) مقدار را یک‌جا می‌سنجی. این جایی است که `match` واقعاً از `if/else` جلو می‌زند:

```rust
match (mine, theirs) {
    (Finished { rating: a }, Finished { rating: b }) if a == b => {
        "we gave it the same score".to_string()
    }
    (Finished { rating: a }, Finished { rating: b }) => {
        format!("{a}/10 against {b}/10")
    }
    (NotStarted, NotStarted) => "neither of us has started".to_string(),
    (Finished { .. }, _) | (_, Finished { .. }) => "one of us has finished".to_string(),
    (Reading { chapter: a }, Reading { chapter: b }) => {
        format!("chapter {a} against chapter {b}")
    }
    _ => "still going".to_string(),
}
```

```text
we gave it the same score
9/10 against 4/10
neither of us has started
one of us has finished
chapter 12 against chapter 40
still going
```

بازوی چهارم را ببین: یک `|` که دو الگوی تودرتو را کنارِ هم گذاشته، یعنی «هر کدام‌شان `Finished` باشد». نسخه‌ی `if`ی این تابع نُه مقایسه و یک `else` می‌شد که کسی به آن فکر نکرده.

### تطبیق از پشتِ یک ارجاع

تا الان همه‌ی تابع‌ها `&Progress` گرفته‌اند، نه `Progress` — چون نمی‌خواهیم مالکیت را بگیریم ([۱.۳.۱](../../03-borrowing-and-references/01-shared-and-mutable-refs/README.fa.md)). وقتی روی یک ارجاع تطبیق می‌دهی، اسم‌هایی که الگو می‌سازد هم ارجاع‌اند:

```rust
fn chapters_read(progress: &Progress) -> u32 {
    match progress {
        Progress::NotStarted => 0,
        Progress::Reading { chapter } => *chapter,
        Progress::Finished { .. } => 0,
    }
}
```

```text
0
12
0
```

`chapter` این‌جا یک `&u32` است، برای همین برای برگرداندنِ خودِ عدد `*chapter` نوشته شده. Rust این کار را برایت ساده کرده — لازم نیست الگو را `&Progress::Reading { .. }` بنویسی — ولی نتیجه‌اش این است که گاهی یک `*` لازم می‌شود. اگر یادت رفت، خطای `E0308` می‌گوید `expected u32, found &u32` و همان `*` را پیشنهاد می‌دهد. قاعده‌ی کاملش (که «حالتِ اتصالِ پیش‌فرض» نام دارد) در [فاز ۲ — تطبیق الگو از نزدیک](../../../phase2-intermediate/08-rust-toolbox/01-pattern-matching-depth/README.fa.md) است.

### `Option` هم یک شمارشیِ معمولی است

درسِ قبل نشان داد که `Option` جادو نیست:

```rust
match latest {
    None => "nothing published yet".to_string(),
    Some(n) if n == read => "caught up".to_string(),
    Some(n) if n > read => format!("{} behind", n - read),
    Some(_) => "ahead of the release".to_string(),
}
```

```text
nothing published yet
caught up
3 behind
ahead of the release
```

دو واریانت، پس دو شکلِ بازو — و کامپایلر همان‌طور می‌شماردشان که واریانت‌های `Progress` را می‌شمرد. اگر بازوی `None` را برداری، همان `E0004` را می‌گیری. **این‌که «باید `None` را هم رسیدگی کنی» یک قاعده‌ی مخصوصِ `Option` نیست؛ همان جامع بودنِ همیشگی است.**

بازوی آخر `Some(_)` است نه `Some(n)`، چون در آن حالت عدد به کارمان نمی‌آید. کارهای کوتاه‌ترِ همین کار (`unwrap_or`، `map` و بقیه) در [۱.۶.۲](../../06-absence-and-failure/02-option-combinators/README.fa.md) می‌آید؛ فعلاً همین `match` را ببین، چون همه‌ی آن‌ها روی همین ساخته شده‌اند.

### جمعِ این دو: حالتِ نامعتبر قابلِ بیان نیست

حالا هر دو نیمه را داری:

| | چه چیزی را غیرممکن می‌کند |
|---|---|
| شمارشی | ساختنِ داده‌ای که ترکیبِ نامعتبر باشد |
| `match` جامع | نوشتنِ کدی که یک حالت را فراموش کند |

نیمه‌ی اول تنها کافی نیست: می‌شود یک شمارشیِ بی‌نقص داشت و بعد در بیست جا `_ => ()` نوشت. نیمه‌ی دوم تنها هم کافی نیست: یک `match` جامع روی یک `struct` با فیلدهای متناقض چیزی را نجات نمی‌دهد.

با هم، آن جمله‌ای می‌شوند که زیاد می‌شنوی: **حالتِ نامعتبر را نمی‌شود بیان کرد.** نه چون کسی مواظب بوده، بلکه چون کامپایلر اجازه نمی‌دهد.

و یک نکته‌ی آخر: خیلی وقت‌ها فقط *یک* بازو برایت جالب است و بقیه هیچ کاری ندارند. برای همان حالت یک نحوِ کوتاه‌تر وجود دارد و درسِ بعدی است — [۱.۵.۵ — `if let`، `while let`، `let else`](../05-if-let-while-let-let-else/README.fa.md). آن‌ها چیزِ تازه‌ای یاد نمی‌دهند؛ همین `match` هستند وقتی فقط یک بازو مهم است.

---

## دست‌به‌کد

```sh
cargo run -p p1-05-04-match-in-depth --example 01-match-is-an-expression
cargo run -p p1-05-04-match-in-depth --example 02-binding-out-of-variants
cargo run -p p1-05-04-match-in-depth --example 03-guards-and-order
cargo run -p p1-05-04-match-in-depth --example 04-tuples-and-options
```

`03` عمداً با یک هشدار ساخته می‌شود. هشدار را بخوان، بعد خروجی را ببین.

بعد چهار تای خراب:

```sh
cargo run -p p1-05-04-match-in-depth --example 05-a-missing-arm --features broken
cargo run -p p1-05-04-match-in-depth --example 06-a-new-variant --features broken
cargo run -p p1-05-04-match-in-depth --example 07-guards-do-not-count --features broken
cargo run -p p1-05-04-match-in-depth --example 08-arms-disagree --features broken
```

بعد این‌ها را امتحان کن:

۱. در `05-a-missing-arm`، به‌جای بازوی جاافتاده یک `_ => "unknown".to_string()` بگذار. کامپایل می‌شود؟ حالا یک واریانتِ پنجم به شمارشی اضافه کن. باز هم کامپایل می‌شود؟
۲. در `02-binding-out-of-variants`، در `describe` بازوی `Dropped` را به `Progress::Dropped { at, reason }` عوض کن و `reason` را چاپ نکن. کامپایلر چه می‌گوید؟
۳. در `03-guards-and-order`، بازوی `(_, 0)` را در `queue_note_reordered` به بالا ببر. هشدار می‌رود و خروجی چه می‌شود؟
۴. در `04-tuples-and-options`، این را اجرا کن و هشدارِ اختیاری را ببین:
   `cargo clippy -p p1-05-04-match-in-depth --example 04-tuples-and-options -- -W clippy::wildcard_enum_match_arm`

---

## خطاهایی که خواهی دید

### `E0004` — واریانتی که رسیدگی نکردی

```text
error[E0004]: non-exhaustive patterns: `&Progress::Paused { .. }` not covered
  --> examples\06-a-new-variant.rs:19:11
   |
19 |     match progress {
   |           ^^^^^^^^ pattern `&Progress::Paused { .. }` not covered
   |
note: `Progress` defined here
  --> examples\06-a-new-variant.rs:10:6
   |
10 | enum Progress {
   |      ^^^^^^^^
...
15 |     Paused { at: u32 },
   |     ------ not covered
   = note: the matched value is of type `&Progress`
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern or an explicit pattern as shown
   |
23 ~         Progress::Dropped { at } => format!("dropped at chapter {at}"),
24 ~         &Progress::Paused { .. } => todo!(),
   |
```

**کامپایلر به چه اعتراض دارد:** این `match` باید برای هر مقدارِ ممکنِ `Progress` جواب داشته باشد، و `Paused` جوابی ندارد. چون `match` یک عبارت است، «جواب نداشتن» یعنی برنامه در آن حالت هیچ مقداری تولید نمی‌کند — و این قابلِ کامپایل شدن نیست.

**راه‌حل:** بازو را اضافه کن و تصمیم بگیر رفتارِ درست چیست:

```rust
Progress::Paused { at } => format!("paused at chapter {at}"),
```

**چرا این راه‌حل است:** پیشنهادِ خودِ کامپایلر `todo!()` است، و این پیشنهادِ خوبی است: می‌گذارد برنامه ساخته شود و اگر واقعاً به آن حالت برسی، پنیک می‌کند. ولی `todo!()` را به‌عنوانِ جواب رها نکن — سؤالی که خطا پرسید («این حالت یعنی چه؟») هنوز بی‌جواب است.

و وسوسه‌ی `_ => ...` را بشناس. کامپایل می‌شود، خطا می‌رود، و همان‌جا مکانیزمی را که این خطا برایت روشن کرد خاموش می‌کنی. اگر سه تابع خطا داده‌اند، سه تصمیم لازم است، نه یک `_`.

اگر یک واریانت را از قبل هم جا انداخته باشی، همین خطا با یک بازوی جاافتاده هم می‌آید — `examples/05-a-missing-arm.rs` کوچک‌ترین شکلِ همین است.

### `E0004` — وقتی نگهبان‌ها فریبت می‌دهند

```text
error[E0004]: non-exhaustive patterns: `0_u8..=u8::MAX` not covered
  --> examples\07-guards-do-not-count.rs:9:11
   |
 9 |     match stars {
   |           ^^^^^ pattern `0_u8..=u8::MAX` not covered
   |
   = note: the matched value is of type `u8`
   = note: match arms with guards don't count towards exhaustivity
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern or an explicit pattern as shown
   |
11 ~         s if s < 8 => format!("the rest ({s}/10)"),
12 ~         0_u8..=u8::MAX => todo!(),
   |
```

**کامپایلر به چه اعتراض دارد:** خطِ `match arms with guards don't count towards exhaustivity` تمامِ داستان است. دو بازوی نگهبان‌دار روی هم همه‌ی `u8`ها را می‌گیرند — ولی کامپایلر برای فهمیدنش باید ریاضیِ `s >= 8` و `s < 8` را حل می‌کرد. این کار را نمی‌کند؛ به‌جایش هر بازوی نگهبان‌دار را «شاید تطبیق بخورد، شاید نه» حساب می‌کند.

**راه‌حل:** آخرین بازو را بدونِ نگهبان بنویس:

```rust
s => format!("the rest ({s}/10)"),
```

**چرا این راه‌حل است:** حالا یک بازوی بدونِ شرط هست که قطعاً تطبیق می‌خورد، پس پوشش کامل است. این خودش یک قاعده‌ی سبکِ خوب است: زنجیره‌ی نگهبان‌ها را با یک بازوی بی‌شرط تمام کن — همان کاری که با `else` در آخرِ یک زنجیره‌ی `if` می‌کنی.

### `E0308` — بازوها هم‌نوع نیستند

```text
error[E0308]: `match` arms have incompatible types
  --> examples\08-arms-disagree.rs:11:18
   |
 9 | /     match stars {
10 | |         0..=4 => "weak".to_string(),
   | |                  ------------------ this is found to be of type `String`
11 | |         5..=7 => "watchable",
   | |                  ^^^^^^^^^^^ expected `String`, found `&str`
12 | |         _ => "good".to_string(),
13 | |     }
   | |_____- `match` arms have incompatible types
   |
help: try using a conversion method
   |
11 |         5..=7 => "watchable".to_string(),
   |                             ++++++++++++
```

**کامپایلر به چه اعتراض دارد:** `match` یک عبارت است، و یک عبارت یک نوع دارد. اولین بازو نوع را روی `String` تثبیت کرده (`this is found to be of type String`) و بازوی بعدی یک `&str` داده.

**راه‌حل:** همان که خودش می‌گوید:

```rust
5..=7 => "watchable".to_string(),
```

**چرا این راه‌حل است:** `&str` و `String` دو نوعِ متفاوت‌اند ([۱.۴.۱](../../04-text-and-strings/01-string-vs-str/README.fa.md))، و Rust بینشان تبدیلِ خودکار انجام نمی‌دهد. راهِ دیگر این است که همه‌ی بازوها `&str` بدهند و تبدیل را یک‌بار در بیرون انجام دهی — که اگر بشود، ارزان‌تر هم هست، چون هیچ تخصیصی نمی‌گیرد.

### `warning: unreachable pattern` — بازویی که نوبتش نمی‌رسد

```text
warning: unreachable pattern
  --> examples\03-guards-and-order.rs:26:9
   |
25 |         (c, n) => format!("chapter {c}, {n} to go"),
   |         ------ matches any value
26 |         (_, 0) => "all caught up".to_string(),
   |         ^^^^^^ no value can reach this
   |
   = note: `#[warn(unreachable_patterns)]` (part of `#[warn(unused)]`) on by default
```

**کامپایلر به چه اعتراض دارد:** بازوی خطِ ۲۵ هر مقداری را می‌گیرد، پس بازوی خطِ ۲۶ هرگز امتحان نمی‌شود. هر دو نیمه‌ی توضیح را نوشته: `matches any value` روی مقصر، `no value can reach this` روی قربانی.

**راه‌حل:** بازوی خاص را ببر بالای بازوی عام. یا اگر واقعاً زائد است، پاکش کن.

**چرا این راه‌حل است:** این هشدار است نه خطا — برنامه ساخته و اجرا می‌شود — ولی تقریباً همیشه یعنی رفتارِ برنامه آن چیزی نیست که فکر می‌کنی. در همان مثال، «all caught up» هرگز چاپ نمی‌شود و به‌جایش «never opened, 0 waiting» درمی‌آید. هشدارهای کامپایلر را مثلِ خطا بخوان؛ فقط عجله‌ی کمتری دارند.

---

## تمرین

### گرم‌کردن

<details>
<summary>این چه چاپ می‌کند؟ <code>match 5 { 1..=5 => "a", 5..=10 => "b", _ => "c" }</code></summary>

`a`. هر دو بازوی اول و دوم به ۵ می‌خورند، و بازوها از بالا به پایین امتحان می‌شوند: اولین تطبیق برنده است. کامپایلش کنی یک هشدار می‌گیری — ولی هشدارِ دیگری: `overlapping_range_endpoints`، که می‌گوید دو بازه در ۵ با هم اشتراک دارند. هشدارِ `unreachable pattern` نیست، چون بازوی دوم برای ۶ تا ۱۰ هنوز قابلِ رسیدن است.

</details>

<details>
<summary>یک شمارشیِ چهارواریانتی داری و <code>match</code>ات سه بازو دارد. چه اتفاقی می‌افتد؟</summary>

`E0004`، با نامِ همان واریانتی که نپوشانده‌ای و فلشی به تعریفش. برنامه ساخته نمی‌شود.

</details>

<details>
<summary>چرا <code>s if s >= 8</code> و <code>s if s < 8</code> با هم جامع نیستند؟</summary>

چون کامپایلر بازوهای نگهبان‌دار را در شمارش حساب نمی‌کند. برای اثباتِ پوشش باید مقایسه‌ی عددی را حل می‌کرد و این کار را نمی‌کند. خودِ خطا این را می‌گوید: `match arms with guards don't count towards exhaustivity`.

</details>

<details>
<summary>تفاوتِ <code>_ =></code> و <code>other =></code> چیست؟</summary>

هیچ، از نظرِ اینکه چه چیزی را می‌گیرند: هر دو هر چیزِ باقی‌مانده را می‌گیرند. تفاوت این است که `other` مقدار را هم نگه می‌دارد تا بتوانی استفاده‌اش کنی، و `_` نه — که یعنی `_` مالکیت هم نمی‌گیرد.

</details>

<details>
<summary>در <code>n @ 1..=9</code>، <code>n</code> چه چیزی دارد؟</summary>

خودِ مقدار، به شرطِ اینکه بینِ ۱ و ۹ باشد. `@` شرط را می‌سنجد و مقدارِ سنجیده‌شده را هم به آن اسم می‌دهد.

</details>

<details>
<summary>یک واریانت به شمارشی اضافه می‌کنی. کدام‌یک از این دو تابع خطا می‌دهد؟</summary>

```rust
fn a(p: &Progress) -> bool {
    match p {
        Progress::Finished { .. } => true,
        Progress::NotStarted | Progress::Reading { .. } | Progress::Dropped { .. } => false,
    }
}

fn b(p: &Progress) -> bool {
    match p {
        Progress::Finished { .. } => true,
        _ => false,
    }
}
```

فقط `a`. `b` بی‌سروصدا `false` برمی‌گرداند و از تو نمی‌پرسد که آیا این درست است — و دقیقاً به همین دلیل `a` نوشتنِ بهتری است، هرچند طولانی‌تر.

</details>

### تعمیر

هر چهار مثالِ خراب را درست کن، و برای هرکدام بگو چه چیزی را عوض کردی و چرا:

۱. `examples/05-a-missing-arm.rs` — یک بار با اضافه کردنِ بازوی گمشده، یک بار با `_`. بعد بگو کدام‌شان را در یک برنامه‌ی واقعی می‌نویسی.
۲. `examples/06-a-new-variant.rs` — هر سه خطا. برای هر تابع تصمیم بگیر `Paused` باید چه رفتاری داشته باشد؛ سه جوابِ یکسان نیست. بعد نگاه کن تابعِ چهارم (`is_finished`) حالا چه برمی‌گرداند و بگو درست است یا نه.
۳. `examples/07-guards-do-not-count.rs` — بدونِ اینکه هیچ نگهبانی را پاک کنی.
۴. `examples/08-arms-disagree.rs` — دو جور: یک بار با یکسان کردنِ نوعِ همه‌ی بازوها روی `String`، یک بار با `&str` نگه داشتنِ همه‌شان.

بعد `examples/03-guards-and-order.rs` را طوری عوض کن که هشدارِ `unreachable pattern` برود و خروجیِ هر دو تابع یکی شود.

### پیاده‌سازی

شش تابع در `src/lib.rs`:

```sh
cargo test -p p1-05-04-match-in-depth
```

هرکدام یک تکه‌ی متفاوت از زبانِ الگو می‌خواهد: بازوی ساده، بازه و جایگزین، نگهبان، `@`، یک تاپل از دو مقدار، و یک `Option`. هیچ‌کدام به `if` احتیاج ندارد — اگر جایی `if` نوشتی که نگهبان نیست، احتمالاً راهِ سخت‌تر را رفته‌ای.

مشخصاتِ دقیق در کامنت‌های خودِ فایل است. جدولِ بالای هر تابع تمامِ چیزی است که برای پاس کردنِ تست‌ها لازم داری؛ لازم نیست فایلِ تست را باز کنی.

### بساز

به شمارشیِ `Progress` در `src/lib.rs` یک واریانتِ `Paused { at: u32 }` اضافه کن و هیچ کارِ دیگری نکن. بعد:

۱. `cargo test -p p1-05-04-match-in-depth` بگیر و خطاها را **بشمار**.
۲. برای هر خطا تصمیم بگیر رفتارِ درستِ `Paused` چیست و بنویسش. `describe` باید چه بگوید؟ `shelf` این را روی کدام قفسه می‌گذارد؟
۳. کدام تابع‌ها خطا **ندادند**؟ برای هرکدام بگو چرا، و بگو آیا رفتارِ فعلی‌شان روی `Paused` درست است.
۴. یک تست برای رفتارِ تازه بنویس.

این تمرین همان کاری است که در یک سیستمِ واقعی هفته‌ای یک‌بار می‌کنی. مدت زمانی که برد را یادداشت کن.

### چالش (اختیاری)

**بخشِ یک.** الگوها روی برش‌ها (slice) هم کار می‌کنند — چیزی که هنوز ندیده‌ای:

```rust
fn run_of(chapters: &[u32]) -> String {
    match chapters {
        [] => "nothing yet".to_string(),
        [only] => format!("just chapter {only}"),
        [first, .., last] => format!("chapters {first} to {last}"),
    }
}
```

اجرایش کن. بعد بازوی `[only]` را بردار و ببین کامپایلر چه می‌گوید. بعد `[first, second, ..]` را هم امتحان کن.

**بخشِ دو.** وقتی فقط می‌خواهی بدانی «آیا این واریانت است یا نه» و مقدارش را نمی‌خواهی، یک ماکرو هست:

```rust
fn is_finished(progress: &Progress) -> bool {
    matches!(progress, Progress::Finished { .. })
}
```

این را جایگزینِ نسخه‌ی `match`ی کن و `cargo clippy` بگیر. clippy درباره‌ی نسخه‌ی قدیمی چه می‌گفت؟ و — سؤالِ مهم‌تر — این نسخه با اضافه شدنِ یک واریانتِ تازه خطا می‌دهد یا نه؟

**بخشِ سه.** این را روی کلِ درس اجرا کن:

```sh
cargo clippy -p p1-05-04-match-in-depth --all-targets -- -W clippy::wildcard_enum_match_arm
```

چند جا هشدار می‌گیری؟ برای هرکدام تصمیم بگیر که `_` آن‌جا انتخابِ درستی بوده یا نه. (این لینت پیش‌فرض خاموش است. بعد از این تمرین بگو چرا فکر می‌کنی خاموش است.)

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به کارت می‌آید |
|---|---|---|
| الگو (pattern) | شکلی که کامپایلر مقدار را با آن می‌سنجد | `let`، `match`، آرگومانِ تابع |
| بازو (arm) | یک `الگو => عبارت` در `match` | هر `match` |
| جامع بودن (exhaustiveness) | اثباتِ اینکه هر حالت یک بازو دارد | چیزی که `E0004` را می‌سازد |
| `E0004` | یک حالت بی‌بازو مانده | نامِ واریانتِ گمشده را می‌دهد |
| الگوی بازه‌ای | `1..=9` | عددها و کاراکترها |
| جایگزین | `7 \| 8` در یک بازو | چند حالت با یک رفتار |
| نگهبان (guard) | `if` روی یک بازوی تطبیق‌خورده | شرط‌هایی که الگو نمی‌تواند بگوید |
| `@` | هم بسنج هم نگه دار: `n @ 1..=9` | وقتی هم شرط می‌خواهی هم مقدار |
| `_` | جانشین، بدونِ اسم | عددها بله، شمارشی‌های خودت با احتیاط |
| `..` | «بقیه‌ی فیلدها» | واریانت‌های پرفیلد |
| بازوی دست‌نیافتنی | بازویی که هرگز نوبتش نمی‌شود | هشدار، و تقریباً همیشه یک باگ |

### الان می‌دانی

- `match` یک عبارت است: مقدار می‌دهد، و همه‌ی بازوها باید هم‌نوع باشند.
- سمتِ چپِ هر `=>` همان الگوهای `let` است، با شکل‌های جالب‌تر.
- کامپایلر ثابت می‌کند هر حالت یک بازو دارد، و اگر نه `E0004` می‌دهد با نامِ واریانتِ گمشده.
- اضافه کردنِ یک واریانت، هر جایی را که باید عوض شود به خطای کامپایل تبدیل می‌کند — مگر آن‌جا که `_` نوشته باشی.
- یک بازو در یک قدم هم تشخیص می‌دهد و هم داده را بیرون می‌کشد.
- بازوها از بالا به پایین امتحان می‌شوند و اولین تطبیق برنده است.
- نگهبان‌ها در شمارشِ جامع بودن حساب نمی‌شوند، پس زنجیره‌شان را با یک بازوی بی‌شرط تمام کن.
- `Option` شمارشیِ معمولی است و همین قواعد دربار‌ه‌اش صدق می‌کند.

### بعداً کامل‌تر می‌بینی

- **نحوِ کوتاهِ «فقط یک بازو برایم مهم است»** — [۱.۵.۵ — `if let`، `while let`، `let else`](../05-if-let-while-let-let-else/README.fa.md)
- **`Option` و کارهایی که بدونِ `match` می‌شود با آن کرد** — [۱.۶.۱ — `Option` و ایمنی در برابرِ null](../../06-absence-and-failure/01-option-and-null-safety/README.fa.md) و [۱.۶.۲](../../06-absence-and-failure/02-option-combinators/README.fa.md)
- **`Result` و `?`، که همین تطبیق را روی خطاها انجام می‌دهند** — [۱.۶.۳](../../06-absence-and-failure/03-result-and-question-mark/README.fa.md)
- **حالتِ اتصالِ پیش‌فرض، `ref`، الگوی برش، و بقیه‌ی زبانِ الگو** — [فاز ۲ — تطبیق الگو از نزدیک](../../../phase2-intermediate/08-rust-toolbox/01-pattern-matching-depth/README.fa.md)
- **`matches!` و ماکروها** — [فاز ۲ — `macro_rules!`](../../../phase2-intermediate/08-rust-toolbox/02-macro-rules-basics/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا `match` یک عبارت است و این چه چیزی را دربار‌ه‌ی بازوها اجباری می‌کند؟
- «جامع بودن» یعنی چه، و کامپایلر چطور اثباتش می‌کند؟
- اضافه کردنِ یک واریانت به یک شمارشی دقیقاً چه اتفاقی می‌اندازد؟
- چرا `_` روی یک شمارشیِ خودت معمولاً انتخابِ بدی است؟
- چرا دو بازوی نگهبان‌دار که روی هم همه‌چیز را می‌پوشانند باز هم جامع نیستند؟
- ترتیبِ بازوها چه فرقی می‌کند، و کامپایلر کِی دربار‌ه‌اش هشدار می‌دهد؟
- چرا `match` روی `Option` چیزِ تازه‌ای لازم ندارد؟

---

## بیشتر

- [کتابِ Rust — `match`](https://doc.rust-lang.org/book/ch06-02-match.html) — همین زمین، رسمی.
- [کتابِ Rust — الگوها و تطبیق](https://doc.rust-lang.org/book/ch19-00-patterns.html) — فصلِ کاملِ الگوها، از جمله چیزهایی که این‌جا فقط اشاره شدند.
- [`rustc --explain E0004`](https://doc.rust-lang.org/error_codes/E0004.html) — توضیحِ رسمیِ همین خطا. در ترمینال هم می‌شود گرفتش.
- [مرجعِ Rust — الگوها](https://doc.rust-lang.org/reference/patterns.html) — فهرستِ دقیقِ همه‌ی شکل‌های الگو.
- [`clippy::wildcard_enum_match_arm`](https://rust-lang.github.io/rust-clippy/master/#wildcard_enum_match_arm) — لینتی که `_`های خطرناک را پیدا می‌کند.

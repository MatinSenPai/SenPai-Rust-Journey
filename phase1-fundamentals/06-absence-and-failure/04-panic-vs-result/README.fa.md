# ۱.۶.۴ — پنیک در برابرِ `Result`

## در یک نگاه

بعد از این درس می‌توانی:

- برای یک شکستِ تازه تصمیم بگیری که باید `Result` برگرداند یا پنیک کند — بر اساسِ اینکه فراخواننده منطقاً می‌تواند رسیدگی‌اش کند یا نه، نه بر اساسِ سلیقه.
- `assert!`، `assert_eq!`، `unreachable!()` و `debug_assert!` را جای درستشان بنویسی، و بگویی وقتی پنیک می‌شود دقیقاً چه اتفاقی در پشته می‌افتد.
- یک پیامِ `.expect()` بنویسی که ساعتِ سه‌ی صبح، وقتی کسی دارد لاگ را می‌خواند، واقعاً کمکش کند — نه پیامی که فقط بگوید مقدار نبود.
- مرزِ کتابخانه، شروعِ برنامه و هندلرِ یک درخواست را برای این تصمیم از هم جدا کنی.

**زمان:** حدود ۵۵ دقیقه · **پیش‌نیاز:** [۱.۶.۳ — `Result` و علامتِ سؤال](../03-result-and-question-mark/README.fa.md)

---

## چرا اهمیت دارد

درسِ قبل `Result` را داد دستت: یک راهِ صریح برای گفتنِ «این ممکن است شکست بخورد، و این هم دلیلش». اما این تمامِ داستانِ شکست در Rust نیست. از همان ۱.۱.۲ یک چیزِ دیگر هم دیده‌ای: پنیک — وقتی یک `u8` از ۲۵۵ رد شود و در حالتِ دیباگ برنامه با یک پیام متوقف شود. و در ۱.۱.۶ دیدی که این رفتار حتی از overflow هم فراگیرتر است: ایندکس زدنِ بیرون از محدوده‌ی یک `Vec` حتی در انتشار هم پنیک می‌کند. و در ۱.۲.۵ یک وعده داده شد: «آیا اگر برنامه پنیک کند، مخرب‌ها (destructor) هنوز اجرا می‌شوند؟ این سؤال به ۱.۶.۴ می‌رسد.» امروز آن وعده را وفا می‌کنیم.

پس حالا دو تا ابزارِ کاملاً متفاوت برای «یک چیزی خراب شد» در دست داری، و این درس دربارهٔ چیزی نیست که تا حالا یاد نگرفته باشی — دربارهٔ **کدام‌یک را کِی** است. این یک تصمیمِ طراحیِ واقعی است که از فازِ ۳ به بعد مدام می‌گیری.

در پایتون این خط اصلاً کشیده نمی‌شود. یک `ValueError` از تایپ‌کردنِ غلطِ کاربر می‌آید، یک `IndexError` از یک باگِ خودِ برنامه — و هر دو یک `except` مشترک را رد می‌کنند. Rust این دو را از هم جدا می‌کند: یکی‌شان بخشی از امضای تابع می‌شود (`Result`) و مجبورت می‌کند رسیدگی کنی؛ آن یکی برنامه را متوقف می‌کند چون چیزی که قرار بود غیرممکن باشد، اتفاق افتاده. کامپایلر این خط را برایت نمی‌کشد — این را خودت باید بکشی، و همین درس یاد می‌دهد چطور.

---

## مفهوم

### قاعده‌ی تصمیم

قبل از هر جزئیاتی، خودِ قاعده — چون همه‌چیزِ زیرش شرحِ همین یک جمله است:

> **`Result` برای شکستی است که فراخواننده منطقاً می‌تواند رسیدگی‌اش کند. پنیک برای یک باگ است — یک قاعده‌ی شکسته، چیزی که قرار بود غیرممکن باشد.**

پرسشِ عملی‌اش این است: «این مقدار از کجا آمده؟» اگر از بیرونِ این برنامه آمده — چیزی که یک کاربر تایپ کرده، یک فایل، یک پاسخِ شبکه — رد شدنش عادی است، انتظارش را داشته باش، `Result` برگردان. اگر از خودِ این برنامه آمده — یک ساختارِ داده‌ای که خودت ساختی، یک پیش‌شرطی که خودِ کدت قرار بود تضمینش کند — رد شدنش یعنی یک‌جایی در همین کدبیس یک قول شکسته شده، و آن باگ است، نه ورودیِ بد.

دو تابعِ کنارِ هم، همین قاعده را نشان می‌دهند:

```rust
fn parse_priority(input: &str) -> Result<u8, String> {
    let value: u8 = match input.trim().parse() {
        Ok(value) => value,
        Err(_) => return Err(format!("'{input}' is not a whole number")),
    };
    if !(1..=5).contains(&value) {
        return Err(format!("priority must be between 1 and 5, got {value}"));
    }
    Ok(value)
}
```

```text
parse_priority("3"): Ok(3)
parse_priority("9"): Err("priority must be between 1 and 5, got 9")
parse_priority("abc"): Err("'abc' is not a whole number")
```

`input` چیزی است که یک آدم تایپ کرده. هرچیزی می‌تواند باشد، و بودنِ نامعتبرش عادی‌ترین اتفاقِ دنیاست — برای همین `Result` برمی‌گردد و هرگز پنیک نمی‌کند.

```rust
fn checked_midpoint(sorted_ascending: &[i32]) -> i32 {
    assert!(
        !sorted_ascending.is_empty(),
        "checked_midpoint: caller must not pass an empty slice"
    );
    sorted_ascending[sorted_ascending.len() / 2]
}
```

```text
checked_midpoint(&[10, 20, 30, 40, 50]): 30
```

این یکی فرق دارد. `sorted_ascending` را یک کاربر تایپ نکرده — یک بخشِ دیگرِ همین برنامه ساخته و به اینجا فرستاده. اگر خالی برسد، این خرابیِ ورودی نیست؛ یعنی یک‌جایی در همین برنامه قولش را زیرِ پا گذاشته. برای همین `assert!` می‌زند و پنیک می‌کند، نه `Result` برمی‌گرداند.

### `panic!` ساده — با شماره‌خط دقیق

`panic!` خودِ ماکرویی است که همه‌ی این‌ها رویش سوارند. یک جدولِ ناقص خوب نشانش می‌دهد:

```rust
fn region_code(name: &str) -> u8 {
    match name {
        "ir" => 98,
        "us" => 1,
        "de" => 49,
        other => panic!(
            "region_code: no dialing code registered for {other:?} — this table is supposed \
             to cover every region this program deploys to"
        ),
    }
}
```

```text
ir -> 98
us -> 1

thread 'main' (2224) panicked at phase1-fundamentals\06-absence-and-failure\04-panic-vs-result\examples\02-broken-invariant.rs:15:18:
region_code: no dialing code registered for "fr" — this table is supposed to cover every region this program deploys to
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

این جدول قرار است هر منطقه‌ای که این برنامه در آن مستقر می‌شود را پوشش دهد — این یک قاعده‌ی خودِ همین کدبیس است، نه چیزی که یک کاربر واردش کرده. وقتی جدول و واقعیت با هم جور درنمی‌آیند، آن یک باگ است، برای همین `Result`ی برنمی‌گرداند که هیچ فراخواننده‌ای انتظارش را ندارد — پنیک می‌کند. و ببین پیام چقدر دقیق است: فایل، شماره‌خط، شماره‌ستون، و بعدش دقیقاً همان متنی که خودت به `panic!` داده بودی.

### با `RUST_BACKTRACE=1` می‌بینی از کجا آمده

آن خطِ آخرِ خروجیِ بالا خودش راهنماست. همین برنامه را یک‌بارِ دیگر، این‌بار با آن متغیرِ محیطی اجرا کن:

```text
thread 'main' (34352) panicked at phase1-fundamentals\06-absence-and-failure\04-panic-vs-result\examples\02-broken-invariant.rs:15:18:
region_code: no dialing code registered for "fr" — this table is supposed to cover every region this program deploys to
stack backtrace:
   0: std::panicking::panic_handler
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library\std\src\panicking.rs:689
   1: core::panicking::panic_fmt
             at /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library\core\src\panicking.rs:80
   2: 02_broken_invariant::region_code
             at .\phase1-fundamentals\06-absence-and-failure\04-panic-vs-result\examples\02-broken-invariant.rs:15
   3: 02_broken_invariant::main
             at .\phase1-fundamentals\06-absence-and-failure\04-panic-vs-result\examples\02-broken-invariant.rs:25
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.
```

(فریم‌های تکراریِ راه‌اندازیِ زمانِ اجرا را از انتها حذف کرده‌ام تا خواناتر شود؛ بقیه دقیقاً همان چیزی است که ترمینال چاپ کرد.) فریم‌های ۰ و ۱ همیشه همین دو تایند — ماشین‌آلاتِ خودِ پنیک. فریم‌های ۲ و ۳ آن‌جایی‌اند که واقعاً به کارت می‌آیند: دقیقاً کدام تابع پنیک زد (`region_code`) و از کجا صدایش زده شد (`main`، خط ۲۵). در یک برنامه‌ی بزرگ با بیست لایه فراخوانی، این همان چیزی است که مسیر را نشانت می‌دهد.

### `assert!` و `assert_eq!` — نگهبانِ یک شرط

`assert!(condition, "message")` که در `checked_midpoint` دیدی چیزی نیست جز «اگر شرط نادرست بود، پنیک کن». خواهرِ رایجش `assert_eq!` است — مقایسه‌ی دو مقدار، با یک تفاوتِ مهم در خروجی وقتی شکست بخورد:

```rust
fn double(n: i32) -> i32 {
    n + n
}

assert_eq!(double(6), 11);
```

```text
thread 'main' (26600) panicked at phase1-fundamentals\06-absence-and-failure\04-panic-vs-result\examples\03-assert-eq-mismatch.rs:19:5:
assertion `left == right` failed
  left: 12
 right: 11
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

نکته این‌جاست: `assert!(double(6) == 11)` فقط می‌گفت «برابر نبودند» — تو باید حدس می‌زدی کدام طرف چه بود. `assert_eq!` هر دو مقدار را نگه می‌دارد و وقتی شکست می‌خورد، هر دو را کنارِ هم نشانت می‌دهد. `assert_ne!` هم هست، برای وقتی می‌خواهی مطمئن شوی دو چیز **متفاوتند**.

### `unreachable!()` — وقتی یک شاخه واقعاً نباید برسد

از ۱.۵.۴ یادت هست که `match` باید همه‌ی حالت‌ها را پوشش دهد. گاهی یک تابع همه‌ی مقدارهایی که *نوعش* اجازه می‌دهد را در `match` می‌آورد، اما یکی از آن شاخه‌ها فقط به‌خاطرِ یک قراردادی که جای دیگری تضمین شده، هرگز نباید اجرا شود:

```rust
fn priority_label(level: u8) -> &'static str {
    match level {
        1 | 2 => "low",
        3 => "normal",
        4 | 5 => "high",
        other => unreachable!(
            "priority_label: level {other} was never validated by parse_priority (must be 1..=5)"
        ),
    }
}
```

```text
thread 'tests::panics_when_the_caller_skips_validation' (30060) panicked at src\lib.rs:56:18:
internal error: entered unreachable code: priority_label: level 9 was never validated by parse_priority (must be 1..=5)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test tests::panics_when_the_caller_skips_validation - should panic ... ok
```

`u8` هزاران مقدار می‌گیرد، اما `priority_label` مستند کرده که فقط با خروجیِ معتبرِ `parse_priority` (یعنی ۱ تا ۵) صدا زده می‌شود. شاخه‌ی `other` باید یک‌جا باشد چون کامپایلر برای هر `u8` یک شاخه می‌خواهد — ولی رسیدنِ واقعی به آن یعنی یک‌جایی این قرارداد را دور زده‌اند. `unreachable!()` دقیقاً همین را می‌گوید: «طبقِ آنچه من می‌دانم، این نقطه هرگز اجرا نمی‌شود؛ اگر شد، جایی باگی هست.»

### `debug_assert!` — بررسی‌ای که در انتشار حذف می‌شود

همان `checked_midpoint` بالاتر، در فایلِ واقعیِ `01-panic-or-result.rs`، یک خطِ دیگر هم دارد که این‌جا نیاوردیم:

```text
debug_assert!(
    sorted_ascending.windows(2).all(|pair| pair[0] <= pair[1]),
    "checked_midpoint: caller must pass an ascending slice"
);
```

این همان معامله‌ای است که در ۱.۱.۲ با overflow دیدی: بررسیِ صحتِ ترتیب چیزی مجانی نیست — باید کلِ برش را یک‌بار پیمود. `assert!` همیشه اجرا می‌شود، حتی در `--release`، چون خالی‌بودنِ برش یک قاعده‌ی سبک است که همیشه ارزشِ چک‌کردن دارد. `debug_assert!` فقط در ساختِ دیباگ **وجود دارد** — در انتشار نه رد می‌شود و نه ساکت می‌ماند، اصلاً کامپایل نمی‌شود. برای بررسی‌های گران که فقط برای پیدا کردنِ باگ در توسعه لازمند، این دقیقاً همان چیزی است که می‌خواهی.

### وقتی panic! می‌شود، `Drop` هنوز اجرا می‌شود

این بخشِ غافلگیرکننده‌ی درس است، و ۱.۲.۵ همین‌جا به آن اشاره کرده بود. سه `Guard` را بساز، بعد وسطِ کار پنیک بزن:

```rust
struct Guard {
    name: &'static str,
}

impl Guard {
    fn new(name: &'static str) -> Guard {
        println!("open  {name}");
        Guard { name }
    }
}
```

```rust
impl Drop for Guard {
    fn drop(&mut self) {
        println!("close {}", self.name);
    }
}

fn main() {
    let _a = Guard::new("a");
    let _b = Guard::new("b");
    let _c = Guard::new("c");
    println!("all three open, about to fail partway through setup");
    panic!("simulated failure while building the fourth resource");
}
```

```text
open  a
open  b
open  c
all three open, about to fail partway through setup

thread 'main' (18256) panicked at phase1-fundamentals\06-absence-and-failure\04-panic-vs-result\examples\06-drop-during-unwind.rs:36:5:
simulated failure while building the fourth resource
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
close c
close b
close a
```

پیامِ پنیک وسطِ خروجی نشسته — و بعدش سه تا `close`، همان ترتیبِ معکوسی که ۱.۲.۵ برای خروجِ عادی از یک بلوک نشان داده بود. پنیک هیچ‌کدام از آن سه `Guard` را جا نگذاشت.

دلیلش یک اسم دارد: **بازگشاییِ پشته (unwinding)**. رفتارِ پیش‌فرضِ Rust این است که پنیک، به‌جای اینکه فوراً پردازه را بکشد، از تابعی که در آن رخ داده به سمتِ بیرون برمی‌گردد — دقیقاً مثلِ یک `return` اجباری از هر تابعِ روی پشته، یکی‌یکی — و در هر قدم، هر مقداری که هنوز در محدوده بود مخرب‌اش را اجرا می‌کند. برای همین `_a`، `_b` و `_c` با اینکه هرگز به آخرِ `main` نرسیدند، باز هم بسته شدند.

```senpai-visual
{"kind":"concept","labels":["ساختِ a","ساختِ b","ساختِ c","panic!","بازگشاییِ پشته","آزادیِ c، b، a"]}
```

### `panic = "abort"` — وقتی بازگشایی را کنار می‌گذاری

بازگشایی مجانی نیست: باید بداند در هر قدم چه چیزی را باید تمیز کند، و این هم به زمان نیاز دارد هم به فضا در باینری. Rust یک انتخابِ دوم هم می‌دهد — در `Cargo.toml`:

```toml
[profile.release]
panic = "abort"
```

با این تنظیم، پنیک دیگر برنمی‌گردد؛ کلِ پردازه همان‌جا متوقف می‌شود. بدونِ بازگشایی، بدونِ `Drop`. باینری کوچک‌تر و سریع‌تر می‌شود، و برای برنامه‌هایی که قرار نیست از یک پنیک برگردند (یا در سیستم‌های خیلی محدود، یا وقتی از یک زبانِ دیگر مثلِ C صدا زده می‌شوند و بازگشایی از آن مرز عبور نمی‌کند) انتخابِ رایجی است. این ورک‌اسپیس این تنظیم را نگذاشته، ولی می‌توانی همین رفتار را بدونِ دست‌زدن به هیچ فایلی، فقط با یک متغیرِ محیطی امتحان کنی:

```text
open  a
open  b
open  c
all three open, about to fail partway through setup

thread 'main' (4780) panicked at phase1-fundamentals\06-absence-and-failure\04-panic-vs-result\examples\06-drop-during-unwind.rs:36:5:
simulated failure while building the fourth resource
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

همان برنامه، همان سه `Guard`، ولی این‌بار حتی یک خطِ `close` هم چاپ نشد. با `CARGO_PROFILE_DEV_PANIC=abort` پردازه دقیقاً سرِ همان خطِ پنیک متوقف شد — بدونِ برگشتن از پشته، بدونِ صدا زدنِ هیچ مخربی. سیستم‌عامل فایل‌ها و سوکت‌ها را خودش وقتی پردازه می‌میرد می‌بندد؛ چیزی که واقعاً از دست می‌رود، منطقِ تمیزکاریِ خودِ برنامه‌ات است — نوشتنِ یک رکوردِ آخر در لاگ، آزاد کردنِ یک قفلِ سطحِ‌برنامه، پاک کردنِ یک فایلِ موقت. برای همین `panic = "abort"` یک تصمیمِ رایگان نیست؛ یک معامله است.

### `.unwrap()` و `.expect()` — «دارم ادعا می‌کنم این نمی‌تواند شکست بخورد»

از ۱.۶.۱ یادت هست که `.unwrap()` روی یک `None` پنیک می‌کند. حالا دقیق‌تر ببینش:

```rust
fn find_config_path<'a>(candidates: &[&'a str], wanted: &str) -> Option<&'a str> {
    candidates.iter().find(|&&c| c == wanted).copied()
}

let path = find_config_path(&["dev.toml", "staging.toml"], "prod.toml").unwrap();
```

```text
looking for prod.toml among ["dev.toml", "staging.toml"]

thread 'main' (24568) panicked at phase1-fundamentals\06-absence-and-failure\04-panic-vs-result\examples\04-unwrap-panics.rs:18:59:
called `Option::unwrap()` on a `None` value
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

این پیام از صد جای مختلفِ کدبیس می‌تواند بیاید و همیشه یک شکل است. `.unwrap()` یعنی «دارم ادعا می‌کنم این هرگز `None`/`Err` نیست» — و وقتی ادعایت غلط از آب دربیاید، این تنها چیزی است که نصیبت می‌شود.

`.expect()` همان ادعا را می‌کند، ولی می‌گذارد بگویی *چرا* فکر می‌کردی نمی‌تواند شکست بخورد:

```rust
let path = find_config_path(&["dev.toml", "staging.toml"], "prod.toml")
    .expect("prod.toml must be listed in `candidates` — deploy config is incomplete");
```

```text
looking for prod.toml among ["dev.toml", "staging.toml"]

thread 'main' (5572) panicked at phase1-fundamentals\06-absence-and-failure\04-panic-vs-result\examples\05-expect-with-a-good-message.rs:19:10:
prod.toml must be listed in `candidates` — deploy config is incomplete
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

فرق را ببین: اولی می‌گوید یک `None` بود — چیزی که خودت هم از قبل می‌دانستی. دومی می‌گوید کدام فرض شکسته شد و این فرض چه معنایی داشت. قاعده یک جمله است: **پیام باید قولی را نام ببرد که فکر می‌کردی برقرار است، نه علامتِ شکست را توصیف کند.** `.expect("no path found")` هیچ بهتر از `.unwrap()` نیست — فقط همان چیزی را که کامپایلر رایگان می‌داد، دوباره با کلماتِ خودت گفته‌ای. `.expect("prod.toml must be listed in candidates — deploy config is incomplete")` چیزِ دیگری می‌گوید: به کسی که ساعتِ سه‌ی صبح این پیام را می‌بیند می‌گوید دقیقاً کجا برود نگاه کند.

### `#[should_panic]` — تستی که انتظارِ پنیک دارد

وقتی پنیک‌کردن دقیقاً همان رفتاریست که می‌خواهی، تست هم باید همین را ادعا کند، نه اینکه شکستِ تست را با شکستِ برنامه اشتباه بگیری:

```rust
#[test]
#[should_panic(expected = "must not pass an empty slice")]
fn panics_on_an_empty_slice() {
    checked_midpoint(&[]);
}
```

```text
running 9 tests
test tests::finds_the_last_digit ... ok
test tests::labels_every_valid_priority ... ok
test tests::finds_the_midpoint ... ok
test tests::panics_on_an_empty_slice - should panic ... ok
test tests::panics_when_the_caller_skips_validation - should panic ... ok
test tests::panics_on_empty_values - should panic ... ok
test tests::rejects_out_of_range_priorities_without_panicking ... ok
test tests::parses_valid_priorities ... ok
test tests::rejects_unparsable_priorities_without_panicking ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`expected = "..."` فقط یک زیررشته می‌خواهد، نه کلِ پیام — کافی است همان قسمتی را بنویسی که ثابت می‌کند این پنیکِ درست بود، نه یک پنیکِ اتفاقیِ دیگر که تصادفاً همان‌جا رخ داده. بدونِ `expected`، `#[should_panic]` فقط می‌پرسد «آیا اصلاً پنیک کرد؟» — کافی، ولی کمتر دقیق.

### مرزهای واقعی: کتابخانه، شروعِ برنامه، هندلرِ یک درخواست

قاعده‌ی بالا («فراخواننده می‌تواند رسیدگی کند یا نه؟») بسته به این‌که کجای یک سیستم ایستاده‌ای جوابِ متفاوتی می‌دهد:

| جایگاه | چه‌کاری درست است | چرا |
|---|---|---|
| کتابخانه — تابعی که به بقیه می‌دهی | `Result` برگردان | تصمیم با فراخواننده است، نه با تو؛ او شاید بخواهد دوباره امتحان کند، پیشِ‌فرض بگذارد، یا خودش پنیک کند |
| شروعِ برنامه — خواندنِ کانفیگ در `main` | پنیک (با `.expect()`ی که پیام‌دار است) مجاز است | اگر تنظیمات غلط باشد ادامه‌دادن بی‌معنی است، و کسی داخلِ برنامه نیست که «رسیدگی» کند |
| هندلرِ یک درخواست — تابعی که برای هر کاربر دوباره اجرا می‌شود | هرگز روی ورودیِ کاربر پنیک نکن | یک ورودیِ بد نباید کلِ سرویس یا حلقه را با خودش پایین بکشد؛ `Result` برگردان و پاسخ بده |

سطرِ سوم را الان فقط به‌عنوانِ یک اصل بسپار — [۲.۱ در فازِ ۳](../../../phase3-backend-foundations/02-axum-and-rest-api-design/01-routing-handlers-extractors/README.fa.md) دقیقاً نشانت می‌دهد یعنی چه، وقتی یک هندلرِ HTTP بنویسی که برای صدها درخواست در ثانیه صدا زده می‌شود و یک `.unwrap()` رویِ بدنه‌ی JSON‌یِ یک کاربر می‌تواند کلِ آن ریسمان را بیندازد.

### `catch_unwind` — نامش را می‌دانی، امروز نه

یک راهِ چهارم هم هست: `std::panic::catch_unwind` می‌تواند یک پنیک را در همان جایی که رخ می‌دهد بگیرد و به‌جایش یک مقدار برگرداند — بدونِ اینکه کلِ برنامه پایین بیاید. دو دلیل هست که این درس رویش نمی‌ایستد. یکی این‌که استفاده‌ی درستش (مثلاً نگه‌داشتنِ یک استخرِ نخ زنده وقتی یکی از کارهایش پنیک می‌زند) به `Arc`، نخ‌ها و `Send`/`Sync` نیاز دارد که هنوز نداری. دیگر این‌که، حتی وقتی داشته باشی، `catch_unwind` جایگزینِ `Result` برای اعتبارسنجیِ ورودی نیست — طراحی‌اش برای مرزهاست (یک استخرِ نخ، یک افزونه‌ی جدا)، نه برای تصمیم‌های روزمره‌ای که همین درس درباره‌شان بود. فقط بدان که هست، و اسمش را جایی که واقعاً به کارت بیاید دوباره می‌بینی.

---

## دست‌به‌کد

```sh
cargo run -p p1-06-04-panic-vs-result --example 01-panic-or-result
```

بعد پنج‌تای پنیک‌کننده:

```sh
cargo run -p p1-06-04-panic-vs-result --example 02-broken-invariant --features broken
cargo run -p p1-06-04-panic-vs-result --example 03-assert-eq-mismatch --features broken
cargo run -p p1-06-04-panic-vs-result --example 04-unwrap-panics --features broken
cargo run -p p1-06-04-panic-vs-result --example 05-expect-with-a-good-message --features broken
cargo run -p p1-06-04-panic-vs-result --example 06-drop-during-unwind --features broken
```

بعد همان پنیکِ ساده را با ردِ فراخوانی‌ها ببین:

```sh
RUST_BACKTRACE=1 cargo run -p p1-06-04-panic-vs-result --example 02-broken-invariant --features broken
```

و بعد `06-drop-during-unwind` را یک‌بارِ دیگر، این‌بار بدونِ بازگشایی:

```sh
# PowerShell
$env:CARGO_PROFILE_DEV_PANIC='abort'; cargo run -p p1-06-04-panic-vs-result --example 06-drop-during-unwind --features broken
# bash
CARGO_PROFILE_DEV_PANIC=abort cargo run -p p1-06-04-panic-vs-result --example 06-drop-during-unwind --features broken
```

بعد این‌ها را امتحان کن:

۱. در `01-panic-or-result`، به `checked_midpoint` یک برشِ نزولی بده (مثلاً `&[5, 3, 1]`). چرا در دیباگ پنیک می‌کند ولی در `--release` نه؟
۲. در `02-broken-invariant`، `"fr" => 33` را به جدول اضافه کن. آیا این درستِ درست است، یا فقط پنیک را یک قدم عقب‌تر برده؟ یک جمله بنویس درباره‌ی این‌که این جدول اصلاً باید کامل باشد یا نه.
۳. در `06-drop-during-unwind`، یک `Guard` چهارم بساز و پنیک را بعد از آن بگذار. ترتیبِ `close`ها چطور عوض می‌شود؟

---

## خطاهایی که خواهی دید

### پنیکِ ساده — پیام و شماره‌خطِ دقیق

```text
thread 'main' (2224) panicked at phase1-fundamentals\06-absence-and-failure\04-panic-vs-result\examples\02-broken-invariant.rs:15:18:
region_code: no dialing code registered for "fr" — this table is supposed to cover every region this program deploys to
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**چه اتفاقی افتاده:** `region_code` هر منطقه‌ای که وجود دارد را در جدولش می‌خواست، ولی `"fr"` آن‌جا نبود. این یک خطای کامپایل نیست — برنامه کامل ساخته شد؛ مشکل در زمانِ اجرا، وقتی جدول و ورودیِ واقعی روبه‌روی هم قرار گرفتند، ظاهر شد.

**راه‌حل:** یا `"fr"` را به جدول اضافه کن (اگر واقعاً همه‌ی مناطق باید آن‌جا باشند)، یا اگر جدول ذاتاً می‌تواند ناقص بماند، امضای تابع را عوض کن به `Option<u8>` و دیگر پنیک نزن.

**چرا این راه‌حل است:** انتخاب بینِ این دو به یک واقعیتِ بیرون از کد بستگی دارد — آیا این جدول واقعاً قرار است همه‌چیز را پوشش دهد؟ اگر بله، پنیک درست است و باگ در جدول است. اگر نه، پنیک از اول اشتباه بود؛ این هرگز یک قاعده‌ی همیشه‌درست نبوده.

### شکستِ `assert_eq!` — دیفِ چپ/راستِ واقعی

```text
thread 'main' (26600) panicked at phase1-fundamentals\06-absence-and-failure\04-panic-vs-result\examples\03-assert-eq-mismatch.rs:19:5:
assertion `left == right` failed
  left: 12
 right: 11
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**چه اتفاقی افتاده:** `assert_eq!(double(6), 11)` نوشته شده، ولی `double(6)` واقعاً ۱۲ است. `left` و `right` دقیقاً همان چیزی‌اند که به ماکرو داده‌ای، به همان ترتیب — نه «درست» و «غلط».

**راه‌حل:** ببین کدام طرف واقعاً اشتباه است. اگر `double` باید ۶ را به ۱۲ ببرد (که همین کار را می‌کند)، انتظارِ نوشته‌شده در `assert_eq!` غلط بود — آن را به ۱۲ تصحیح کن.

**چرا این راه‌حل است:** یک `assert_eq!` شکسته دو تا مظنون دارد، نه یکی: خودِ تابع، یا انتظاری که از آن نوشته‌ای. پیام فقط می‌گوید «این دو برابر نبودند» — تشخیصِ این‌که کدام‌شان درست بود، کارِ خودت است، و این‌جا `double` بی‌گناه است.

### `.unwrap()` بی‌پیام در برابرِ `.expect()` با پیامِ خوب

```text
thread 'main' (24568) panicked at phase1-fundamentals\06-absence-and-failure\04-panic-vs-result\examples\04-unwrap-panics.rs:18:59:
called `Option::unwrap()` on a `None` value
```

**چه اتفاقی افتاده:** `find_config_path` برای `"prod.toml"` چیزی پیدا نکرد و `None` برگرداند؛ `.unwrap()` رویش پنیک زد. پیام فقط می‌گوید یک `None` بود — همان چیزی که خودِ نوعِ `Option` هم می‌گفت.

**راه‌حل:**

```text
.expect("prod.toml must be listed in `candidates` — deploy config is incomplete")
```

که پنیکِ زیر را می‌دهد:

```text
thread 'main' (5572) panicked at phase1-fundamentals\06-absence-and-failure\04-panic-vs-result\examples\05-expect-with-a-good-message.rs:19:10:
prod.toml must be listed in `candidates` — deploy config is incomplete
```

**چرا این راه‌حل است:** پنیک هنوز اتفاق می‌افتد — این فیکس، شکست را ناپدید نمی‌کند، چون خودِ خرابی (نبودنِ `prod.toml`) جای دیگری است. آنچه فیکس می‌شود صداقتِ پیام است: کسی که این خط را در لاگ می‌بیند دیگر لازم نیست حدس بزند «کدام `None`؟» — پیام همان‌جا اسمِ فرضی را می‌برد که شکسته شده.

---

## تمرین

### گرم‌کردن

<details>
<summary>یک تابع متنی را که کاربر تایپ کرده پارس می‌کند و ممکن است نامعتبر باشد. <code>Result</code> یا پنیک؟</summary>

`Result`. ورودیِ کاربر بیرون از برنامه است؛ نامعتبر بودنش عادی است، نه باگ.

</details>

<details>
<summary>یک تابعِ داخلی فرض می‌کند آرگومانش هرگز خالی نیست، چون تنها جایی که صدایش می‌زند از قبل آن را پر کرده. اگر خالی برسد؟</summary>

پنیک — با `assert!` یا `.expect()`. خالی‌بودن این‌جا یعنی یک‌جای دیگرِ همین برنامه قولش را زیرِ پا گذاشته؛ این باگ است، نه ورودیِ بد.

</details>

<details>
<summary>یک <code>debug_assert!</code> با شرطِ نادرست در کد هست. <code>cargo run</code> پنیک می‌کند؟ <code>cargo run --release</code> چطور؟</summary>

در دیباگ بله. در `--release` نه — چون `debug_assert!` در انتشار اصلاً کامپایل نمی‌شود، رد نمی‌شود، غایب است.

</details>

<details>
<summary><code>.expect("داشتن ندارد پیام")</code> روی یک <code>Some(x)</code> چه چاپ می‌کند؟</summary>

هیچی، و پنیک هم نمی‌زند. `.expect()` فقط وقتی صدایش درمی‌آید که مقدار واقعاً غایب باشد — روی `Some`/`Ok` فقط مقدارِ داخل را برمی‌گرداند.

</details>

<details>
<summary>سه <code>Guard</code> با نام‌های <code>a</code>، <code>b</code>، <code>c</code> باز می‌شوند، بعد پنیک می‌زنی. ترتیبِ <code>close</code>ها؟</summary>

`c`، بعد `b`، بعد `a` — همان ترتیبِ معکوسِ همیشگیِ ۱.۲.۵، چون بازگشایی هم از همان قاعده پیروی می‌کند.

</details>

<details>
<summary>درست یا غلط: <code>catch_unwind</code> راهِ درستِ رسیدگی به ورودیِ نامعتبرِ کاربر است.</summary>

غلط. `catch_unwind` برای مرزها ساخته شده (استخرِ نخ، افزونه)، نه برای اعتبارسنجیِ روزمره. برای ورودیِ کاربر، جواب همیشه `Result` است.

</details>

### تعمیر

`examples/03-assert-eq-mismatch.rs` را درست کن — ولی اول تشخیص بده کدام طرف واقعاً اشتباه است: خودِ `double`، یا عددی که در `assert_eq!` نوشته شده؟ (یک جمله بنویس که چطور فهمیدی.)

`examples/04-unwrap-panics.rs` را طوری درست کن که پیامش، اگر باز هم پنیک بزند، واقعاً کمک کند — دقیقاً همان چیزی که در `05-expect-with-a-good-message.rs` دیدی. سپس یک نسخه‌ی دوم بنویس که اصلاً پنیک نزند: از `.unwrap_or(...)` استفاده کن و برگرد به یک مسیرِ پیش‌فرض. کدام‌یک برای این تابع درست‌تر است، بسته به این‌که `find_config_path` را چه کسی صدا می‌زند؟

`examples/02-broken-invariant.rs` را دو جور درست کن: یک‌بار با کامل‌کردنِ جدول، یک‌بار با تغییرِ امضا به `Option<u8>` تا دیگر هرگز پنیک نزند. کدام‌یک را واقعاً انتخاب می‌کنی به این بستگی دارد که آیا این جدول قرار است همه‌چیز را پوشش دهد یا نه — و این چیزی نیست که خودِ کد بتواند به تو بگوید.

### پیاده‌سازی

چهار تابع در `src/lib.rs`:

```sh
cargo test -p p1-06-04-panic-vs-result
```

دوتایشان (`parse_priority`، `checked_midpoint`) را از بالا می‌شناسی. دوتای دیگر تازه‌اند: `priority_label` باید مثلِ نمونه‌ی بالا `unreachable!()` بزند، و `last_digit_of` باید با `.expect()` پیامی بنویسد که فرض را نام می‌برد، نه علامتِ شکست را.

### بساز

یک `pub fn` بنویس (اسم و امضا با خودت) که یک تنظیمِ لازم را از یک `HashMap<String, String>` می‌خواند — مثلِ خواندنِ یک متغیرِ محیطی هنگامِ بالا آمدنِ برنامه. اگر کلید غایب بود، با `.expect()` و پیامی که دقیقاً می‌گوید کدام کلید و چرا لازم است پنیک کن.

بعد یک تابعِ دومِ کنارش بنویس که یک فیلدِ ارسالی‌ی کاربر (مثلاً یک ایمیل یا یک شماره) را پارس می‌کند و `Result` برمی‌گرداند.

بعد یک جمله بنویس: چرا این دو، با اینکه هر دو در ظاهر «جستجوی یک متن که ممکن است نباشد»اند، در نوع اساساً با هم فرق دارند؟

### چالش (اختیاری)

**بخشِ یک.** مستنداتِ [`std::panic::catch_unwind`](https://doc.rust-lang.org/std/panic/fn.catch_unwind.html) را بخوان. یک صدازدنِ `checked_midpoint(&[])` را داخلش بپیچ و چاپ کن چه چیزی برمی‌گردد. این به جلو می‌رود — استفاده‌ی واقعی‌اش وقتی معنا پیدا می‌کند که نخ‌ها را در [فازِ ۲](../../../phase2-intermediate/07-concurrency-and-async/01-threads-mutex-arc/README.fa.md) ببینی، ولی شکلِ برگشتی‌اش را همین امروز می‌توانی ببینی.

**بخشِ دو.** `06-drop-during-unwind` را با `CARGO_PROFILE_DEV_PANIC=abort` روی `--release` هم امتحان کن. آیا چیزی فرق می‌کند نسبت به دیباگِ abort‑شده؟

**بخشِ سه.** یک تستِ `#[should_panic(expected = "...")]` برای `priority_label(0)` بنویس، بدونِ نگاه‌کردن به پیامِ دقیقِ `unreachable!()` — فقط از رویِ چیزی که خودت در پیام گذاشته‌ای حدس بزن، بعد اجرا کن و ببین درست حدس زدی یا نه.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به کارت می‌آید |
|---|---|---|
| `panic!` | توقفِ فوریِ نخِ جاری با یک پیام | یک قاعده‌ی خودِ برنامه شکسته شده |
| `assert!` / `assert_eq!` / `assert_ne!` | پنیک اگر شرط/برابری/نابرابری برقرار نبود | نگهبانِ یک پیش‌شرط |
| `unreachable!()` | پنیک در شاخه‌ای که طبقِ یک قرارداد هرگز نباید اجرا شود | `match`ی که نوعاً باید کامل باشد ولی معنایی نه |
| `debug_assert!` | مثلِ `assert!`، ولی در انتشار اصلاً کامپایل نمی‌شود | بررسیِ گران، فقط برای توسعه |
| بازگشاییِ پشته (unwinding) | برگشتنِ تدریجی از پشته هنگامِ پنیک، با اجرای هر `Drop` سرِ راه | رفتارِ پیش‌فرضِ Rust |
| `panic = "abort"` | پردازه همان‌جا متوقف می‌شود؛ نه بازگشایی، نه `Drop` | باینریِ کوچک‌تر، بدونِ تمیزکاری |
| `RUST_BACKTRACE=1` | نشان‌دادنِ مسیرِ فراخوانی‌هایی که به پنیک رسیدند | پیداکردنِ محلِ واقعیِ خطا |
| `.expect("...")` | مثلِ `.unwrap()`، ولی پیام قولِ شکسته‌شده را نام می‌برد | همیشه به‌جایِ `.unwrap()` در کدِ واقعی |
| `#[should_panic]` | تستی که فقط اگر پنیک بزند پاس می‌شود | وقتی پنیک‌کردن خودش رفتارِ درست است |
| `catch_unwind` | گرفتنِ یک پنیک در مرز، به‌جایِ اجازه‌دادن به سقوطِ کامل | استخرِ نخ، افزونه — نه اعتبارسنجیِ روزمره |

### الان می‌دانی

- `Result` برای شکستی است که فراخواننده منطقاً می‌تواند رسیدگی کند؛ پنیک برای یک قاعده‌ی شکسته‌ی خودِ برنامه است.
- `assert!`/`assert_eq!`/`unreachable!()` نگهبانِ پیش‌شرط‌ها و شاخه‌های غیرممکن‌اند؛ `debug_assert!` همان کار را می‌کند ولی در انتشار غایب است.
- پنیک به‌طورِ پیش‌فرض پشته را باز می‌کند و هر `Drop`ی که سرِ راه است را اجرا می‌کند — دقیقاً همان ترتیبِ معکوسِ ۱.۲.۵.
- `panic = "abort"` این بازگشایی را کنار می‌گذارد؛ پردازه فوراً متوقف می‌شود، بدونِ `Drop`.
- `.expect()` باید قولی را نام ببرد که فکر می‌کردی برقرار است، نه علامتِ شکست را توصیف کند.
- `#[should_panic(expected = "...")]` تستی می‌سازد که پنیکِ درست را از هر پنیکِ دیگری جدا می‌کند.
- کتابخانه `Result` برمی‌گرداند، شروعِ برنامه می‌تواند پنیک کند، هندلرِ یک درخواست هرگز نباید روی ورودیِ کاربر پنیک بزند.

### بعداً کامل‌تر می‌بینی

- **تبدیلِ خودکارِ خطا با `From` و `?`** — [۱.۶.۵ — `From` و تبدیلِ خطا](../05-from-and-error-conversion/README.fa.md)
- **نوعِ خطای خودت، به‌جای `String`** — [فازِ ۲ — نوع‌های خطای سفارشی](../../../phase2-intermediate/04-error-handling-and-lifetimes/01-custom-error-types/README.fa.md)
- **`thiserror` و `anyhow`، ابزارهای واقعیِ صنعتی برای همین کاری که امروز با `String` انجام دادیم** — [فازِ ۲ — `thiserror` و `anyhow`](../../../phase2-intermediate/04-error-handling-and-lifetimes/02-thiserror-and-anyhow/README.fa.md)
- **پنیک در یک نخِ جدا، و اینکه چرا `Mutex` را «مسموم» می‌کند** — [فازِ ۲ — نخ‌ها، `Mutex` و `Arc`](../../../phase2-intermediate/07-concurrency-and-async/01-threads-mutex-arc/README.fa.md)
- **هندلرِ HTTP‌ای که هرگز نباید روی ورودیِ کاربر پنیک بزند** — [فازِ ۳ — مسیرها و هندلرها با Axum](../../../phase3-backend-foundations/02-axum-and-rest-api-design/01-routing-handlers-extractors/README.fa.md)
- **قالب‌بندیِ یکدستِ خطاها برای پاسخِ API** — [فازِ ۳ — پاکت‌های یکدستِ خطا](../../../phase3-backend-foundations/07-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.fa.md)

### می‌توانی توضیح بدهی؟

- قاعده‌ی تصمیم بینِ `Result` و پنیک را در یک جمله بگو.
- فرقِ `assert!` و `debug_assert!` چیست، و کدام‌یک در `--release` باقی می‌ماند؟
- وقتی پنیک می‌شود، دقیقاً چه اتفاقی برای مقدارهایی که هنوز در محدوده بودند می‌افتد؟
- `panic = "abort"` چه چیزی را عوض می‌کند؟
- یک پیامِ بدِ `.expect()` را از یک پیامِ خوب چطور تشخیص می‌دهی؟
- چرا یک هندلرِ درخواست هرگز نباید روی ورودیِ کاربر پنیک بزند، حتی اگر همان کدِ دقیقاً همین کار را در `main` انجام دادنش مجاز بود؟

---

## بیشتر

- [کتابِ Rust — رسیدگی به خطا](https://doc.rust-lang.org/book/ch09-00-error-handling.html) — همین زمین، رسمی، با فصلِ کاملی درباره‌ی این‌که کِی پنیک و کِی `Result`.
- [`std::panic`](https://doc.rust-lang.org/std/panic/index.html) — مستنداتِ ماژولی که `catch_unwind` و `PanicHookInfo` در آن‌اند.
- [`clippy::unwrap_used` و `clippy::expect_used`](https://rust-lang.github.io/rust-clippy/master/#unwrap_used) — لینت‌های محدودکننده‌ای که در کدِ کتابخانه‌ای/بک‌اند روشنشان می‌کنی تا `.unwrap()`ِ فراموش‌شده اصلاً کامپایل نشود.

# ۲.۷.۴ — تستِ خاصیت‌محور با `proptest`، تستِ عکس‌برداری با `insta`

## در یک نگاه

بعد از این درس می‌توانی:

- توضیح بدهی یک «خاصیت» (property) چیست — قانونی که باید برایِ *هر* ورودی برقرار بماند، نه فقط چندتایی که خودت انتخاب کرده‌ای — و یک بلوکِ `proptest!` واقعی برایش بنویسی.
- یک شکستِ واقعیِ کوچک‌شده‌ی proptest را بخوانی و بگویی کوچک‌ترین ورودی‌ای که خاصیت را نقض کرده دقیقاً چیست و چرا.
- یک خروجیِ پیچیده را به‌عنوانِ عکسِ insta ثبت کنی، و وقتی دیفِ یک اجرایِ بعدی رویِ صفحه ظاهر شد، بینِ تأییدکردن و ردکردن درست انتخاب کنی.

**زمان:** حدود ۶۵ دقیقه · **پیش‌نیاز:**
[۲.۷.۳ — بدل‌های تست در Rust](../03-test-doubles-in-rust/README.fa.md)

---

## چرا اهمیت دارد

تا این‌جا (۲.۷.۱ تا ۲.۷.۳) هر تستی که نوشتی یک شکل داشت: چند ورودیِ مشخص را خودت انتخاب می‌کردی، جوابِ درستش را هم خودت حساب می‌کردی، و با `assert_eq!` مقایسه‌شان می‌کردی. برایِ اکثرِ کد این کاملاً کافی است — و بعد از این درس هم همچنان همین‌طور می‌ماند.

ولی یک ضعفِ ساختاری دارد: تو داری ورودی‌ها را انتخاب می‌کنی، و باگ دقیقاً عاشقِ همان ورودی‌ای است که فکرش را نکردی. یک تابعِ رفت‌وبرگشتی — چیزی را format می‌کنی و بعد parse، یا encode می‌کنی و بعد decode — را با سه‌تا عددِ مثبتِ کوچک تست کن: سبز می‌شود. عددِ منفی را هیچ‌وقت امتحان نکردی؟ باگ همان‌جا زندگی می‌کند، بی‌سروصدا، چون تستت هیچ‌وقت آن‌جا نگاه نکرد.

مشکلِ دوم کاملاً جداست: بعضی خروجی‌ها اصلاً کوچک نیستند. یک گزارشِ چندخطی، یک ساختارِ تودرتو که خروجیِ Debugش صفحه را پر می‌کند — دستی نوشتنِ `assert_eq!(output, "...")` برایِ همچین چیزی هم خسته‌کننده است، هم شکننده: هر تغییرِ کوچکِ فرمت یعنی باید دوباره دستی رشته را بازنویسی کنی. هیچ‌کس این کار را واقعاً انجام نمی‌دهد؛ تست یا حذف می‌شود یا کورکورانه به‌روز.

این درس دو ابزارِ تازه می‌آورد، هرکدام برایِ یکی از این دو مشکل — **اضافه به** ۲.۷.۲، نه جایگزینش:

- **proptest**: به‌جایِ چند ورودیِ دستی، یک خاصیت را توصیف می‌کنی — قانونی که باید برایِ هر ورودی درست باشد — و کتابخانه صدها ورودی، شاملِ لبه‌های عجیب، تولید می‌کند و سعی می‌کند بشکندش.
- **insta**: خروجیِ پیچیده را یک‌بار می‌گیری، خودت به‌عنوانِ منبعِ حقیقت بازبینی و تأییدش می‌کنی، و از آن به بعد هر اجرا با همان عکسِ ثبت‌شده مقایسه می‌شود — با صدایِ بلند رد می‌شود اگر چیزی بدونِ بازبینی عوض شده باشد.

هیچ‌کدام‌شان جایِ تستِ ساده را نمی‌گیرد. برایِ اکثرِ کد، یک `#[test]` با یک `assert_eq!` دقیق دقیقاً همان چیزی است که لازم داری — نوشتنش سریع‌تر است، خواندنش هم. این دو، برایِ همان دو حالتِ خاص‌اند: یک ناوردایِ واقعی که رویِ فضایِ وسیعی از ورودی باید برقرار بماند، و خروجی‌ای که درست است ولی برایِ assertِ دستی خیلی بزرگ یا پیچیده است.

---

## مفهوم

### یک قرارداد که باید برایِ هر ورودی برقرار بماند

این نوع را در نظر بگیر — یک مختصات که می‌شود به رشته تبدیلش کرد و از رشته دوباره ساختش:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn format(&self) -> String {
        format!("{},{}", self.x, self.y)
    }
}
```

```rust
impl Point {
    fn parse(s: &str) -> Option<Point> {
        let (x_str, y_str) = s.split_once(',')?;
        let x = x_str.parse().ok()?;
        let y = y_str.parse().ok()?;
        Some(Point { x, y })
    }
}
```

حالا دقیقاً همان‌طور که ۲.۷.۲ یادت داد، یک تست بنویس:

```rust
#[test]
fn round_trips_a_hand_picked_point() {
    let p = Point { x: 3, y: 4 };
    assert_eq!(Point::parse(&p.format()), Some(p));
}
```

```text
running 1 test
test tests::round_trips_a_hand_picked_point ... ok
```

سبز است. ولی دقیقاً چه چیزی را ثابت کرده؟ فقط اینکه رفت‌وبرگشت برایِ `(3, 4)` درست کار می‌کند. میلیاردها جفتِ دیگرِ ممکنِ `(x, y)` — منفی‌ها، صفر، `i32::MIN`، هرچیزِ دیگر — هیچ‌وقت امتحان نشدند.

قراردادِ واقعی‌ای که می‌خواهی این است: **برایِ هر `x` و هر `y`، `parse(format(p))` باید دقیقاً همان `p` را برگرداند.** این یک **خاصیت (property)** است — نه یک مثال، بلکه یک قانون رویِ کلِ فضایِ ورودی. proptest دقیقاً همین را می‌گیرد و امتحان می‌کند:

```rust
proptest! {
    #[test]
    fn round_trip(x in any::<i32>(), y in any::<i32>()) {
        let original = Point { x, y };
        let parsed = Point::parse(&original.format());
        prop_assert_eq!(parsed, Some(original));
    }
}
```

```sh
cargo test -p p2-07-04-property-and-snapshot-testing --example 01-point-round-trip
```

```text
running 2 tests
test tests::round_trips_a_hand_picked_point ... ok
test tests::round_trip ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

هر دو تست سبزند، ولی کارِ متفاوتی کرده‌اند. `round_trip` یک تابع است که `x` و `y` را به‌عنوانِ پارامتر می‌گیرد؛ `any::<i32>()` یک **راهبرد (strategy)** است — توصیفِ اینکه proptest از کجا مقدار بسازد، اینجا «هر `i32`ِ ممکن». پیش‌فرض proptest این تابع را ۲۵۶ بار صدا می‌زند، هر بار با یک `x`/`y`ِ تصادفیِ تازه — نه یک بار، مثلِ تستِ دستی. `prop_assert_eq!` هم دقیقاً مثلِ `assert_eq!` است، فقط داخلِ `proptest!` جوری کار می‌کند که یک شکست را به‌جایِ فقط پنیک‌کردن، به موتورِ کوچک‌سازیِ proptest هم گزارش بدهد — همان چیزی که بخشِ بعد می‌بینی.

### وقتی قرارداد واقعاً می‌شکند: کوچک‌سازی (shrinking)

بگذار `format` یک باگِ واقعی داشته باشد:

```rust
fn format(&self) -> String {
    // BUG: unsigned_abs() throws away the sign of `x`.
    format!("{},{}", self.x.unsigned_abs(), self.y)
}
```

```sh
cargo run -p p2-07-04-property-and-snapshot-testing --example 03-broken-sign-drop --features broken
```

```text
Point { x: -1, y: 0 } -> "1,0" -> Some(Point { x: 1, y: 0 }) — the sign of x is just gone
```

حالا همان `proptest!` را رویِ همین نسخه اجرا کن:

```sh
cargo test -p p2-07-04-property-and-snapshot-testing --example 03-broken-sign-drop --features broken
```

```text
running 1 test
test tests::round_trip ... FAILED

failures:

---- tests::round_trip stdout ----

thread 'tests::round_trip' (28720) panicked at phase2-intermediate\07-project-structure-and-testing\04-property-and-snapshot-testing\examples\03-broken-sign-drop.rs:38:5:
Test failed: assertion failed: `(left == right)` 
  left: `Some(Point { x: 1, y: 0 })`,
 right: `Some(Point { x: -1, y: 0 })` at phase2-intermediate\07-project-structure-and-testing\04-property-and-snapshot-testing\examples\03-broken-sign-drop.rs:47.
minimal failing input: x = -1, y = 0
	successes: 0
	local rejects: 0
	global rejects: 0

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    tests::round_trip

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

proptest تقریباً ۵۰٪ از فضایِ `i32` (هر عددِ منفی) را شکست می‌داد — پس تقریباً بلافاصله یک موردِ شکست‌خورده پیدا کرد؛ عددِ دقیقِ `successes` بالا هر بار که خودت اجرایش کنی فرق می‌کند، چون جستجو تصادفی است. کارِ جالب بعدش شروع می‌شود: proptest همان ورودیِ اولیه (شاید یک عددِ بزرگ و بی‌ربط مثلِ `-1992831647`) را نگه نمی‌دارد. آن را دوباره و دوباره **کوچک‌تر** می‌کند — نصف می‌کند، به سمتِ صفر می‌برد — و هر بار دوباره امتحان می‌کند: «آیا این‌یکیِ کوچک‌تر هم هنوز خاصیت را می‌شکند؟» تا وقتی دیگر نتواند کوچک‌ترش کند. نتیجه‌اش همیشه همین است: `x = -1, y = 0`. نه چون این عدد اولین موردی بود که امتحان شد، بلکه چون این **کوچک‌ترین** موردی است که هنوز باگ را نشان می‌دهد.

```senpai-visual
{"kind":"concept","labels":["تولید ورودی‌های تصادفیِ زیاد","یک ورودی شکست می‌خورد","کوچک‌سازی به‌سمتِ ساده‌ترین حالت","باز هم شکست: دوباره کوچک کن","کوچک‌ترین موردِ شکست گزارش می‌شود"]}
```

این دقیقاً همان چیزی است که proptest را از «فقط یک ورودیِ تصادفی امتحان کن» جدا می‌کند: هیچ‌وقت با یک شکستِ بزرگ و شلوغ تنهایت نمی‌گذارد. همیشه کوچک‌ترین، خوانا‌ترین موردی که همان باگ را نشان می‌دهد به‌ات می‌دهد — دقیقاً چیزی که برایِ رفعِ باگ لازم داری.

(proptest به‌طورِ پیش‌فرض این ورودیِ شکست‌خورده را در یک فایلِ کنارِ سورس هم ذخیره می‌کند تا دفعه‌ی بعد همان مورد را اول امتحان کند — یک ویژگیِ واقعی و مفید. مثالِ بالا این ویژگی را عمداً خاموش کرده، چون این فایل قرار است هر بار که این مثالِ درسی را اجرا می‌کنی از نو ساخته شود، نه اینکه در مخزنِ درس ذخیره‌اش کنی.)

`any::<i32>()` تنها راهبرد نیست. proptest برایِ مجموعه‌ها هم راهبرد دارد (`prop::collection::vec(any::<bool>(), 0..16)` یک `Vec<bool>` با طولِ بینِ ۰ تا ۱۵ می‌سازد)، برایِ تاپل‌ها، و برایِ نوعِ خودت با `.prop_map(...)`. یکی از این‌ها را در «تمرین» می‌بینی، پشتِ یک تستِ آماده — لازم نیست خودت بنویسی‌اش، فقط بدان وجود دارد.

### خروجی‌ای که نوشتنِ دستیِ assert برایش معقول نیست

مسئله‌ی دوم را ببین. این تابع یک گزارشِ چندخطی می‌سازد:

```rust
fn watch_digest(entries: &[Entry]) -> String {
    let mut out = String::new();
    for e in entries {
        let rating = match e.rating {
            Some(r) => format!("{r}/10"),
            None => "unrated".to_string(),
        };
        out.push_str(&format!(
            "{} - {} episodes - {}\n",
            e.title, e.episodes_watched, rating
        ));
    }
    out
}
```

می‌شد نتیجه‌اش را دستی توی یک `assert_eq!` نوشت، ولی سه‌خطی نیست — واقعیت این است که گزارش‌های واقعی ده‌ها خط دارند، و هر بار که یک فیلدِ تازه اضافه می‌کنی باید دوباره دستی رشته را بازنویسی کنی. insta این کار را برایت می‌کند: خروجی را یک‌بار می‌گیرد، خودت به‌عنوانِ منبعِ حقیقت تأییدش می‌کنی، و insta از آن به بعد فقط دیف می‌گیرد:

```rust
#[test]
fn digest_snapshot() {
    insta::assert_snapshot!(watch_digest(&sample_entries()));
}
```

اولین باری که این تست اجرا شود، هیچ عکسی برایِ مقایسه وجود ندارد — insta شکست می‌خورد، عمداً، و یک فایلِ «در انتظار» می‌سازد:

```text
running 1 test
stored new snapshot M:\SenPai-Rust-Journey\phase2-intermediate\07-project-structure-and-testing\04-property-and-snapshot-testing\examples\snapshots\02_watch_digest_snapshot__tests__digest_snapshot.snap.new
test tests::digest_snapshot ... FAILED

failures:

---- tests::digest_snapshot stdout ----
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ Snapshot Summary ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Snapshot file: phase2-intermediate\07-project-structure-and-testing\04-property-and-snapshot-testing\examples\snapshots\02_watch_digest_snapshot__tests__digest_snapshot.snap
Snapshot: digest_snapshot
Source: M:\SenPai-Rust-Journey:55
───────────────────────────────────────────────────────────────────────────────
Expression: watch_digest(&sample_entries())
───────────────────────────────────────────────────────────────────────────────
+new results
────────────┬──────────────────────────────────────────────────────────────────
          1 │+Frieren - 12 episodes - 9/10
          2 │+Bocchi the Rock! - 12 episodes - 10/10
          3 │+Made in Abyss - 3 episodes - unrated
────────────┴──────────────────────────────────────────────────────────────────
To update snapshots run `cargo insta review`
Stopped on the first failure. Run `cargo insta test` to run all snapshots.

thread 'tests::digest_snapshot' (14796) panicked at C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\insta-1.48.0\src\runtime.rs:719:13:
snapshot assertion for 'digest_snapshot' failed in line 55
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    tests::digest_snapshot

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
```

(مسیرِ کاملِ کنارِ `Source:` رویِ کامپیوترِ خودت شکلِ دیگری دارد — همیشه همان‌جایی است که این فرمان را اجرا کرده‌ای.) دقت کن: این خطایِ کامپایلر نیست، یک شکستِ *عمدیِ* زمانِ تست است. insta محتوایِ واقعی را در یک فایلِ کنارش نوشته، با پسوندِ `.snap.new`:

```text
---
source: phase2-intermediate/07-project-structure-and-testing/04-property-and-snapshot-testing/examples/02-watch-digest-snapshot.rs
assertion_line: 55
expression: watch_digest(&sample_entries())
---
Frieren - 12 episodes - 9/10
Bocchi the Rock! - 12 episodes - 10/10
Made in Abyss - 3 episodes - unrated
```

این دقیقاً همان چیزی است که «capture کن، بعد بازبینی کن» یعنی: insta هیچ‌وقت خودش تصمیم نمی‌گیرد این خروجی درست است یا نه. فقط می‌گوید «این چیزی است که تولید شد؛ تو نگاهش کن.»

### بازبینی، تأیید، رد: جریانِ کاریِ insta

بعدِ نگاه‌کردن به دیف بالا، خودت (نویسنده) تصمیم می‌گیری این خروجی درست است. دو راه برایِ **تأیید**کردنش هست: دستی فایلِ `.snap.new` را با حذفِ پسوندش به `.snap` تبدیل کنی، یا فرمانِ رسمی را بزنی — که اگر `cargo-insta` نصب باشد (`cargo install cargo-insta`) کارِ همین تغییرِ نام را برایت می‌کند و فایلِ `assertion_line` را هم پاک می‌کند، چون شماره‌خط‌ها جابه‌جا می‌شوند ولی خودِ متن نه:

```sh
cargo insta accept
```

```text
insta review finished
accepted:
  phase2-intermediate/07-project-structure-and-testing/04-property-and-snapshot-testing/examples/02-watch-digest-snapshot.rs (digest_snapshot.snap)
```

از این لحظه، `.snap` همان منبعِ حقیقت است. هر اجرایِ بعدیِ `cargo test` فقط یک مقایسه‌ی رشته‌ای می‌کند، بدونِ اینکه دوباره پنیک بدهد — تا وقتی واقعاً چیزی عوض شود:

```senpai-visual
{"kind":"concept","labels":["اجرای اول: هنوز عکسی نیست","insta شکست می‌خورد، یک عکسِ در انتظار می‌نویسد","خودت دیف را با دست بازبینی می‌کنی","تأیید: همان می‌شود حقیقت","رد: عکسِ در انتظار دور ریخته می‌شود"]}
```

حالا فرض کن یک هم‌تیمی، بدونِ اطلاع، متنِ گزارش را کمی عوض کرده:

```rust
// Someone shortened "episodes" to "eps" here without telling anyone.
// The committed snapshot still says "episodes".
out.push_str(&format!(
    "{} - {} eps - {}\n",
    e.title, e.episodes_watched, rating
));
```

```sh
cargo test -p p2-07-04-property-and-snapshot-testing --example 04-broken-stale-snapshot --features broken
```

این وسطِ خروجی است — خودِ دیف. ترنسکریپتِ کاملش، با پیامِ پنیک و همه‌چیزِ اطرافش، در «خطاهایی که خواهی دید» است:

```text
-old snapshot
+new results
────────────┬──────────────────────────────────────────────────────────────────
    1       │-Frieren - 12 episodes - 9/10
    2       │-Bocchi the Rock! - 12 episodes - 10/10
    3       │-Made in Abyss - 3 episodes - unrated
          1 │+Frieren - 12 eps - 9/10
          2 │+Bocchi the Rock! - 12 eps - 10/10
          3 │+Made in Abyss - 3 eps - unrated
────────────┴──────────────────────────────────────────────────────────────────
To update snapshots run `cargo insta review`
```

insta هیچ‌وقت نمی‌داند این تغییر عمدی بوده یا یک باگ — فقط می‌داند خروجی با آخرین چیزی که تأیید کرده‌ای فرق دارد، و به همینِ خاطر با صدایِ بلند رد می‌شود. تصمیم دستِ توست: اگر «eps» واقعاً تغییرِ دلخواهت بود، `cargo insta accept` می‌زنی و `.snap` تازه می‌شود؛ اگر یک اشتباه بود، `cargo insta reject` می‌زنی (یا همان فایلِ `.snap.new` را پاک می‌کنی) و کد را برمی‌گردانی. همین سازوکار، بدونِ هیچ فرقی، همان چیزی است که یک regressionِ واقعی را هم می‌گیرد — insta بینِ «قصدی» و «باگ» فرق نمی‌گذارد، فقط تفاوت را نشانت می‌دهد.

### کِی هرکدام، و کِی هیچ‌کدام

قاعده‌ی عملی کوتاه است:

- یک **ناوردایِ واقعی** داری که باید رویِ فضایِ وسیعی از ورودی برقرار بماند (رفت‌وبرگشت، ترتیب، مجموع، یک محدوده) → proptest.
- خروجی **درست است ولی بزرگ یا پیچیده** است — یک گزارش، یک ساختارِ تودرتو، خروجیِ Debugِ چیزی — و نوشتنِ دستیِ assert برایش بیشتر وقت تلف می‌کند تا کمک → insta.
- بقیه‌ی موارد — اکثرِ کد — همان `#[test]` با یک `assert_eq!` که ۲.۷.۲ یادت داد. این دو ابزارِ تازه اضافه‌شدند، جایگزینِ چیزی نشدند.

انتخابِ ابزارِ اشتباه هم واقعاً هزینه دارد: یک proptest رویِ یک تابعِ بدونِ هیچ ناوردایِ معنادار فقط کندت می‌کند بدونِ اینکه چیزیِ تازه‌ای پیدا کند؛ یک عکس از یک خروجیِ سه‌کلمه‌ای فقط یک لایه‌ی غیرِضروری بینِ تو و یک `assert_eq!`ِ ساده می‌گذارد.

---

## دست‌به‌کد

```sh
cargo run -p p2-07-04-property-and-snapshot-testing --example 01-point-round-trip
cargo test -p p2-07-04-property-and-snapshot-testing --example 01-point-round-trip
cargo run -p p2-07-04-property-and-snapshot-testing --example 02-watch-digest-snapshot
cargo test -p p2-07-04-property-and-snapshot-testing --example 02-watch-digest-snapshot
```

بعد دوتای خراب:

```sh
cargo test -p p2-07-04-property-and-snapshot-testing --example 03-broken-sign-drop --features broken
cargo test -p p2-07-04-property-and-snapshot-testing --example 04-broken-stale-snapshot --features broken
```

بعد این‌ها را امتحان کن:

۱. در `02-watch-digest-snapshot`، فایلِ `examples/snapshots/02_watch_digest_snapshot__tests__digest_snapshot.snap` را حذف کن و دوباره `cargo test --example 02-watch-digest-snapshot` را بزن. همان شکستِ «اولین‌بار» بالا را می‌بینی؟ حالا خودت `.snap.new` را با حذفِ پسوندش تأیید کن و دوباره اجرا کن.
۲. در `01-point-round-trip`، یک خطِ سوم به `proptest!` اضافه کن: `#![proptest_config(ProptestConfig { cases: 2000, ..ProptestConfig::default() })]` (بالایِ خودِ `fn round_trip`). زمانِ اجرا چقدر عوض می‌شود؟
۳. در `03-broken-sign-drop`، همین دستورِ تست را دو یا سه بار پشتِ سرِ هم اجرا کن. کدام بخشِ خروجی هر بار عوض می‌شود و کدام همیشه یکی می‌ماند؟ چرا؟

---

## خطاهایی که خواهی دید

### یک شکستِ proptest — کوچک‌ترین موردِ نقض‌کننده

```text
running 1 test
test tests::round_trip ... FAILED

failures:

---- tests::round_trip stdout ----

thread 'tests::round_trip' (28720) panicked at phase2-intermediate\07-project-structure-and-testing\04-property-and-snapshot-testing\examples\03-broken-sign-drop.rs:38:5:
Test failed: assertion failed: `(left == right)` 
  left: `Some(Point { x: 1, y: 0 })`,
 right: `Some(Point { x: -1, y: 0 })` at phase2-intermediate\07-project-structure-and-testing\04-property-and-snapshot-testing\examples\03-broken-sign-drop.rs:47.
minimal failing input: x = -1, y = 0
	successes: 0
	local rejects: 0
	global rejects: 0

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    tests::round_trip

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**کامپایلر به چه اعتراض دارد:** این حتی خطایِ کامپایلر نیست — برنامه کامپایل شد، proptest صدها `x`/`y` امتحان کرد، و روی این‌یکی (بعدِ کوچک‌سازی: `x = -1, y = 0`) `prop_assert_eq!` شکست خورد. `left` چیزی است که کد واقعاً برگرداند؛ `right` چیزی است که انتظار داشتیم.

**راه‌حل:** `.unsigned_abs()` را بردار — علامتِ `x` باید توی فرمت بماند:

```rust
fn format(&self) -> String {
    format!("{},{}", self.x, self.y)
}
```

**چرا این راه‌حل است:** با این نسخه، `format` هیچ اطلاعاتی گم نمی‌کند — هر `Point` دقیقاً یک نمایشِ رشته‌ای دارد و `parse` دقیقاً همان را برمی‌گرداند، برایِ هر `x` و `y`، نه فقط برایِ آن‌هایی که مثبت‌اند.

### یک پنیکِ زمانِ اجرا — عدم‌تطابقِ عکسِ insta

```text
running 1 test
stored new snapshot M:\SenPai-Rust-Journey\phase2-intermediate\07-project-structure-and-testing\04-property-and-snapshot-testing\examples\snapshots\04_broken_stale_snapshot__tests__digest_snapshot.snap.new
test tests::digest_snapshot ... FAILED

failures:

---- tests::digest_snapshot stdout ----
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ Snapshot Summary ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Snapshot file: phase2-intermediate\07-project-structure-and-testing\04-property-and-snapshot-testing\examples\snapshots\04_broken_stale_snapshot__tests__digest_snapshot.snap
Snapshot: digest_snapshot
Source: M:\SenPai-Rust-Journey:60
───────────────────────────────────────────────────────────────────────────────
Expression: watch_digest(&sample_entries())
───────────────────────────────────────────────────────────────────────────────
-old snapshot
+new results
────────────┬──────────────────────────────────────────────────────────────────
    1       │-Frieren - 12 episodes - 9/10
    2       │-Bocchi the Rock! - 12 episodes - 10/10
    3       │-Made in Abyss - 3 episodes - unrated
          1 │+Frieren - 12 eps - 9/10
          2 │+Bocchi the Rock! - 12 eps - 10/10
          3 │+Made in Abyss - 3 eps - unrated
────────────┴──────────────────────────────────────────────────────────────────
To update snapshots run `cargo insta review`
Stopped on the first failure. Run `cargo insta test` to run all snapshots.

thread 'tests::digest_snapshot' (20648) panicked at C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\insta-1.48.0\src\runtime.rs:719:13:
snapshot assertion for 'digest_snapshot' failed in line 60
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    tests::digest_snapshot

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
```

**کامپایلر به چه اعتراض دارد:** باز هم خطایِ کامپایلر نیست. `watch_digest` واقعاً اجرا شد و یک خروجیِ کاملاً معتبر داد؛ فقط با آن `.snap`ی که قبلاً تأیید کرده بودی یکی نیست. insta این را دقیقاً همان‌طور که یک `assert_eq!` عمل می‌کند گزارش می‌دهد — رد می‌شود، نه سکوت.

**راه‌حل:** طبقِ مشخصاتِ محصول، «eps» یک اشتباه بوده؛ کد را برگردان تا با `.snap`ِ موجود یکی شود:

```rust
out.push_str(&format!(
    "{} - {} episodes - {}\n",
    e.title, e.episodes_watched, rating
));
```

**چرا این راه‌حل است:** اگر «eps» واقعاً تغییرِ دلخواه بود، راه‌حلِ درست این بود که `.snap` را با `cargo insta accept` تازه کنی، نه کد را برگردانی. اینجا برعکس است: کد اشتباه رفته، و `.snap`ِ تأییدشده دقیقاً همان چیزی است که باید بماند — این تمامِ فایده‌ی داشتنِ یک منبعِ حقیقتِ ثبت‌شده است.

---

## تمرین

### گرم‌کردن

<details>
<summary>در ترنسکریپتِ بالا نوشته «minimal failing input: x = -1, y = 0» و پایین‌ترش «successes: 0». این عدد یعنی چه؟</summary>

یعنی پیش از رسیدن به موردِ شکست‌خورده، هیچ ورودیِ تصادفیِ دیگری با موفقیت رد نشده بود — proptest تقریباً بلافاصله (روی همان ورودی‌هایِ اول) یک موردِ منفی پیدا کرد. اگر دوباره اجرا کنی، این عدد معمولاً فرق می‌کند؛ `minimal failing input` ولی معمولاً همان می‌ماند، چون کوچک‌سازی تعیین‌کننده است، نه جستجویِ اولیه.

</details>

<details>
<summary>درست یا غلط: فایل‌هایِ <code>.snap</code> باید در گیت commit شوند، ولی <code>.snap.new</code> نباید.</summary>

درست. `.snap` منبعِ حقیقتِ تأییدشده است — همان چیزی که هر توسعه‌دهنده‌ی دیگر هم باید داشته باشدش. `.snap.new` فقط یک پیشنهادِ در-انتظارِ بازبینی است، مالِ همان لحظه‌ی رویِ کامپیوترِ توست؛ نباید commit شود.

</details>

<details>
<summary>تابعی داری با پنج تستِ دستیِ سبز از ۲.۷.۲. برایِ همان تابع یک تستِ proptest هم اضافه می‌کنی. آیا باید آن پنج‌تا را حذف کنی؟</summary>

نه. آن پنج‌تا مثال‌هایِ مشخص و خوانا هستند، هرکدام یک چکِ نام‌دار و دائمی رویِ مقدارِ خودش — خروجیِ `cargo test` دقیقاً نشان می‌دهد کدام‌یک پاس شده یا خراب شده. یک اجرایِ سبزِ proptest هیچ‌وقت نشانت نمی‌دهد کدام ۲۵۶ مقدار امتحان شدند؛ فقط یک شکست است که یک ورودیِ مشخص را رو می‌کند. آن پنج تستِ دستی چیزی‌اند که می‌گذارند، با نام، ببینی یک موردِ شناخته‌شده بعدِ یک تغییر هنوز درست کار می‌کند. proptest اضافه می‌شود، جایگزین نمی‌کند.

</details>

<details>
<summary>این کامپایل می‌شود؟ اگر بله، پاس می‌شود یا شکست می‌خورد؟</summary>

```rust
proptest! {
    #[test]
    fn trivial(x in any::<i32>()) {
        prop_assert_eq!(x, x);
    }
}
```

</details>

<details>
<summary>پاسخ</summary>

کامپایل می‌شود و پاس می‌شود — رویِ هر ۲۵۶ موردِ تصادفی. `x == x` برایِ هر `i32` همیشه درست است (بازتابی‌بودن، از ۲.۳.۴). این خودش یک نکته‌ی مهم است: proptest فقط به‌اندازه‌ی خاصیتی که بهش می‌دهی خوب است — یک خاصیتِ بی‌اثر مثلِ این، هرچقدر هم مورد امتحان کند، هیچ باگی پیدا نمی‌کند.

</details>

### تعمیر

هر دو مثالِ خراب را درست کن:

۱. `examples/03-broken-sign-drop.rs` را طوری درست کن که خاصیتِ رفت‌وبرگشت برایِ **هر** `i32` — نه فقط مثبت‌ها — برقرار بماند.
۲. `examples/04-broken-stale-snapshot.rs` را درست کن. طبقِ مشخصاتِ محصول، «eps» یک تغییرِ ناخواسته بوده — کد را طوری برگردان که دوباره با `.snap`ِ موجود یکی شود، بدونِ اینکه خودِ `.snap` را دست بزنی. (اگر برعکسش بود — یعنی «eps» عمدی بود — کدام فرمان را می‌زدی تا `.snap` تازه شود؟)

### پیاده‌سازی

سه تابع/نوع در `src/lib.rs`:

```sh
cargo test -p p2-07-04-property-and-snapshot-testing
```

`encode_flags`/`decode_flags` را طبقِ کامنتِ مستندساز بالایِ هرکدام پیاده کن — تستِ پنهانشان یک خاصیتِ رفت‌وبرگشت است (proptest)، نه یک ورودیِ مشخص. `checklist` را هم طبقِ کامنتش پیاده کن — تستِ پنهانش یک عکسِ از پیش تأییدشده در `src/snapshots/` است؛ فرمتِ خروجی‌ات باید دقیقاً با چیزی که کامنتِ مستندساز گفته یکی باشد.

### بساز

یک تابعِ کوچک از دامنه‌ای که خودت انتخاب می‌کنی بنویس، و تصمیم بگیر کدام ابزار بهش می‌خورد — یک ناوردایِ رفت‌وبرگشتی (proptest) یا یک خروجیِ پیچیده‌تر از یک assertِ دستی (insta). فقط یکی از این دو را انتخاب کن، همان‌طور که این درس گفت هرکدام برایِ چه چیزی است، و یک تستِ واقعی از همان نوع برایش بنویس.

### چالش (اختیاری)

**بخشِ یک.** رویِ تستِ `flags_round_trip` (در `src/lib.rs`)، طولِ بیشینه‌ی `Vec<bool>` را از `0..16` به `0..500` عوض کن، و به `ProptestConfig`ای که همان‌جا از قبل هست یک فیلدِ `cases: 5000` هم اضافه کن (کنارِ `failure_persistence: None`ای که از قبل داشت). زمانِ اجرا را قبل و بعد اندازه بگیر. چیزی می‌شکند؟

**بخشِ دو.** (این یکی جلوتر را نگاه می‌کند.) proptest به‌دنبالِ کوچک‌ترین ورودی‌ای می‌گردد که *جوابِ غلط* می‌دهد. یک سؤالِ کاملاً متفاوت این است: وقتی یک ورودیِ نماینده را انتخاب کردی، کدِ تو رویش چقدر *سریع* اجرا می‌شود — سنجیده‌شده به‌طورِ دقیق، نه فقط با یک کرونومتر؟ آن سؤالِ [۲.۷.۵ — سنجشِ کارایی با `criterion`](../05-benchmarking-with-criterion/README.fa.md) است.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| خاصیت (property) | قانونی که باید برایِ هر ورودی، نه فقط چند مثال، برقرار بماند | یک ناوردایِ واقعی — رفت‌وبرگشت، ترتیب، مجموع |
| `proptest!` | ماکرویی که یک تابعِ پارامتری را به یک تست تبدیل می‌کند | نوشتنِ خودِ خاصیت |
| راهبرد (strategy) | توصیفِ اینکه proptest از کجا مقدار بسازد، مثلِ `any::<i32>()` | تعیینِ فضایِ ورودی‌هایِ تستی |
| کوچک‌سازی (shrinking) | کاهشِ خودکارِ یک ورودیِ شکست‌خورده به کوچک‌ترین حالتِ هنوز-شکست‌خورده | خواندنِ آسان‌ترِ یک شکست |
| عکس (snapshot) | خروجیِ ثبت‌شده‌ای که هر اجرایِ بعدی با آن مقایسه می‌شود | خروجیِ درست ولی خیلی بزرگ/پیچیده برایِ assertِ دستی |
| `.snap` / `.snap.new` | عکسِ تأییدشده / عکسِ در انتظارِ بازبینی | اولی commit می‌شود، دومی نه |
| `cargo insta accept` / `reject` | تأیید یا ردِ یک عکسِ در انتظار | جایگزینِ تغییرِ نامِ دستیِ فایل |

### الان می‌دانی

- یک خاصیت قانونی است که باید برایِ *هر* ورودی برقرار بماند، نه فقط چند مثالی که خودت انتخاب کرده‌ای؛ `proptest!` همان تابعِ تست را صدها بار، با ورودی‌هایِ تصادفیِ تازه، اجرا می‌کند.
- وقتی proptest یک شکست پیدا می‌کند، آن را به کوچک‌ترین حالتِ هنوز-شکست‌خورده کوچک می‌کند — تو همیشه یک موردِ خوانا می‌بینی، نه یک عددِ تصادفیِ شلوغ.
- insta یک خروجیِ پیچیده را یک‌بار می‌گیرد و به‌عنوانِ منبعِ حقیقت (`.snap`) ذخیره‌اش می‌کند؛ هر اجرایِ بعدی با آن دیف می‌گیرد.
- عکسِ در انتظار (`.snap.new`) هیچ‌وقت خودکار جایِ عکسِ تأییدشده را نمی‌گیرد — تو باید تصمیم بگیری: تأیید (`cargo insta accept`) یا رد (`cargo insta reject`).
- هردو ابزار اضافه‌شده‌اند به جعبه‌ابزارِ ۲.۷.۲، نه جایگزینِ آن؛ اکثرِ کد همچنان با یک `#[test]` و یک `assert_eq!` بهتر تست می‌شود.

### بعداً کامل‌تر می‌بینی

- **سنجشِ کارایی، نه فقط درستی** — [۲.۷.۵ — سنجشِ کارایی با `criterion`](../05-benchmarking-with-criterion/README.fa.md)

### می‌توانی توضیح بدهی؟

- فرقِ یک «مثال» و یک «خاصیت» چیست؟ یک خاصیتِ واقعی برایِ تابعی که خودت اخیراً نوشته‌ای مثال بزن.
- proptest وقتی یک شکست پیدا می‌کند چه کار می‌کند که یک تستِ دستی هیچ‌وقت نمی‌کند؟
- چرا اولین‌باری که `insta::assert_snapshot!` را اجرا می‌کنی، تست شکست می‌خورد — با اینکه کد هیچ باگی ندارد؟
- فرقِ `.snap` و `.snap.new` چیست، و کدام‌شان commit می‌شود؟
- یک هم‌تیمی می‌گوید «چرا این تست شکست خورد؟ من که کدِ اصلی را عوض نکردم!» — insta چه چیزی را می‌بیند که او ندیده؟
- برایِ کدام‌یک از این‌ها proptest بهتر است و برایِ کدام‌یک insta: (الف) یک تابعِ جمع‌زدنِ دو عدد، (ب) یک تابعِ رمزگشاییِ Base64، (ج) یک تابعی که یک قطعه‌ی HTML چندخطی می‌سازد؟

---

## بیشتر

- [مستنداتِ proptest](https://docs.rs/proptest/latest/proptest/) — فهرستِ کاملِ راهبردها، شاملِ `prop_oneof!` برایِ enumها و `.prop_map()`/`.prop_flat_map()` برایِ نوع‌هایِ خودت.
- [کتابِ proptest](https://proptest-rs.github.io/proptest/intro.html) — چرا کوچک‌سازی کار می‌کند، و چطور یک راهبردِ سفارشی بنویسی.
- [مستنداتِ insta](https://insta.rs/docs/) — جریانِ کاملِ `cargo insta review`، تنظیماتِ فرمت (YAML/JSON/Debug)، و ادغام با CI.
- [`cargo-fuzz`](https://rust-fuzz.github.io/book/) — ایده‌ای همسایه، نه همین: به‌جایِ چک‌کردنِ یک خاصیتِ مشخص، فقط دنبالِ هر ورودی‌ای می‌گردد که برنامه را پنیک یا کرش بدهد. این دوره پوشش نمی‌دهدش؛ اگر کنجکاو شدی، همین‌جا شروع کن.

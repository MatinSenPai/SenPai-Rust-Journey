# ۲.۷.۵ — سنجشِ کارایی با `criterion`

## در یک نگاه

بعد از این درس می‌توانی:

- توضیح بدهی چرا یک `Instant::now()`/`.elapsed()`ِ تنها، دورِ یک تابع، سنجشِ کارایی نیست — و سه دلیلِ واقعی‌اش را نام ببری: نویزِ اجرا-به-اجرا، مقیاس‌گذاریِ فرکانسِ CPU، و یک نمونه‌ی تنها که هیچ آماری پشتش نیست.
- یک بنچمارکِ واقعی با `criterion` بسازی (`criterion_group!`/`criterion_main!`)، `cargo bench` را اجرا کنی، و میانگین+بازه‌ی اطمینانِ خروجی‌اش را درست بخوانی.
- بگویی `std::hint::black_box` دقیقاً جلویِ چه چیزی را می‌گیرد، و چرا هر بنچمارکِ `criterion` بهش نیاز دارد.
- برایِ یک تکه‌کدِ واقعی تشخیص بدهی کِی بنچمارک‌گرفتن واقعاً ارزشش را دارد — و چرا «همین الان که نگرانم» جوابِ درستی نیست.

**زمان:** حدود ۵۵ دقیقه · **پیش‌نیاز:** [۲.۷.۴ — تستِ خاصیت‌محور با `proptest`، تستِ عکس‌برداری با `insta`](../04-property-and-snapshot-testing/README.fa.md)

---

## چرا اهمیت دارد

هر درسِ این ماژول، تا همین‌جا، به یک سؤال جواب داده: «این کد درست است؟» — همان چیزی که `cargo test` کاملاً دورِ آن ساخته شده. این درس به سؤالِ دیگری جواب می‌دهد: «این کد سریع است؟ و تغییری که دادم واقعاً سریع‌ترش کرد یا فقط حسم این بود؟» — سؤالی که تست‌ها اصلاً بهش جواب نمی‌دهند، و سؤالی که یک تلاشِ ساده‌لوحانه برایِ جوابش می‌تواند گمراهت کند، نه فقط بی‌فایده باشد.

اگر پایتون کار کرده باشی، شکلِ این مشکل را از قبل می‌شناسی. `timeit` یک قطعه‌کد را یک‌بار زمان نمی‌گیرد؛ بارها اجرایش می‌کند، چون یک `time.time()`ِ تنها آن‌قدر نویزدار است که نشود بهش اعتماد کرد. `criterion` دقیقاً همان مشکل را حل می‌کند، فقط سخت‌گیرانه‌تر: نه یک میانگینِ خام، بلکه توزیعِ کامل — میانگین، بازه‌ی اطمینان، و کدام نمونه‌ها آماری «پرت» بودند و نباید زیادی روی‌شان حساب کرد. تشبیه همین‌جا هم می‌شکند: `timeit` را معمولاً دستی و یک‌بار اجرا می‌کنی؛ `criterion` بخشی از گردشِ کارِ توسعه می‌شود — نتیجه‌اش را ذخیره می‌کند و اجرایِ بعدی را با اجرایِ قبلی مقایسه می‌کند.

و این آخرین قطعه از پازلِ همین ماژول است. [۲.۷.۱](../01-modules-visibility-workspaces/README.fa.md) به کد ساختار داد؛ [۲.۷.۲](../02-unit-integration-doc-tests/README.fa.md) سه‌جور تست به‌ات داد که درستیِ آن ساختار را اثبات کنند؛ [۲.۷.۳](../03-test-doubles-in-rust/README.fa.md) بدل‌هایی داد که وابستگی‌های قابل-جایگزین را تست‌پذیر کنند؛ [۲.۷.۴](../04-property-and-snapshot-testing/README.fa.md) وقتی ورودی خیلی زیاد بود یا خروجی خیلی پیچیده، راهِ تستش را نشان داد. همه‌شان یک سؤال را جواب می‌دهند: «این درست کار می‌کند؟». امروز، برایِ اولین‌بار، سؤال عوض می‌شود.

---

## مفهوم

### چرا یک استاپ‌واچِ دورِ یک اجرا گولت می‌زند

این تابع را داریم — جست‌وجویِ خطی، یکی‌یکی، تا آخرِ لیست:

```rust
pub fn contains_linear(haystack: &[u32], needle: u32) -> bool {
    haystack.iter().any(|&item| item == needle)
}
```

اولین غریزه این است: یک `Instant` بگذار دورش، ببین چقدر طول می‌کشد.

```rust
let start = Instant::now();
let found = contains_linear(&haystack, 999_999);
println!("run 1: found={found}  elapsed={:?}", start.elapsed());
```

همین را سه بار پشتِ سرِ هم، در یک اجرا، صدا بزن — دقیقاً همان چیزی که `examples/01-naive-timing.rs` می‌کند. این خروجیِ واقعی‌اش، روی همین ماشین، در دو اجرایِ جداگانه‌ی برنامه:

```text
run 1: found=false  elapsed=2.3µs
run 2: found=false  elapsed=300ns
run 3: found=false  elapsed=600ns
```

```text
run 1: found=false  elapsed=1.8µs
run 2: found=false  elapsed=300ns
run 3: found=false  elapsed=300ns
```

روی ماشینِ خودت این اعداد فرق خواهند داشت — نکته دقیقاً همین است. سه چیز این‌جا در کار است:

- **بدونِ گرم‌کردن.** `run 1` هر بار چند برابرِ `run 2`/`run 3` طول کشیده، با اینکه دقیقاً همان تابع را روی همان ورودی صدا زده. اولین فراخوانی هزینه‌ای می‌دهد که فراخوانی‌هایِ بعدی نمی‌دهند — کشِ سرد، پیش‌بینِ شاخه‌یِ سرد. یک اندازه‌گیریِ تنها، بدونِ گرم‌کردن، این هزینه را قاطیِ نتیجه می‌کند، با اینکه ربطی به رفتارِ پایدارِ تابع در یک برنامه‌ی واقعی ندارد.
- **نویزِ اجرا-به-اجرا.** حتی `run 3` بینِ دو اجرا فرق کرد: ۶۰۰ نانوثانیه در برابرِ ۳۰۰. زمان‌بندِ سیستم‌عامل، هرچه دیگر روی ماشین در حالِ اجراست، حتی چیدمانِ حافظه — همه رویِ عددی که می‌بینی اثر می‌گذارند، و هیچ‌کدام به کدت مربوط نیستند.
- **مقیاس‌گذاریِ فرکانسِ CPU.** CPUهای امروزی، بسته به دما و مصرفِ برق، فرکانسِ ساعت‌شان را لحظه‌به‌لحظه بالا و پایین می‌برند (Turbo Boost و مشابهش). همان کد، در یک لحظه که پردازنده «بوست» شده، از لحظه‌ای که نشده، تندتر اجرا می‌شود — بدونِ اینکه یک بایت از کد عوض شده باشد.

با یک نمونه‌ی تنها، نمی‌توانی این سه را از هم جدا کنی. `run 1` عددِ واقعی‌ای بود، ولی عددِ *اشتباه* برایِ سؤالی که واقعاً می‌پرسی: «این تابع، در حالتِ پایدار، معمولاً چقدر طول می‌کشد؟»

### `criterion`: چه فرقی واقعاً می‌کند

`criterion` یک تابع را صدها یا هزاران بار صدا می‌زند، اول چند ثانیه صرفِ گرم‌کردن می‌کند و آن نمونه‌ها را دور می‌ریزد، بعد نمونه‌های واقعی را جمع می‌کند و رویِشان آمار حساب می‌کند — نه یک عدد، یک **میانگین با بازه‌ی اطمینان**، به‌علاوه‌ی علامت‌گذاریِ نمونه‌هایی که آماری «پرت» بودند.

این تکه از `benches/comparison.rs` — فایلی که همین الان کامل و قابلِ‌اجراست، تمرین نیست:

```rust
fn lookup_benchmark(c: &mut Criterion) {
    let haystack: Vec<u32> = (0..1_000).collect();
    let set: HashSet<u32> = haystack.iter().copied().collect();
    let needle = 999_999;

    c.bench_function("contains_linear_1000", |b| {
        b.iter(|| contains_linear(black_box(&haystack), black_box(needle)))
    });
    c.bench_function("contains_hashset_1000", |b| {
        b.iter(|| contains_hashset(black_box(&set), black_box(needle)))
    });
}
```

`c.bench_function(name, |b| b.iter(|| ...))` قلبِ ماجراست: `name` در خروجی و گزارش ظاهر می‌شود؛ کلوژرِ داخلِ `b.iter` همان چیزی است که بارها اجرا و زمان‌گیری می‌شود. `criterion_group!`/`criterion_main!` (پایینِ همان فایل) `fn main` را خودشان می‌سازند — هیچ‌وقت دستی یکی نمی‌نویسی. و `Cargo.toml` باید بگوید `harness = false`، چون `criterion` هارنسِ خودش را می‌آورد، نه هارنسِ توکارِ نامدارِ Rust را.

اجرا کن — `cargo bench -p p2-07-05-benchmarking-with-criterion` — و این خروجیِ واقعی‌اش را می‌بینی:

```text
Gnuplot not found, using plotters backend
Benchmarking contains_linear_1000
Benchmarking contains_linear_1000: Warming up for 3.0000 s
Benchmarking contains_linear_1000: Collecting 100 samples in estimated 5.0010 s (21M iterations)
Benchmarking contains_linear_1000: Analyzing
contains_linear_1000    time:   [240.95 ns 242.13 ns 243.40 ns]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
```

```text
Benchmarking contains_hashset_1000
Benchmarking contains_hashset_1000: Warming up for 3.0000 s
Benchmarking contains_hashset_1000: Collecting 100 samples in estimated 5.0000 s (746M iterations)
Benchmarking contains_hashset_1000: Analyzing
contains_hashset_1000   time:   [6.3456 ns 6.4187 ns 6.5034 ns]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
```

رویِ ماشینِ خودت این اعداد فرق می‌کنند — ولی شکل‌شان فرق نمی‌کند. سه عددِ داخلِ `[...]` مرزِ پایینِ بازه‌ی اطمینان، میانگین، و مرزِ بالا هستند؛ وقتی می‌خواهی یک عدد نقل کنی، همان وسطی را نقل کن. `contains_linear_1000` این‌جا ۲۴۲.۱۳ نانوثانیه، `contains_hashset_1000` فقط ۶.۴۲ نانوثانیه — تقریباً ۳۸ برابر. این دقیقاً همان چیزی است که [۲.۱.۴](../../01-collections/04-choosing-a-collection/README.fa.md) به‌عنوانِ نظریه بهت گفت: `HashMap`/`HashSet` عضویت را در `O(1)`ِ میانگین جواب می‌دهند، `Vec` باید یکی‌یکی همه را ببیند. امروز آن را با عدد دیدی، نه فقط شنیدی.

و «Found N outliers» را جدی بگیر: `criterion` این نمونه‌ها را قایم نمی‌کند یا بی‌سروصدا دور نمی‌ریزد — علامت‌شان می‌زند، تا بدانی چند تا از صد نمونه بی‌ربط به رفتارِ پایدارِ تابع بودند (یک وقفه‌ی زمان‌بند، یک لحظه‌ی شلوغیِ سیستم). این دقیقاً همان چیزی است که یک `println!(elapsed)`ِ تنها هیچ‌وقت بهت نمی‌گوید.

### `black_box`: نگه‌داشتنِ بهینه‌سازِ کامپایلر رویِ راستی

کامپایلرهایِ بهینه‌ساز حق دارند محاسبه‌ای را حذف کنند که نتیجه‌اش هیچ‌جا استفاده نمی‌شود، یا ورودی‌اش را ثابت ببینند و جوابش را سرِ کامپایل از پیش حساب کنند. هر دو، در کدِ واقعی، خوب‌اند — سریع‌ترت می‌کنند. ولی داخلِ یک بنچمارک، یعنی داری زمانِ *هیچ‌کاری* را اندازه می‌گیری، نه زمانِ کاری که فکر می‌کنی داری اندازه می‌گیری.

این تابعِ ساده را دو جور، در یک حلقه‌ی ۱۰۰ میلیون‌باره، اجرا کن — `examples/02-why-black-box.rs`:

```rust
for _ in 0..ITERS {
    total += square(black_box(7));
}
```

```rust
for _ in 0..ITERS {
    total += square(7);
}
```

خروجیِ واقعی‌اش:

```text
with black_box:    48.1169ms  (total=4900000000)
without black_box: 100ns  (total=4900000000)
```

همان محاسبه، همان ۱۰۰ میلیون فراخوانی، همان `total` نهایی — و نسخه‌ی بدونِ `black_box` نزدیک به نیم‌میلیون برابر سریع‌تر «تمام» شد. کامپایلر دیده `square(7)` همیشه همان ۴۹ است، آن را یک‌بار سرِ کامپایل حساب کرده، و حلقه را به «۱۰۰ میلیون بار همین عددِ ثابت را جمع بزن» تبدیل کرده — که خودش هم می‌تواند به یک ضربِ ساده تبدیل شود. `std::hint::black_box(x)` این را رد می‌کند: مقدارِ `x` را از دیدِ بهینه‌ساز پنهان می‌کند، بدونِ اینکه واقعاً چیزی در مقدارش عوض کند، پس کامپایلر مجبور می‌شود فرض کند این مقدار *می‌تواند* هر بار فرق کند — و محاسبه واقعاً هر بار اجرا می‌شود.

```senpai-visual
{"kind":"concept","labels":["ورودی داخل black_box","کامپایلر نمی‌تواند ببیندش","به‌عنوان مقداری ناشناخته","محاسبه‌ی واقعی می‌ماند","سنجش، کارِ واقعی را نشان می‌دهد"]}
```

معنیِ اصلیِ `black_box` را از قبل هم دیده‌ای، هرچند درسش نداده بودیم: [۱.۱.۳](../../../phase1-fundamentals/01-foundations/03-compound-types-and-destructuring/README.fa.md) از همین تابع استفاده کرد تا کامپایلر مقدارِ ثابتِ `۵` را نبیند و ایندکس‌کردنِ خارج‌از-بازه را زودتر از اجرا رد نکند. همان تابع، همان قدرتِ عمومی — «هر چیزی را از دیدِ بهینه‌ساز پنهان کن» — امروز فقط برایِ دلیلِ دیگری به‌کارش می‌بریم: نه جلوگیری از ردِ سرِ کامپایل، بلکه جلوگیری از حذف‌شدنِ کار سرِ اجرا. `black_box` هیچ ربطِ ذاتی‌ای به بنچمارک‌گرفتن ندارد؛ فقط اتفاقاً این‌جا رایج‌ترین جایی است که بهش نیاز داری.

حالا داخلِ خودِ `benches/comparison.rs` همین را ببین، این‌بار روی یک `Criterion::bench_function` واقعی:

```rust
c.bench_function("square_with_black_box", |b| b.iter(|| square(black_box(7))));
c.bench_function("square_without_black_box", |b| b.iter(|| square(7)));
```

```text
Benchmarking square_with_black_box
Benchmarking square_with_black_box: Warming up for 3.0000 s
Benchmarking square_with_black_box: Collecting 100 samples in estimated 5.0000 s (10B iterations)
Benchmarking square_with_black_box: Analyzing
square_with_black_box   time:   [490.91 ps 494.27 ps 498.09 ps]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
```

```text
Benchmarking square_without_black_box
Benchmarking square_without_black_box: Warming up for 3.0000 s
Benchmarking square_without_black_box: Collecting 100 samples in estimated 5.0000 s (21B iterations)
Benchmarking square_without_black_box: Analyzing
square_without_black_box
                        time:   [231.67 ps 234.23 ps 237.67 ps]
Found 12 outliers among 100 measurements (12.00%)
  9 (9.00%) high mild
  3 (3.00%) high severe
```

این‌بار فرق فقط حدودِ ۲ برابر است، نه نیم‌میلیون برابر — و این هم واقعی است، نه اشتباه. دلیلش این است که `Bencher::iter` خودش، بدونِ اینکه بخواهی، *خروجیِ* کلوژرت را در `black_box` می‌پیچد (همین‌طور که در «بیشتر» می‌توانی خودت در مستنداتِ criterion ببینی)، تا کلِ فراخوانی به‌خاطرِ «نتیجه‌اش استفاده نشد» حذف نشود. چیزی که از دستِ خودت باقی می‌ماند، محافظت از *ورودی* است — دقیقاً همان‌جایی که `square_without_black_box` هنوز یک‌ذره سریع‌تر است: کامپایلر نمی‌تواند کلِ فراخوانی را حذف کند (criterion جلویش را گرفته)، ولی هنوز می‌تواند ببیند ورودی همیشه `۷` است و ضربِ داخلش را زودتر حساب کند. (نکته‌ی حاشیه‌ای: خودِ `criterion` هم یک `black_box` دارد — این درس عمداً `std::hint::black_box` را مستقیم به‌کار می‌برد، همانی که [۱.۱.۳](../../../phase1-fundamentals/01-foundations/03-compound-types-and-destructuring/README.fa.md) از قبل نشانت داد.)

### کِی واقعاً ارزشِ بنچمارک‌گرفتن دارد

امروز دو مقایسه دیدی: یکی نزدیک به ۳۸ برابر فرق داشت، دیگری فقط حدودِ ۲ برابر. پیش از اجرا کردن، نمی‌توانستی مطمئن باشی کدام‌یک کدام است — و همین دقیقاً چرا بنچمارک‌گرفتن ارزش دارد: جوابِ دقیق را جایِ حدس می‌گذارد. ولی همین مثال یک نکته‌ی مهم‌تر هم دارد: هیچ‌کدام از این دو تابع را بنچمارک نکردیم چون *فکر می‌کردیم* کند است — بنچمارک کردیم چون داشتیم دقیقاً همین درس را می‌گفتیم.

در کدِ واقعی، ترتیب باید برعکس باشد. دونالد کنوت (Donald Knuth) در سالِ ۱۹۷۴ نوشت:

> "We should forget about small efficiencies, say about 97% of the time: premature optimization is the root of all evil. Yet we should not pass up our opportunities in that critical 3%."
>
> — Donald Knuth, *Structured Programming with go to Statements*, 1974

یعنی: بیشترِ کدِ یک برنامه اصلاً مسیرِ داغ نیست — دیتابیس، شبکه، یا یک کاربر منتظرِ کلیک، صدها برابرِ کندتر از هر تفاوتی‌اند که بینِ دو پیاده‌سازیِ تابعِ خودت پیدا می‌کنی. بنچمارک‌گرفتن روی کدی که اصلاً گلوگاه نیست، وقتی تلف‌کردن است که می‌توانستی صرفِ چیزی کنی که واقعاً اهمیت دارد — و بدتر، خوانایی را قربانیِ سرعتی می‌کند که کسی حسش نمی‌کند.

ترتیبِ درست: اول با یک ابزارِ نمایه‌بردار (profiler) — یا حتی یک متریکِ تأخیرِ واقعی در پروداکشن — بفهم کدام تابع واقعاً وقتِ زیادی می‌خورد. بعد، فقط برایِ همان یکی، فرضیه‌ی خودت را با `criterion` بسنج. بنچمارک‌گرفتن جوابِ «کدام‌یک سریع‌تر است» را دقیق می‌دهد؛ جوابِ «کدام‌یک اصلاً مهم است» را نمی‌دهد — آن سؤال را جایِ دیگری جواب می‌دهی.

---

## دست‌به‌کد

```sh
cargo run --release -p p2-07-05-benchmarking-with-criterion --example 01-naive-timing
cargo run --release -p p2-07-05-benchmarking-with-criterion --example 02-why-black-box
cargo bench -p p2-07-05-benchmarking-with-criterion
```

بعد سه‌تای خراب:

```sh
cargo build -p p2-07-05-benchmarking-with-criterion --example 03-nightly-bench-attribute --features broken
cargo build -p p2-07-05-benchmarking-with-criterion --example 04-forgot-black-box-import --features broken
cargo build -p p2-07-05-benchmarking-with-criterion --bench missing-criterion-main --features broken
```

بعد این‌ها را امتحان کن:

۱. در `01-naive-timing`، اندازه‌ی `haystack` را از ۱۰۰۰ به ۱۰۰_۰۰۰ ببر و چند بار دوباره اجرا کن. سه عددِ هر اجرا نسبت‌به هم نویزدارترند یا کمتر؟
۲. در `02-why-black-box`، `ITERS` را از ۱۰۰_۰۰۰_۰۰۰ به ۱_۰۰۰_۰۰۰ کم کن. عددِ «without black_box» هنوز همین‌قدر چشمگیر می‌ماند؟ چرا فکر می‌کنی همین‌طور شد؟
۳. بعد از `cargo bench`، یک گزارشِ HTML هم داری: `target/criterion/report/index.html` (نسبت‌به ریشه‌ی مخزن). بازش کن و نمودارِ توزیعِ `contains_linear_1000` را با `contains_hashset_1000` مقایسه کن.

---

## خطاهایی که خواهی دید

### `E0554` — هارنسِ توکارِ Rust نیازمندِ nightly است

```text
error[E0554]: `#![feature]` may not be used on the stable release channel
 --> phase2-intermediate\07-project-structure-and-testing\05-benchmarking-with-criterion\examples\03-nightly-bench-attribute.rs:9:1
  |
9 | #![feature(test)]
  | ^^^^^^^^^^^^^^^^^
```

**کامپایلر به چه اعتراض دارد:** خودِ Rust یک ویژگیِ `#[bench]` دارد — قدیمی‌تر از `criterion` — ولی پشتِ یک فیچرِ ناپایدار (`test`) قفل است که فقط رویِ کانالِ nightly در دسترس است. `examples/03-nightly-bench-attribute.rs` سعی کرده همان راه را برود؛ کامپایلرِ stable که این مخزن رویش کار می‌کند، حتی اجازه نمی‌دهد این فایل ساخته شود.

**راه‌حل:** به‌جایِ `#![feature(test)]`/`extern crate test`/`#[bench]`، یا یک اندازه‌گیریِ ساده با `Instant` بنویس (`01-naive-timing` را الگو بگیر)، یا `criterion` را به‌کار ببر — که خودِ این درس است.

**چرا این راه‌حل است:** `criterion` دقیقاً به همین دلیل وجود دارد که هارنسِ توکار روی stable در دسترس نیست. یک کریتِ معمولی (`criterion = "0.5"` در `[dev-dependencies]`) با هارنسِ خودش (`harness = false`) این محدودیت را کاملاً دور می‌زند و اضافه‌بر آن، آماری‌تر هم هست.

### `E0425` — `black_box` بدونِ ایمپورت

```text
error[E0425]: cannot find function `black_box` in this scope
 --> phase2-intermediate\07-project-structure-and-testing\05-benchmarking-with-criterion\examples\04-forgot-black-box-import.rs:9:18
  |
9 |     let hidden = black_box(7);
  |                  ^^^^^^^^^ not found in this scope
  |
help: consider importing this function
  |
8 + use std::hint::black_box;
  |
```

**کامپایلر به چه اعتراض دارد:** `black_box` بخشی از prelude نیست — مثلِ هر چیزِ دیگری در `std`، باید صریح ایمپورتش کنی. این فایل بدونِ `use std::hint::black_box;` صدایش زده.

**راه‌حل:** خطِ پیشنهادیِ خودِ کامپایلر را اضافه کن: `use std::hint::black_box;`.

**چرا این راه‌حل است:** خودِ پیامِ کمکِ کامپایلر دقیقاً همین را می‌گوید — و درست است. این شایع‌ترین اشتباهِ تایپی‌ای است که وقتی یک بنچمارکِ criterion را از حفظ می‌نویسی به‌اش می‌خوری.

### `E0601` — `criterion_group!` بدونِ `criterion_main!`

```text
error[E0601]: `main` function not found in crate `missing_criterion_main`
  --> phase2-intermediate\07-project-structure-and-testing\05-benchmarking-with-criterion\benches\missing-criterion-main.rs:19:45
   |
19 | criterion_group!(benches, lookup_benchmark);
   |                                             ^ consider adding a `main` function to `phase2-intermediate\07-project-structure-and-testing\05-benchmarking-with-criterion\benches\missing-criterion-main.rs`
```

**کامپایلر به چه اعتراض دارد:** `criterion_group!` فقط می‌گوید کدام توابع جزوِ این بنچ‌اند — چیزی را اجرا نمی‌کند و `fn main` نمی‌سازد. ساختنِ `fn main` تنها کارِ `criterion_main!` است. این فایل اولی را دارد، دومی را فراموش کرده — و یک کریتِ باینری بدونِ `main`، دقیقاً مثلِ هر کریتِ باینریِ دیگری، کامپایل نمی‌شود.

**راه‌حل:** یک خط اضافه کن: `criterion_main!(benches);`.

**چرا این راه‌حل است:** `benches/comparison.rs` را ببین — همین یک ماکرو، دقیقاً همین‌جا، است که `fn main`ِ واقعی را تولید می‌کند. بدونش، `criterion_group!` فقط یک لیست است که هیچ‌کس صدایش نمی‌زند.

---

## تمرین

### گرم‌کردن

<details>
<summary>این کد را سه بار پشتِ سرِ هم اجرا می‌کنی و سه عددِ متفاوت می‌بینی. کدام‌یک را باید باور کنی؟</summary>

```rust
let start = Instant::now();
let result = do_work();
println!("{:?}", start.elapsed());
```

</details>

<details>
<summary>پاسخ</summary>

هیچ‌کدام را، به‌تنهایی. یک نمونه‌ی تنها نمی‌تواند نویزِ اجرا-به-اجرا، مقیاس‌گذاریِ فرکانسِ CPU، یا هزینه‌ی گرم‌نشدن را از رفتارِ واقعیِ تابع جدا کند. برایِ یک عددِ قابلِ‌اعتماد، به بارها اجرا کردن و آمار نیاز داری — دقیقاً کارِ `criterion`.

</details>

<details>
<summary>دو حلقه، هرکدام ۱۰۰ میلیون بار <code>square(x)</code> را صدا می‌زنند — یکی با <code>black_box(7)</code>، دیگری با <code>7</code>ِ خام. کدام سریع‌تر «تمام» می‌شود، و چرا؟</summary>

نسخه‌ی بدونِ `black_box`. کامپایلر می‌بیند ورودی همیشه `۷` است، جوابش را یک‌بار سرِ کامپایل حساب می‌کند، و حلقه را به یک محاسبه‌ی ثابت تبدیل می‌کند — نه اینکه واقعاً ۱۰۰ میلیون بار ضرب بزند.

</details>

<details>
<summary>این فایل، بدونِ هیچ تغییرِ دیگری، کامپایل می‌شود؟</summary>

```rust
use criterion::{criterion_group, Criterion};

fn my_benchmark(c: &mut Criterion) {
    c.bench_function("x", |b| b.iter(|| 1 + 1));
}

criterion_group!(benches, my_benchmark);
```

</details>

<details>
<summary>پاسخ</summary>

نه. `criterion_group!` به‌تنهایی `fn main` نمی‌سازد؛ این فایل دقیقاً همان چیزی است که در «خطاهایی که خواهی دید» به‌عنوانِ `E0601` دیدی. یک خطِ `criterion_main!(benches);` کم دارد.

</details>

<details>
<summary><code>contains_linear_1000 time: [240.95 ns 242.13 ns 243.40 ns]</code> — این سه عدد یعنی چه، و کدامش را نقل می‌کنی وقتی به کسی می‌گویی «این تابع حدود ۲۴۲ نانوثانیه طول می‌کشد»؟</summary>

مرزِ پایین، میانگین، و مرزِ بالایِ بازه‌ی اطمینان‌اند. وسطی — ۲۴۲.۱۳ نانوثانیه — همان چیزی است که نقل می‌کنی؛ دو تایِ کناری‌اش بهت می‌گویند این میانگین چقدر قابلِ‌اعتماد است، نه فقط یک عددِ تنها که باید کورکورانه باورش کنی.

</details>

### تعمیر

هر سه مثالِ خراب را درست کن:

۱. `examples/03-nightly-bench-attribute.rs` را طوری بازنویسی کن که رویِ stable کامپایل شود — یا با یک اندازه‌گیریِ سادهِ `Instant`، یا با تبدیلش به یک بنچِ criterionِ واقعی.
۲. `examples/04-forgot-black-box-import.rs` را با اضافه‌کردنِ `use std::hint::black_box;` درست کن.
۳. `benches/missing-criterion-main.rs` را با اضافه‌کردنِ `criterion_main!(benches);` درست کن، بعد `cargo bench -p p2-07-05-benchmarking-with-criterion --bench missing-criterion-main --features broken` را اجرا کن و ببین واقعاً کار می‌کند.

### پیاده‌سازی

دو تابع در `src/lib.rs` — `sum_of_squares_loop` و `sum_of_squares_iter` — هردو باید مجموعِ مربعِ هر عددِ صحیح از ۱ تا `n` را برگردانند (۰ برایِ `n == 0`؛ مثلاً برایِ `n = 3` جواب `1*1 + 2*2 + 3*3 = 14` است). یکی را با یک حلقه‌ی `for` بنویس، دیگری را با یک زنجیره‌ی ایتریتور — بدونِ حلقه.

```sh
cargo test -p p2-07-05-benchmarking-with-criterion
```

### بساز

`benches/comparison.rs` را باز کن و یک تابعِ بنچمارکِ سوم اضافه کن که `sum_of_squares_loop` و `sum_of_squares_iter` را — با همان `n` برایِ هر دو، آن‌قدر بزرگ که تفاوت (اگر باشد) دیده شود — مقایسه کند. اسمش را به `criterion_group!` اضافه کن، `cargo bench` را اجرا کن، و به عددهایی که می‌بینی نگاه کن. [۲.۲.۵](../../02-iterators-and-closures/05-laziness-and-performance/README.fa.md) با شمردنِ فراخوانی‌ها ثابت کرد یک زنجیره‌ی ایتریتور دقیقاً به‌اندازه‌ی یک حلقه‌ی دستی کار می‌کند، نه بیشتر. امروز، با زمان، ببین همان ادعا چقدر واقعاً درست از آب درمی‌آید.

### چالش (اختیاری)

**بخشِ یک.** `criterion` می‌تواند نتیجه را ذخیره و بعداً مقایسه کند: `cargo bench -p p2-07-05-benchmarking-with-criterion -- --save-baseline before`. حالا عمداً چیزی را در `lookup_benchmark` کندتر کن — مثلاً `haystack` را از ۱۰۰۰ عنصر به ۱۰۰_۰۰۰ ببر — دوباره اجرا کن: `cargo bench -p p2-07-05-benchmarking-with-criterion -- --baseline before`. `criterion` چه چیزی درباره‌ی تغییر گزارش می‌دهد؟

**بخشِ دو.** (این یکی جلوتر را نگاه می‌کند.) بنچمارک بهت می‌گوید *کدام* پیاده‌سازی کندتر است و *چقدر* — نمی‌گوید *چرا* یک پیاده‌سازیِ به‌خصوص کند است، وقتی خودش هم چند تابع را صدا می‌زند. ابزاری که دقیقاً همین را جواب می‌دهد — یک نمودارِ شعله‌ای (flamegraph) که نشان می‌دهد وقت داخلِ کدام تابع رفته — مالِ [فاز ۴ — بنچمارک‌هایِ criterion و نمودارِ شعله‌ای](../../../phase4-backend-advanced/08-performance-and-profiling/01-criterion-benchmarks-and-flamegraphs/README.fa.md) است.

---

## جمع‌بندی

این درس، ماژول را می‌بندد. یک نگاهِ کوتاه به مسیری که طی کردی: [۲.۷.۱](../01-modules-visibility-workspaces/README.fa.md) به کد ساختار و دیدپذیری داد؛ [۲.۷.۲](../02-unit-integration-doc-tests/README.fa.md) سه‌جور تستِ توکار را از هم جدا کرد؛ [۲.۷.۳](../03-test-doubles-in-rust/README.fa.md) بدل‌هایی برایِ وابستگی‌هایِ قابل-جایگزین ساخت؛ [۲.۷.۴](../04-property-and-snapshot-testing/README.fa.md) وقتی فضایِ ورودی خیلی بزرگ بود یا خروجی خیلی پیچیده، راهِ تستش را نشان داد. امروز، آخرین قدم: وقتی می‌دانی کد درست کار می‌کند و کارایی‌اش واقعاً مهم شده، `criterion` جوابِ «کدام‌یک سریع‌تر است» را با آمار می‌دهد، نه با حدس.

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| سنجشِ آماریِ کارایی | اجرایِ یک تابع بارها، گزارشِ میانگین+بازه‌ی اطمینان، به‌جایِ یک نمونه‌ی تنها | مقایسه‌یِ دو پیاده‌سازی |
| `criterion` | کریتِ سنجشِ کارایی که این کار را برایت می‌کند | `[dev-dependencies]`، `cargo bench` |
| `criterion_group!` / `criterion_main!` | ماکروهایی که توابعِ بنچمارک را ثبت و `fn main` را می‌سازند | هر فایلِ `benches/*.rs` |
| `harness = false` | به cargo می‌گوید هارنسِ توکارِ تست را برایِ این هدف به‌کار نبرد | `[[bench]]` در `Cargo.toml` |
| `black_box` | مقداری را از دیدِ بهینه‌سازِ کامپایلر پنهان می‌کند | ورودی‌هایِ هر بنچمارک |
| علامت‌گذاریِ داده‌ی پرت | نمونه‌هایی که آماری با بقیه جور درنمی‌آیند | خواندنِ خروجیِ `cargo bench` |
| بنچمارک بعد از نمایه‌برداری | فقط پس از اینکه دانستی کدام تابع واقعاً کند است | تصمیمِ کِی بنچمارک بگیری |

### الان می‌دانی

- چرا یک `Instant::now()`/`.elapsed()`ِ تنها به اندازه‌ی کافی نویزدار است که نتوانی بهش اعتماد کنی — و سه دلیلِ مشخصش را.
- `criterion` چطور با گرم‌کردن، نمونه‌گیریِ زیاد، و آمار (میانگین، بازه‌ی اطمینان، داده‌هایِ پرت) این مشکل را حل می‌کند.
- چطور یک بنچمارکِ واقعی می‌نویسی: `[[bench]]` با `harness = false` در `Cargo.toml`، `criterion_group!`/`criterion_main!` در فایل.
- `std::hint::black_box` جلویِ چه چیزی را می‌گیرد، و چرا بدونش یک بنچمارک می‌تواند زمانِ *هیچ‌کاری* را اندازه بگیرد نه زمانِ کاری که فکر می‌کنی.
- چرا بنچمارک‌گرفتن پیش از نمایه‌برداری (profiling) — روی کدی که هنوز نمی‌دانی گلوگاه است یا نه — وقتِ تلف‌شده است، نه احتیاط.

### بعداً کامل‌تر می‌بینی

- **نمودارِ شعله‌ای (flamegraph) — فهمیدنِ *چرا* یک تابعِ به‌خصوص کند است، نه فقط اینکه هست** — [فاز ۴ — بنچمارک‌هایِ criterion و نمودارِ شعله‌ای](../../../phase4-backend-advanced/08-performance-and-profiling/01-criterion-benchmarks-and-flamegraphs/README.fa.md)
- **مقایسه با یک baselineِ ذخیره‌شده، برایِ تشخیصِ خودکارِ رگرسیون** — همان درسِ فاز ۴، عمیق‌تر از چیزی که در «چالش» امروز دیدی.

### می‌توانی توضیح بدهی؟

- چرا سه بار اجرا کردنِ یک `Instant::now()`/`.elapsed()`ِ خام سه عددِ متفاوت می‌دهد، و کدام‌شان را باید باور کنی؟
- `criterion` دقیقاً چه کاری می‌کند که یک `println!(elapsed)` نمی‌کند؟
- `black_box` را با کلماتِ خودت توضیح بده — جلویِ چه چیزی را می‌گیرد، و چرا اسمش ربطی به «بنچمارک» ندارد؟
- چرا نسخه‌ی criterionِ مقایسه‌ی `square` فقط ۲ برابر فرق داشت، در حالی‌که نسخه‌ی حلقه‌ی خامِ همان مقایسه نزدیک به نیم‌میلیون برابر فرق داشت؟
- چرا بنچمارک‌گرفتن باید *بعدِ* نمایه‌برداری بیاید، نه قبلش؟

---

## بیشتر

- [کتابِ `criterion.rs`](https://bheisler.github.io/criterion.rs/book/index.html) — راهنمایِ رسمی، شاملِ بخشِ `--save-baseline`/`--baseline` که در «چالش» دیدی.
- [`std::hint::black_box`](https://doc.rust-lang.org/std/hint/fn.black_box.html) — مستنداتِ رسمی؛ توضیح می‌دهد چرا این تابع هیچ تضمینِ رسمی‌ای نمی‌دهد، فقط یک درخواستِ بهترین‌تلاش به کامپایلر است.
- [`Bencher::iter` در مستنداتِ criterion](https://docs.rs/criterion/latest/criterion/struct.Bencher.html#method.iter) — همان‌جا که می‌بینی خروجیِ کلوژرت خودش در `black_box` پیچیده می‌شود.
- Donald Knuth, *Structured Programming with go to Statements*, ACM Computing Surveys 6:4 (دسامبر ۱۹۷۴), ص ۲۶۱–۳۰۱ — [نسخه‌ی رایگانِ PDF](https://pic.plover.com/knuth-GOTO.pdf).

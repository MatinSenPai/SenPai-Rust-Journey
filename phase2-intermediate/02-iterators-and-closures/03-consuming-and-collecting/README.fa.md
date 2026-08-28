# ۲.۲.۳ — مصرف و جمع‌آوری، از جمله `Result<Vec<_>, E>`

## در یک نگاه

بعد از این درس می‌توانی:

- توضیح بدهی چرا `.collect()` باید از قبل بداند دارد چه می‌سازد، و با توربوفیش یا نوع‌نویسی این را بهش بگویی.
- همان ایتریتور را به `Vec`، `String`، `HashMap` یا `HashSet` جمع کنی — هرکدام را که موقعیت خواست.
- مصرف‌کننده‌ی درست را برایِ یک موقعیتِ مشخص انتخاب کنی: `.sum()`، `.count()`، `.min()`/`.max()`، `.find()`، `.any()`/`.all()` یا `.last()`.
- یک ایتریتور از `Result`ها را مستقیم به یک `Result<Vec<T>, E>` جمع کنی، و بگویی چرا اولین `Err` همان لحظه کلِ جمع‌آوری را متوقف می‌کند.

**زمان:** حدود ۷۰ دقیقه · **پیش‌نیاز:** [۲.۲.۲ — آداپتورهای ایتریتور](../02-iterator-adapters/README.fa.md)

---

## چرا اهمیت دارد

درسِ قبل، [۲.۲.۲](../02-iterator-adapters/README.fa.md)، با یک جمله تمام شد که شاید گذرا خوانده باشیش: آداپتورهای ایتریتور — `.map()`، `.filter()`، `.enumerate()` و بقیه — تنبل‌اند، و تا وقتی کسی واقعاً مصرفشان نکند هیچ کاری انجام نمی‌دهند. آن «کسی» موضوعِ همین درس است.

یک بدهیِ قدیمی‌تر هم هست. [۱.۷.۲](../../../phase1-fundamentals/07-putting-it-together/02-phase-review/README.fa.md) — مرورِ فازِ ۱ — یک خط از `.filter()` و `.collect()` رویِ یک بازه جلوی چشمت گذاشت و رک گفت: «این‌ها از فازِ ۲ می‌آیند، ماژولِ ۲.۲، و هنوز رسماً یادشان نگرفته‌ای.» حالا وقتش است.

و یک سؤالِ ساکت‌تر هم هست. در [۱.۶.۳](../../../phase1-fundamentals/06-absence-and-failure/03-result-and-question-mark/README.fa.md)، در تمرینِ «بساز»، ازت خواسته شد یک `pub fn parse_all(inputs: &[&str]) -> Result<Vec<i32>, String>` بنویسی: هر رشته را با دست، تویِ یک حلقه، با `?` پارس کنی — و اگر یکی شکست خورد، همان لحظه با همان خطا برگردی، بدونِ امتحان‌کردنِ باقی. اگر آن تمرین را واقعاً نوشتی، احتمالاً یک لحظه فکر کردی «این حلقه برایِ یک الگویِ این‌قدر رایج، طولانی به نظر می‌رسد — حتماً یک راهِ کوتاه‌ترش هم هست.» بود. همین‌جاست.

اگر از پایتون آمده باشی: معمولاً یک لیست را با `list(...)` یا `[... for ...]`، یک دیکشنری را با `dict(...)` یا `{...: ... for ...}`، و یک ست را با `set(...)` می‌سازی — هر مقصد، تابع یا نحوِ خودش را دارد، و همان اسم یا نحو به‌تنهایی می‌گوید چه چیزی دارد ساخته می‌شود. Rust همه‌ی این‌ها را با یک متدِ واحد انجام می‌دهد: `.collect()`. اسمِ متد به‌تنهایی هیچ نشانه‌ای از هدف نمی‌دهد — این دقیقاً همان چیزی است که بخشِ اولِ درس رویش مکث می‌کند. و برایِ «همه را پردازش کن، ولی به‌محضِ اولین خطا متوقف شو»، پایتون معمولاً یک حلقه‌ی صریح با `try`/`except` می‌خواهد؛ در Rust همان کار یک `.collect()` است — مرکزِ ثقلِ همین درس.

---

## مفهوم

### مصرف‌کننده‌ها: چیزی که خط‌لوله را واقعاً می‌کِشد

در [۲.۲.۲](../02-iterator-adapters/README.fa.md) دیدی که `.filter()`، `.map()` و بقیه — که آن‌ها را آداپتور می‌نامیم — به‌تنهایی هیچ کاری نمی‌کنند؛ فقط یک خط‌لوله را توصیف می‌کنند. چیزی که واقعاً آن خط‌لوله را می‌کشد و کار را انجام می‌دهد، یک **مصرف‌کننده (consuming adapter)** است. `.collect()` معروف‌ترینشان است، ولی تنها یکی از چندتاست — همان ایتریتور را می‌شود به چند مصرف‌کننده‌ی متفاوت داد، بسته به این‌که در آخر چه چیزی لازم داری:

```rust
let numbers = vec![1, 2, 3, 4, 5, 6];

let evens: Vec<i32> = numbers.iter().filter(|&&n| n % 2 == 0).copied().collect();
let even_count = numbers.iter().filter(|&&n| n % 2 == 0).count();
let even_sum: i32 = numbers.iter().filter(|&&n| n % 2 == 0).sum();

println!("evens: {evens:?}");
println!("count: {even_count}");
println!("sum:   {even_sum}");
```

```text
evens: [2, 4, 6]
count: 3
sum:   12
```

سه بار همان `.filter()` نوشته شده، چون هر بار قرار است متدِ مصرف‌کننده‌ی متفاوتی صدایش بزند — یک ایتریتور که یک‌بار مصرف شد، تمام شده؛ نمی‌شود دوباره از همان مقدار برایِ متدِ بعدی استفاده کرد. باقیِ این درس دورِ همین ایده می‌چرخد: کدام مصرف‌کننده را کِی انتخاب کنی.

### `.collect()` باید بداند دارد چه می‌سازد

از بینِ همه‌ی مصرف‌کننده‌ها، `.collect()` یک فرقِ اساسی با بقیه دارد: `.sum()`، `.count()`، `.find()` و امثالشان هرکدام دقیقاً یک نوع خروجی می‌دهند — `.count()` همیشه یک `usize` می‌دهد، فرقی نمی‌کند چه ایتریتوری رویش صدا بزنی. ولی `.collect()` می‌تواند از رویِ همان ایتریتور، بسته به این‌که تو چه بخواهی، چندین چیزِ کاملاً متفاوت بسازد: یک `Vec`، یک `HashMap`، یک `String`، حتی — همان‌طور که تا آخرِ این درس می‌بینی — یک `Result`. برایِ همین، کامپایلر تنها با دیدنِ `.collect()` نمی‌تواند حدس بزند هدف چیست؛ باید صریح بهش بگویی، به یکی از دو روش:

```rust
let long_runs: Vec<i32> = episodes.iter().filter(|&&n| n >= 12).copied().collect();
println!("long_runs (annotated binding): {long_runs:?}");

let long_runs_turbofish = episodes
    .iter()
    .filter(|&&n| n >= 12)
    .copied()
    .collect::<Vec<i32>>();
println!("long_runs (turbofish):        {long_runs_turbofish:?}");
```

```text
long_runs (annotated binding): [12, 24, 50, 13]
long_runs (turbofish):        [12, 24, 50, 13]
```

(با `episodes = [12, 24, 6, 50, 13]`.) روشِ اول نوعِ متغیر را می‌نویسد و Rust از همان‌جا به عقب می‌رود تا بفهمد `.collect()` باید چه بسازد. روشِ دوم — همان توربوفیشی که از [۱.۶.۳](../../../phase1-fundamentals/06-absence-and-failure/03-result-and-question-mark/README.fa.md) می‌شناسی — همان نوع را دقیقاً سرِ خودِ فراخوانی می‌نویسد، برایِ وقتی که مقدار بلافاصله مصرف می‌شود و جایی برایِ یک `let` جداگانه ندارد. حتی می‌شود فقط بخشی از نوع را نوشت: `collect::<Vec<_>>()` هم کار می‌کند، چون Rust نوعِ عنصر را می‌تواند از رویِ خودِ `episodes` بفهمد؛ فقط باید بداند کدام ظرف — `Vec`، نه چیزِ دیگر. اگر هیچ‌کدام از این دو راه را ننویسی، کامپایلر گیر می‌کند — این خودش یکی از بخشِ «خطاهایی که خواهی دید» است.

این توانایی که `.collect()` از رویِ یک ایتریتور چند چیزِ متفاوت بسازد، جادو نیست — یک صفتِ استاندارد به اسمِ `FromIterator` پشتش است. هر نوعی که بخواهد هدفِ `.collect()` باشد باید `FromIterator` را پیاده کرده باشد؛ `Vec`، `String`، `HashMap`، `HashSet` و — مرکزِ ثقلِ همین درس — حتی خودِ `Result`، همه‌شان این کار را کرده‌اند. چهارتای اول را همین حالا می‌بینی.

### جمع‌آوری در یک `String`

وقتی هر عنصر یک `char` یا یک `&str` است، `.collect()` می‌تواند مستقیم یک `String` بسازد — نه با یک جداکننده بینشان، فقط با چسباندنِ پشتِ سرِ هم:

```rust
let letters = ['R', 'u', 's', 't'];
let word: String = letters.into_iter().collect();
println!("chars -> String:   {word}");

let parts = ["Sen", "pai"];
let shout: String = parts.into_iter().collect();
println!("&str parts -> String: {shout}");
```

```text
chars -> String:   Rust
&str parts -> String: Senpai
```

اگر بینِ تکه‌ها یک جداکننده هم لازم داری — مثلاً یک فاصله بینِ کلمه‌ها — آن دیگر کارِ `.collect()` نیست؛ `.join(" ")` را رویِ خودِ برش می‌خواهی، نه رویِ ایتریتور. `.collect()` فقط می‌چسباند.

### جمع‌آوری در یک `HashMap`

وقتی هر عنصر یک تاپلِ `(کلید, مقدار)` است، `.collect()` می‌تواند یک `HashMap<K, V>` بسازد — همان کاری که در [۲.۱.۲](../../01-collections/02-hashmap-in-depth/README.fa.md) با `HashMap::new()` و یک حلقه‌ی `.insert()` انجام دادی، این‌بار در یک خط:

```rust
let entries = [("Frieren", 28), ("Bocchi the Rock", 12), ("K-On!", 13)];

let episodes: HashMap<&str, u32> = entries.into_iter().collect();
println!("Frieren episodes:  {:?}", episodes.get("Frieren"));
println!("total shows:       {}", episodes.len());
```

```text
Frieren episodes:  Some(28)
total shows:       3
```

اگر یک کلید تکرار شود، دقیقاً همان رفتاری را می‌گیری که از صدازدنِ `.insert()` چند بار پشتِ سرِ هم می‌گرفتی: مقدارِ آخرین جفتی که آن کلید را داشت برنده می‌شود — بقیه بی‌سروصدا جایگزین می‌شوند.

### جمع‌آوری در یک `HashSet`

وقتی فقط می‌خواهی بدونی «چه مقدارهایِ متمایزی اینجا هست»، نه چندبار هرکدام تکرار شده، `HashSet<T>` دقیقاً همان کاری را می‌کند که `Vec<T>` می‌کرد — با یک فرق: یک مقدارِ تکراری را بی‌سروصدا نادیده می‌گیرد، نه اینکه نگهش دارد:

```rust
let tags = ["comedy", "drama", "comedy", "slice of life", "drama", "comedy"];
println!("tags seen (with repeats): {}", tags.len());

let unique: HashSet<&str> = tags.into_iter().collect();
println!("unique tags:               {}", unique.len());
println!("contains \"drama\":          {}", unique.contains("drama"));
```

```text
tags seen (with repeats): 6
unique tags:               3
contains "drama":          true
```

هیچ منطقِ حذفِ تکراری‌ای دستی ننوشتی — همان صفتِ `Hash`/`Eq` که [۲.۱.۲](../../01-collections/02-hashmap-in-depth/README.fa.md) رویِ کلیدهایِ `HashMap` توضیح داد، اینجا هم پشتِ‌صحنه همین کار را می‌کند.

### مصرف‌کننده‌های دیگر: یک تورِ کوتاه

نه هر مصرف‌کننده‌ای یک مجموعه‌ی تازه می‌سازد. وقتی خروجیِ نهایی یک عدد است، یک `bool` است، یا فقط یک آیتم است — نه یک ظرفِ تازه — یکی از این‌ها مستقیم‌تر از `.collect()` است:

```rust
let ratings = [7, 9, 5, 10, 6];

println!("sum:              {}", ratings.iter().sum::<i32>());
println!("count:            {}", ratings.iter().filter(|&&r| r >= 7).count());
println!("min:              {:?}", ratings.iter().min());
println!("max:              {:?}", ratings.iter().max());
println!("first below 6:    {:?}", ratings.iter().find(|&&r| r < 6));
println!("all at least 5:   {}", ratings.iter().all(|&r| r >= 5));
println!("any perfect 10:   {}", ratings.iter().any(|&r| r == 10));
println!("last:             {:?}", ratings.iter().last());
```

```text
sum:              37
count:            3
min:              Some(5)
max:              Some(10)
first below 6:    Some(5)
all at least 5:   true
any perfect 10:   true
last:             Some(6)
```

نکته‌ای که `.find()` نشان می‌دهد: اولین آیتمی که شرط را برآورده می‌کند برمی‌گرداند، نه کوچک‌ترین یا بزرگ‌ترینشان — همان معنایی که در [۱.۱.۵](../../../phase1-fundamentals/01-foundations/05-control-flow/README.fa.md) با یک حلقه‌ی دستی نوشتی (`index_of_first_negative`)، حالا یک متد. `.min()` و `.max()` هردو `Option<&T>` برمی‌گردانند، چون یک ایتریتورِ خالی نه کمینه دارد نه بیشینه — همان قاعده‌ی `Option` که از فازِ ۱ می‌شناسی.

### مرکزِ ثقلِ درس: جمع‌آوریِ `Result` در یک `Result<Vec<T>, E>`

تا اینجا هر ایتریتوری که جمع کردی، آیتم‌هایِ «سالم» داشت — عددها، رشته‌ها، تاپل‌ها. ولی یک الگویِ خیلی رایج هست: یک لیست از رشته‌ها داری، هرکدام را می‌خواهی پارس کنی، و اگر همه موفق شدند یک `Vec` از نتیجه‌ها می‌خواهی — ولی اگر حتی یکی‌شان شکست خورد، کلِ کار باید شکست بخورد، با همان اولین خطا. دقیقاً همین چیزی است که در [۱.۶.۳](../../../phase1-fundamentals/06-absence-and-failure/03-result-and-question-mark/README.fa.md)، در تمرینِ «بساز»، با دست نوشتی — یک حلقه، یک `?`، یک بازگشتِ زودهنگام رویِ اولین `Err`.

حالا همان کار، در یک خط:

```rust
let all_good = ["1", "2", "3"];
let parsed: Result<Vec<i32>, _> = all_good.iter().map(|s| s.parse::<i32>()).collect();
println!("all valid:   {parsed:?}");
```

```text
all valid:   Ok([1, 2, 3])
```

هر `s.parse::<i32>()` یک `Result<i32, ParseIntError>` می‌دهد — پس ایتریتوری که به `.collect()` می‌رسد، آیتم‌هایش `Result` است، نه `i32`. با این حال نوعِ خواسته‌شده `Result<Vec<i32>, _>` است — یک `Result` که دورِ یک `Vec` پیچیده شده، نه یک `Vec` از `Result`ها. این جادو نیست: خودِ `Result<T, E>` هم، درست مثلِ `Vec` و `HashMap`، صفتِ `FromIterator` را پیاده کرده — پیاده‌سازی‌ای که دقیقاً همان کاری را می‌کند که در ۱.۶.۳ با دست نوشتی: هر آیتم را می‌خواند؛ اگر `Ok` بود مقدارش را نگه می‌دارد و سراغِ بعدی می‌رود؛ به‌محضِ رسیدن به اولین `Err`، همان لحظه متوقف می‌شود و همان `Err` کلِ جواب می‌شود. اگر تا آخر همه `Ok` بودند، یک `Ok` واحد می‌گیری که دورِ یک `Vec` از همه‌ی مقدارهایِ بازشده پیچیده شده.

اثرِ عملی‌اش **توقفِ زودهنگام (short-circuiting)** است — و می‌شود با چشمِ خودت دید، نه فقط باورش کرد:

```rust
let has_a_bad_one = ["1", "x", "3"];
let parsed: Result<Vec<i32>, _> = has_a_bad_one
    .iter()
    .map(|s| {
        println!("  parsing {s:?}...");
        s.parse::<i32>()
    })
    .collect();
println!("one invalid: {parsed:?}");
```

```text
  parsing "1"...
  parsing "x"...
one invalid: Err(ParseIntError { kind: InvalidDigit })
```

به `println!` توی خودِ کلوژر نگاه کن: `"3"` هیچ‌وقت پارس نمی‌شود. `.collect()` به‌محضِ اینکه از `"x"` یک `Err` گرفت، دیگر اصلاً سراغِ آیتمِ بعدیِ ایتریتور نرفت — چون `.map()` تنبل است (از [۲.۲.۲](../02-iterator-adapters/README.fa.md) یادت هست)، کلوژرش تا وقتی کسی `.next()` را صدا نزند اصلاً اجرا نمی‌شود، و `.collect()` بعد از دیدنِ اولین `Err` دیگر `.next()` صدا نمی‌زند.

```senpai-visual
{"kind":"result","labels":["[\"1\", \"x\", \"3\"]","\"1\" → Ok","\"x\" → Err","توقفِ فوری","خروجی: Err(e)"]}
```

اگر ۱.۶.۳ را نوشته باشی، این همان سؤالِ ساکتی است که شاید موقعِ نوشتنِ آن حلقه‌ی دستی تویِ ذهنت بود — «حتماً یک راهِ کوتاه‌ترش هست» — و حالا می‌دانی چرا کار می‌کند، نه فقط اینکه کار می‌کند.

---

## دست‌به‌کد

```sh
cargo run -p p2-02-03-consuming-and-collecting --example 01-collect-into-vec
cargo run -p p2-02-03-consuming-and-collecting --example 02-collect-into-string
cargo run -p p2-02-03-consuming-and-collecting --example 03-collect-into-hashmap
cargo run -p p2-02-03-consuming-and-collecting --example 04-collect-into-hashset
cargo run -p p2-02-03-consuming-and-collecting --example 05-other-consumers
cargo run -p p2-02-03-consuming-and-collecting --example 06-collecting-results
```

بعد سه‌تای خراب:

```sh
cargo run -p p2-02-03-consuming-and-collecting --example 07-collect-ambiguous-type --features broken
cargo run -p p2-02-03-consuming-and-collecting --example 08-collect-result-into-vec-directly --features broken
cargo run -p p2-02-03-consuming-and-collecting --example 09-max-on-floats --features broken
```

بعد این‌ها را امتحان کن:

۱. در `03-collect-into-hashmap`، یک تاپلِ چهارم — `("Frieren", 99)` — به انتهایِ `entries` اضافه کن. `episodes.get("Frieren")` بعدش چه می‌شود؟
۲. در `04-collect-into-hashset`، فقط نوعِ `unique` را از `HashSet<&str>` به `Vec<&str>` عوض کن (خودِ `.collect()` دست‌نخورده می‌ماند). `unique.len()` این‌بار با `tags.len()` برابر می‌شود یا نه؟ چرا؟
۳. در `06-collecting-results`، ترتیبِ `has_a_bad_one` را به `["1", "3", "x"]` عوض کن — یعنی رشته‌ی خراب را آخر بگذار. این‌بار کدام رشته‌ها را می‌بینی که «parsing ...» چاپ می‌کنند؟

---

## خطاهایی که خواهی دید

### `E0283` — کامپایلر نمی‌داند `.collect()` باید چه بسازد

```text
error[E0283]: type annotations needed
    --> phase2-intermediate\02-iterators-and-closures\03-consuming-and-collecting\examples\07-collect-ambiguous-type.rs:10:9
     |
  10 |     let evens = (1..10).filter(|n| n % 2 == 0).collect();
     |         ^^^^^                                  ------- type must be known at this point
     |
     = note: the type must implement `FromIterator<i32>`
note: required by a bound in `collect`
    --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\iter\traits\iterator.rs:2077:19
     |
2077 |     fn collect<B: FromIterator<Self::Item>>(self) -> B
     |                   ^^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `Iterator::collect`
help: consider giving `evens` an explicit type
     |
  10 |     let evens: Vec<_> = (1..10).filter(|n| n % 2 == 0).collect();
     |              ++++++++

For more information about this error, try `rustc --explain E0283`.
```

**کامپایلر به چه اعتراض دارد:** پیامِ `= note` دقیقاً همان چیزی است که بخشِ «مفهوم» گفت: نوعِ `evens` باید صفتِ `FromIterator<i32>` را پیاده کند — ولی هیچ نوعی به‌طورِ پیش‌فرض «آن یکی» نیست؛ می‌تواند `Vec<i32>` باشد، `HashSet<i32>`، یا هر نوعِ دیگری که همین صفت را پیاده کرده. کامپایلر نمی‌تواند حدس بزند، و همین را می‌گوید.

**راه‌حل:** یکی از دو راهی که در بخشِ «مفهوم» دیدی:

```rust
let evens: Vec<i32> = (1..10).filter(|n| n % 2 == 0).collect();
```

```text
[2, 4, 6, 8]
```

**چرا این راه‌حل است:** نوعِ `evens` حالا صریح است، پس Rust می‌داند دقیقاً کدام پیاده‌سازیِ `FromIterator` را صدا بزند. همان توربوفیش — `.collect::<Vec<i32>>()` — هم همین کار را می‌کرد؛ فقط جایِ نوشتنِ نوع فرق دارد.

### `E0277` — یک `Vec<i32>` از رویِ ایتریتورِ `Result` ساخته نمی‌شود

```text
error[E0277]: a value of type `Vec<i32>` cannot be built from an iterator over elements of type `Result<i32, ParseIntError>`
    --> phase2-intermediate\02-iterators-and-closures\03-consuming-and-collecting\examples\08-collect-result-into-vec-directly.rs:13:68
     |
  13 |     let parsed: Vec<i32> = inputs.iter().map(|s| s.parse::<i32>()).collect();
     |                                                                    ^^^^^^^ value of type `Vec<i32>` cannot be built from `std::iter::Iterator<Item=Result<i32, ParseIntError>>`
     |
help: the trait `FromIterator<Result<i32, ParseIntError>>` is not implemented for `Vec<i32>`
      but trait `FromIterator<i32>` is implemented for it
    --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\alloc\src\vec\mod.rs:3923:1
     |
3923 | impl<T> FromIterator<T> for Vec<T> {
     | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
     = help: for that trait implementation, expected `i32`, found `Result<i32, ParseIntError>`
note: the method call chain might not have had the expected associated types
    --> phase2-intermediate\02-iterators-and-closures\03-consuming-and-collecting\examples\08-collect-result-into-vec-directly.rs:13:42
     |
  12 |     let inputs = ["1", "2", "x"];
     |                  --------------- this expression has type `[&str; 3]`
  13 |     let parsed: Vec<i32> = inputs.iter().map(|s| s.parse::<i32>()).collect();
     |                                   ------ ^^^^^^^^^^^^^^^^^^^^^^^^^ `Iterator::Item` changed to `Result<i32, ParseIntError>` here
     |                                   |
     |                                   `Iterator::Item` is `&&str` here
note: required by a bound in `collect`
    --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\iter\traits\iterator.rs:2077:19
     |
2077 |     fn collect<B: FromIterator<Self::Item>>(self) -> B
     |                   ^^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `Iterator::collect`

For more information about this error, try `rustc --explain E0277`.
```

**کامپایلر به چه اعتراض دارد:** این طولانی‌تر از خطاهایی است که تا الان دیده‌ای، ولی خبرِ اصلی‌اش همان خطِ اول است: `Vec<i32>` از رویِ یک ایتریتور که آیتم‌هایش `Result<i32, ParseIntError>` است، نمی‌تواند ساخته شود. `s.parse::<i32>()` یک `Result` برمی‌گرداند، نه یک `i32` خام؛ پس بعدِ `.map()`، آیتم‌هایِ ایتریتور `Result<i32, ParseIntError>`اند. `help` اولش دقیقاً می‌گوید `Vec<i32>` فقط `FromIterator<i32>` را پیاده کرده — نه `FromIterator<Result<i32, ParseIntError>>` را.

**راه‌حل:** به‌جایِ خواستنِ `Vec<i32>`، همان چیزی را بخواه که این ایتریتور واقعاً می‌تواند بسازد — یک `Result` که دورِ `Vec` پیچیده شده:

```rust
let parsed: Result<Vec<i32>, _> = inputs.iter().map(|s| s.parse::<i32>()).collect();
```

```text
Err(ParseIntError { kind: InvalidDigit })
```

(با `inputs = ["1", "2", "x"]`.)

**چرا این راه‌حل است:** حالا نوعِ خواسته‌شده دقیقاً با نوعِ آیتم‌هایِ ایتریتور جور است — چون `Result<Vec<i32>, ParseIntError>` همان صفتی را پیاده می‌کند که لازم است: `FromIterator<Result<i32, ParseIntError>>`. این دقیقاً همان مکانیزمِ بخشِ «مفهوم» است، این‌بار از زاویه‌ی خطا.

### `E0277` — `.max()` به یک ترتیبِ کامل نیاز دارد که `f64` قولش را نمی‌دهد

```text
error[E0277]: the trait bound `{float}: Ord` is not satisfied
    --> phase2-intermediate\02-iterators-and-closures\03-consuming-and-collecting\examples\09-max-on-floats.rs:11:33
     |
  11 |     let biggest = values.iter().max();
     |                                 ^^^ the trait `Ord` is not implemented for `{float}`
     |
     = help: the following other types implement trait `Ord`:
               i128
               i16
               i32
               i64
               i8
               isize
               u128
               u16
             and 4 others
     = note: required for `&{float}` to implement `Ord`
note: required by a bound in `std::iter::Iterator::max`
    --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\iter\traits\iterator.rs:3253:21
     |
3250 |     fn max(self) -> Option<Self::Item>
     |        --- required by a bound in this associated function
...
3253 |         Self::Item: Ord,
     |                     ^^^ required by this bound in `Iterator::max`

For more information about this error, try `rustc --explain E0277`.
```

**کامپایلر به چه اعتراض دارد:** `.max()` (بدونِ آرگومان) فقط برایِ نوع‌هایی کار می‌کند که یک ترتیبِ کامل و بی‌ابهام دارند — برایِ هر دو مقدار، همیشه دقیقاً یک جواب هست به «کدام بزرگ‌تر است؟». این تضمین اسمِ رسمی دارد، یک صفت به اسمِ `Ord`، و `f64` آن را پیاده نکرده — به‌خاطرِ `NaN` («عدد نیست»)، که با هیچ عددِ دیگری، حتی با خودش، قابلِ‌مقایسه نیست. (خودِ `Ord` و خانواده‌اش کاملاً مالِ [۲.۳.۴](../../03-traits-and-generics/04-standard-derives-by-hand/README.fa.md) است؛ همین یک پاراگراف امروز کافی است.)

**راه‌حل:** یک مقایسه‌ی صریح بده، به‌جایِ اینکه منتظرِ `Ord` بمانی:

```rust
let values: Vec<f64> = vec![1.0, 5.5, 2.3];
let biggest = values.iter().max_by(|a, b| a.total_cmp(b));
println!("{biggest:?}");
```

```text
Some(5.5)
```

**چرا این راه‌حل است:** `.max_by()` خودت مقایسه را می‌نویسی، پس دیگر منتظرِ `Ord` نمی‌ماند. `f64::total_cmp` دقیقاً برایِ همین ساخته شده — یک ترتیبِ کامل رویِ همه‌یِ مقدارهایِ `f64`، شاملِ `NaN`. (این همان `total_cmp` ای است که در [۲.۱.۱](../../01-collections/01-vec-depth/README.fa.md) برایِ `.sort_by()` دیدی — همان ابزار، این‌بار برایِ `.max_by()`.)

---

## تمرین

### گرم‌کردن

<details>
<summary>این کامپایل می‌شود؟ اگر بله، چه چاپ می‌کند؟

```rust
let v = vec![1, 2, 3];
let doubled = v.iter().map(|n| n * 2);
println!("{doubled:?}");
```
</summary>

```text
Map { iter: Iter([1, 2, 3]) }
```

بله کامپایل می‌شود — `doubled` فقط توصیفِ یک خط‌لوله است، همان ساختارِ داخلیِ `Map` ایتریتور، هیچ عددِ دوبرابرشده‌ای داخلش نیست. `{:?}` این ساختار را نشان می‌دهد، نه چیزی که اگر مصرفش می‌کردی می‌گرفتی. این دقیقاً همان تنبلی‌ای است که [۲.۲.۲](../02-iterator-adapters/README.fa.md) نشانت داد — حتی چاپ‌کردن هم `.map()` را اجرا نمی‌کند.

</details>

<details>
<summary>این کامپایل می‌شود؟

```rust
let names = vec!["a", "b"];
let joined = names.into_iter().collect();
println!("{joined}");
```
</summary>

نه. `.collect()` نمی‌داند باید `String` بسازد یا `Vec<&str>` یا چیزِ دیگر — کدِ خطا `E0283` است؛ کاملِ ماجرا در «خطاهایی که خواهی دید».

</details>

<details>
<summary>این چه چاپ می‌کند؟

```rust
let scores = [3, 7, 2, 9];
println!("{:?}", scores.iter().find(|&&s| s > 5));
```
</summary>

```text
Some(7)
```

`.find()` اولین آیتمی که شرط را برآورده می‌کند برمی‌گرداند، نه بزرگ‌ترینشان — ۷ قبل از ۹ می‌آید.

</details>

<details>
<summary>این چه چاپ می‌کند؟

```rust
let unique: std::collections::HashSet<char> = "hello".chars().collect();
println!("{}", unique.len());
```
</summary>

```text
4
```

`"hello"` پنج کاراکتر دارد، ولی `'l'` دوبار تکرار شده؛ `HashSet` فقط مقدارهایِ متمایز را نگه می‌دارد: `h`، `e`، `l`، `o`.

</details>

<details>
<summary>این کامپایل می‌شود؟

```rust
let values = vec![1.0, 5.5, 2.3];
let biggest = values.iter().max();
```
</summary>

نه. `f64` صفتِ `Ord` را ندارد (به‌خاطرِ `NaN`)، و `.max()` بدونِ آرگومان به `Ord` نیاز دارد. کدِ خطا `E0277` است؛ کاملِ ماجرا در «خطاهایی که خواهی دید».

</details>

<details>
<summary>این چه چاپ می‌کند؟

```rust
let inputs = ["10", "abc", "30"];
let result: Result<Vec<i32>, _> = inputs.iter().map(|s| s.parse::<i32>()).collect();
match result {
    Ok(values) => println!("ok: {values:?}"),
    Err(_) => println!("err"),
}
```
</summary>

```text
err
```

`"abc"` پارس نمی‌شود، پس کلِ `.collect()` یک `Err` برمی‌گرداند — حتی با اینکه `"10"` قبلش و `"30"` بعدش هردو معتبرند.

</details>

### تعمیر

هر سه مثالِ خراب را درست کن:

۱. `examples/07-collect-ambiguous-type.rs` — یک نوع‌نویسی یا توربوفیش اضافه کن تا `.collect()` بداند باید چه بسازد.
۲. `examples/08-collect-result-into-vec-directly.rs` — نوعِ `parsed` را از `Vec<i32>` به `Result<Vec<i32>, _>` عوض کن.
۳. `examples/09-max-on-floats.rs` — نوعِ `values` را صریح `Vec<f64>` کن، و `.max()` را با `.max_by(|a, b| a.total_cmp(b))` عوض کن.

### پیاده‌سازی

پنج تابع در `src/lib.rs`:

```sh
cargo test -p p2-02-03-consuming-and-collecting
```

هرکدام را می‌شود با یک `.collect()` (به‌علاوه‌ی هر آداپتوری که لازم دارد) نوشت — بدونِ حلقه‌ی دستی، بدونِ `HashMap::new()` به‌علاوه‌ی یک حلقه‌ی `.insert()`. آخری، `parse_all`، همان تابعی است که در [۱.۶.۳](../../../phase1-fundamentals/06-absence-and-failure/03-result-and-question-mark/README.fa.md) با حلقه و `?` نوشتی — همان امضا، همان مشخصات؛ این‌بار یک‌خطی بنویسش.

### بساز

یک `pub fn` بنویس که رویِ چیزی که خودت انتخاب می‌کنی کار کند — امتیازهایِ یک لیستِ انیمه، خط‌هایِ یک فایلِ لاگ، هرچیزی — و هر دویِ این‌ها را به‌کار ببرد: (۱) `.collect()` به یکی از نوع‌مقصدهایِ امروز، و (۲) دستِ‌کم یکی از مصرف‌کننده‌هایِ دیگر (`.sum()`، `.count()`، `.min()`/`.max()`، `.find()`، `.any()`/`.all()`، `.last()`). شکلِ دقیقِ ورودی و خروجی را خودت انتخاب کن، در کامنتِ مستنداتِ تابع بنویسش، بعد دستِ‌کم دو تست اضافه کن.

### چالش (اختیاری)

**بخشِ یک.** مستنداتِ استانداردِ Rust را برایِ `FromIterator` بگرد. آیا `Option<T>` هم آن را پیاده کرده؟ حدس بزن `collect::<Option<Vec<T>>>()` رویِ یک ایتریتور از `Option` که وسطش یک `None` دارد چه برمی‌گرداند — بعد یک تابعِ کوچک بنویس و امتحانش کن تا حدست را بسنجی.

**بخشِ دو.** حالا `collect::<Result<HashSet<i32>, String>>()` را رویِ یک ایتریتور از `Result` امتحان کن. همان توقفِ زودهنگام، رویِ یک نوع‌مقصدِ دیگر هم کار می‌کند؟ چرا آره یا چرا نه؟

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| مصرف‌کننده (consuming adapter) | متدی که خط‌لوله را می‌کشد و یک نتیجه‌ی نهاییِ غیرِایتریتور می‌سازد | `.collect()`، `.sum()`، `.count()` و بقیه |
| `.collect()` | مصرف‌کننده‌یِ همه‌کاره؛ از رویِ همان ایتریتور می‌تواند چند نوعِ متفاوت بسازد | هرجا خروجیِ نهایی یک `Vec`، `String`، `HashMap`، `HashSet` یا `Result` است |
| صفتِ `FromIterator` | قراردادی که `.collect()` رویش جنریک است | هر نوعی که هدفِ `.collect()` باشد باید پیاده‌اش کرده باشد |
| توربوفیش / نوع‌نویسی | دو راهِ گفتن به `.collect()` چه بسازد | وقتی هدف از رویِ خودش قابلِ‌استنتاج نیست |
| `Result<Vec<T>, E>` از `.collect()` | جمع‌آوریِ یک ایتریتور از `Result` مستقیم به یک `Result` واحد | جایگزینِ حلقه‌ی دستی + `?` |
| توقفِ زودهنگام (short-circuiting) | متوقف‌شدن به‌محضِ معلوم‌شدنِ جوابِ نهایی | اولین `Err` در جمع‌آوریِ `Result`ها |

### الان می‌دانی

- `.collect()` مصرف‌کننده‌ی همه‌کاره است؛ چون از رویِ یک ایتریتور می‌تواند چند نوعِ متفاوت بسازد، باید با توربوفیش یا نوع‌نویسی بهش بگویی هدف چیست.
- همان ایتریتور را می‌شود به `Vec`، به `String` (از `char` یا `&str`، با چسباندنِ ساده)، به `HashMap` (از تاپل‌هایِ کلید-مقدار، آخرین تکرار برنده) یا به `HashSet` (با حذفِ خودکارِ تکراری‌ها) جمع کرد.
- `.sum()`، `.count()`، `.min()`/`.max()`، `.find()`، `.any()`/`.all()` و `.last()` مصرف‌کننده‌هایِ مستقیم‌ترند، برایِ وقتی خروجیِ نهایی یک عدد، یک `bool` یا یک آیتم است — نه یک مجموعه‌ی تازه.
- `Result<T, E>` هم مثلِ `Vec` و `HashMap` صفتِ `FromIterator` را پیاده کرده؛ یک ایتریتور از `Result` را می‌شود مستقیم به یک `Result<Vec<T>, E>` جمع کرد.
- آن جمع‌آوری توقفِ زودهنگام دارد: اولین `Err` همان‌جا جمع‌آوری را متوقف می‌کند و خودش کلِ جواب می‌شود؛ اگر همه `Ok` بودند، یک `Ok` واحد با یک `Vec` از همه‌ی مقدارهایِ بازشده می‌گیری.

### بعداً کامل‌تر می‌بینی

- **پیاده‌سازیِ `Iterator` و `IntoIterator` برایِ نوعِ خودت** — [۲.۲.۴](../04-implementing-iterator/README.fa.md)
- **تنبلی و کاراییِ زنجیره‌هایِ ایتریتور، در عمق** — [۲.۲.۵](../05-laziness-and-performance/README.fa.md)
- **صفتِ `Ord`/`PartialOrd` و خانواده‌ی مشتق‌هایِ استاندارد (همینی که در خطایِ `f64` دیدی)** — [۲.۳.۴](../../03-traits-and-generics/04-standard-derives-by-hand/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا `.collect()` به توربوفیش یا نوع‌نویسی نیاز دارد، وقتی `.sum()` یا `.count()` به هیچ‌کدام نیاز ندارند؟
- سه نوعِ متفاوت را نام ببر که بشود از رویِ یک ایتریتورِ `(کلمه, تعداد)` جمعشان کرد، و برایِ هرکدام بگو `.collect()` چه چیزِ متفاوتی می‌سازد.
- جمع‌آوریِ یک ایتریتور از `Result` را با کلمه‌ی «توقفِ زودهنگام» توضیح بده — دقیقاً چه‌وقت متوقف می‌شود و چه چیزی برمی‌گرداند؟
- چرا `.max()` رویِ یک ایتریتورِ `f64` کامپایل نمی‌شود، ولی رویِ یک ایتریتورِ `i32` می‌شود؟
- وقتی یک کلید در جمع‌آوریِ یک `HashMap` تکرار شود، کدام مقدار می‌ماند؟

---

## بیشتر

- [کتابِ Rust — پردازشِ یک سری از آیتم‌ها با ایتریتورها](https://doc.rust-lang.org/book/ch13-02-iterators.html) — همین زمین، بخشِ مصرف‌کننده‌ها.
- [مستنداتِ `std::iter::FromIterator`](https://doc.rust-lang.org/std/iter/trait.FromIterator.html) — فهرستِ نوع‌هایِ استانداردی که پیاده‌اش کرده‌اند.
- [مستنداتِ `std::result::Result`](https://doc.rust-lang.org/std/result/enum.Result.html) — همان‌جا که پیاده‌سازیِ `FromIterator` برایِ `Result` را می‌بینی، از زبانِ خودِ کتابخانه‌ی استاندارد.
- [مستنداتِ `std::iter::Iterator`](https://doc.rust-lang.org/std/iter/trait.Iterator.html) — فهرستِ کاملِ مصرف‌کننده‌ها؛ خیلی بیشتر از نُه‌تایی که امروز دیدی.

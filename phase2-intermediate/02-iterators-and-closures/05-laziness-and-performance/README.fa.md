# ۲.۲.۵ — تنبلی و کاراییِ ایتریتورها

## در یک نگاه

بعد از این درس می‌توانی:

- ثابت کنی — با شمردن، نه با حرفِ کسی — که یک زنجیره‌ی چندمرحله‌ای فقط وقتی مصرفش کنی کار می‌کند، و هر عنصر یکی‌یکی از **کلِ** زنجیره رد می‌شود، نه اینکه هر مرحله یک‌بار از کلِ داده رد بشود.
- تله‌ی یک `.collect()` که سهواً وسطِ یک زنجیره جا مانده را در کدِ خودت تشخیص بدهی، و توضیح بدهی دقیقاً چه چیزی این کار را گران می‌کند.
- یک ایتریتورِ بی‌نهایت (`repeat`، `.cycle()`، `successors`) بسازی، و با اطمینان بگویی چرا چنین چیزی اصلاً فقط زیرِ یک مدلِ تنبل ممکن است وجود داشته باشد.

**زمان:** حدود ۶۵ دقیقه · **پیش‌نیاز:** [۲.۲.۴ — پیاده‌سازیِ `Iterator` و `IntoIterator` برای نوعِ خودت](../04-implementing-iterator/README.fa.md)، و به‌طورِ خاص [۲.۲.۲ — آداپتورهای Iterator](../02-iterator-adapters/README.fa.md)

---

## چرا اهمیت دارد

توی [۲.۲.۲](../02-iterator-adapters/README.fa.md) یک جمله از کنارت رد شد که احتمالاً وزنش را واقعاً حس نکردی: «آداپتورها تنبل‌اند؛ تا وقتی مصرفشان نکنی هیچ اتفاقی نمی‌افتد.» آن جمله را باور کردی — شاید حتی خروجی‌اش را هم دیدی — ولی هیچ‌وقت مجبور نشدی *ثابتش* کنی، و هیچ‌وقت ندیدی این «تنبلی» دقیقاً چه دری را برایت باز می‌کند. این درس همان قرض را صاف می‌کند.

سه چیز قرار است اتفاق بیفتد.

اول، ادعای تنبلی را با شمردن ثابت می‌کنیم، نه با حرف. یک زنجیره از یک منبعِ نامحدود می‌سازیم، رویش `.take(3)` می‌گذاریم، و دقیقاً می‌شماریم هر مرحله چند بار واقعاً اجرا شد. جوابی که می‌بینی دقیقاً همان چیزی است که یک حلقه‌ی دستیِ معادل هم انجام می‌داد — نه بیشتر، نه کمتر. همین است که باعث می‌شود بگویند زنجیره‌های ایتریتور «انتزاعِ رایگان» (zero-cost abstraction) هستند: نوشتنشان به شکلِ تعریفی، هیچ هزینه‌ی اجراییِ اضافه‌ای رویِ کدِ تولیدشده نمی‌گذارد.

دوم، همین تنبلی یک تله هم دارد. یک `.collect()` که سهواً وسطِ یک زنجیره جا مانده — نه در انتهایش — دقیقاً همان چیزی را که تنبلی به‌ات می‌داد از دستت می‌گیرد: یک پاسِ کامل و یک تخصیصِ واقعی که هیچ‌وقت لازم نبود.

سوم — و اینجا همه‌چیز کنارِ هم می‌نشیند — بدونِ همین تنبلی، اصلاً نمی‌شد ایتریتوری نوشت که هرگز تمام نمی‌شود. یک `Vec` بی‌نهایت را نمی‌شود ساخت؛ اما یک *برنامه* که تا وقتی چیزِ محدودی ازش نخواهی هیچ کاری نمی‌کند را می‌شود ساخت. `std::iter::repeat`، `.cycle()` و `std::iter::successors` — هر سه‌شان مستقیماً محصولِ همین یک ایده‌اند، و بدونِ آن اصلاً معنا نداشتند.

---

## مفهوم

### از ساخته‌شدن تا مصرف‌شدن: کجا واقعاً کار انجام می‌شود

یک زنجیره بساز که هر مرحله‌اش، وسطِ کارش، یک خط چاپ کند — بعد فقط بسازش، بدونِ اینکه مصرفش کنی:

```rust
let chain = vec![1, 2, 3, 4, 5]
    .into_iter()
    .map(|n| {
        println!("  map saw {n}");
        n * 2
    })
    .filter(|n| {
        println!("  filter saw {n}");
        n % 4 == 0
    });
println!("chain built — nothing printed above this line from map/filter");
```

```text
chain built — nothing printed above this line from map/filter
```

همین. نه «map saw 1»ای، نه «filter saw...»ای. `chain` یک مقدار است — یک ساختارِ داده‌ی کوچک که می‌گوید «یک `vec![1,2,3,4,5]`، رویش این کلوژر، بعد آن یکی» — نه چیزی که از قبل *اجرا* شده باشد. حالا مصرفش کن:

```rust
let result: Vec<i32> = chain.collect();
println!("result: {result:?}");
```

```text
  map saw 1
  filter saw 2
  map saw 2
  filter saw 4
  map saw 3
  filter saw 6
  map saw 4
  filter saw 8
  map saw 5
  filter saw 10
result: [4, 8]
```

فقط الان، سرِ صدا زدنِ `.collect()`، کلوژرها واقعاً اجرا شدند. و به ترتیبِ چاپ‌ها دقت کن: «map saw 1» و بعدش بلافاصله «filter saw 2» — نه اینکه اول همه‌ی پنج‌تا از `map` رد بشوند و بعد همه‌ی پنج‌تا از `filter`. این دقیقاً همان چیزی است که زیربخشِ بعدی رویش دقیق‌تر می‌شود.

### هر عنصر کلِ زنجیره را طی می‌کند، نه هر مرحله کلِ داده را

حالا همین ایده را رویِ یک منبعِ **نامحدود** امتحان کن — یک بازه‌ی بازِ `1..` که هیچ سقفی ندارد — و رویش `.take(3)` بگذار. هر کلوژر یک شمارنده هم دارد:

```rust
let mut map_calls = 0u32;
let mut filter_calls = 0u32;
let chain_result: Vec<i32> = (1..)
    .map(|n| {
        map_calls += 1;
        n * 2
    })
    .filter(|n| {
        filter_calls += 1;
        n % 3 == 0
    })
    .take(3)
    .collect();
println!("chain result: {chain_result:?}");
println!("chain: map ran {map_calls} times, filter ran {filter_calls} times");
```

```text
chain result: [6, 12, 18]
chain: map ran 9 times, filter ran 9 times
```

منبع نامحدود بود — بی‌نهایت عدد داشت که می‌توانست بدهد — و با این‌همه، `map` فقط ۹ بار اجرا شد، نه یک بار بیشتر. این تنها زمانی معنا دارد که ارزیابی واقعاً **تقاضامحور (demand-driven)** باشد: `.collect()` از `.take(3)` یک مقدار می‌خواهد، `.take(3)` از `.filter(...)` یک مقدار می‌خواهد، `.filter(...)` از `.map(...)` یک مقدار می‌خواهد، و `.map(...)` از `(1..)` یک مقدار می‌خواهد — فقط همین یک عدد بالا می‌آید، از کلِ زنجیره رد می‌شود، و اگر `filter` قبولش نکرد، دوباره از اول: عددِ بعدی. هیچ‌کس هیچ‌وقت «همه‌ی مقدارها را از `map` رد کن، بعد همه را بده به `filter`» نگفت — چون اصلاً همچین چیزی وجود ندارد.

همین‌جاست که می‌شود اسمِ دقیقِ این رفتار را گذاشت: **ایتریتورِ تنبل (lazy iterator)**، همان چیزی که در [۲.۲.۲](../02-iterator-adapters/README.fa.md) گذرا دیدی، حالا شمرده‌شده و ثابت‌شده. با جفتِ مشتاق/تنبل (eager/lazy) که از ترکیب‌گرهای `Option` می‌شناسی فرق دارد — آنجا حرف روی *یک آرگومان* بود (`.unwrap_or(x)` در برابرِ `.unwrap_or_else(|| x)`)؛ اینجا حرف روی *یک زنجیره‌ی کامل* است: کل خط‌لوله فقط عنصر‌به‌عنصر، و فقط به تقاضا، حرکت می‌کند.

### همان مقدار کار، فقط با اسمی دیگر: مقایسه با یک حلقه‌ی دستی

اگر شک داری که این ۹ بار یک عددِ خاصِ زنجیره‌هاست، همان کار را با یک حلقه‌ی دستی بنویس — بدونِ هیچ آداپتوری:

```rust
let mut n = 1i32;
let mut loop_map_calls = 0u32;
let mut loop_filter_calls = 0u32;
let mut loop_result = Vec::new();
while loop_result.len() < 3 {
    loop_map_calls += 1;
    let doubled = n * 2;
    loop_filter_calls += 1;
    if doubled % 3 == 0 {
        loop_result.push(doubled);
    }
    n += 1;
}
println!("loop result:  {loop_result:?}");
println!("loop:  map ran {loop_map_calls} times, filter ran {loop_filter_calls} times");
```

```text
loop result:  [6, 12, 18]
loop:  map ran 9 times, filter ran 9 times
```

عینِ هم. همان نتیجه، همان ۹ بار برای «دوبرابرکردن»، همان ۹ بار برای «چک‌کردنِ بخش‌پذیری». زنجیره‌ی ایتریتور کارِ اضافه‌ای انجام نداد، و کارِ کمتری هم انجام نداد — فقط همین حلقه را با اسمِ دو مرحله‌اش («دوبرابر کن»، «فقط مضربِ ۳ را نگه دار») بیان کرد، به‌جایِ نوشتنِ دستیِ `n`، `while` و شمارنده. این دقیقاً همان چیزی است که منظورِ «انتزاعِ رایگان» است: نسخه‌ی تعریفی، هیچ هزینه‌ی اجراییِ اضافه‌ای بابتِ خواناتر بودنش نمی‌پردازد.

### چرا این همیشه درست است: یک آداپتور فقط یک `next()`ِ دیگر است

این رفتار شانسی نیست — دقیقاً همان چیزی است که [۲.۲.۴](../04-implementing-iterator/README.fa.md) به‌ات یاد داد، فقط این‌بار از زاویه‌ی دیگر. وقتی خودت `Iterator` را برایِ یک نوع پیاده‌سازی کردی، هر چیزی که بعدش می‌آمد — یک `for`، یک `.collect()`، یک `.take()` — فقط یک کار می‌کرد: پشتِ سرِ هم `next()` را صدا می‌زد و می‌پرسید «بعدی چیه؟». `.map()` و `.filter()` هیچ جادویی ندارند: خودشان هم فقط یک `struct` هستند که یک ایتریتورِ دیگر را دورِ خودشان نگه می‌دارند، و `next()`ِ خودشان همین کار را می‌کند — از ایتریتورِ درونی `next()` می‌خواهند، و اگر لازم بود رویِ نتیجه‌اش کاری انجام می‌دهند.

برایِ اینکه این را با دستِ خودت ببینی، همان کاری را که در [۲.۲.۴](../04-implementing-iterator/README.fa.md) کردی دوباره انجام بده — یک `struct` معمولی، با `impl Iterator` دستی، به‌علاوه‌ی یک شمارنده:

```rust
struct Doubling {
    inner: std::ops::Range<i32>,
    calls: u32,
}

impl Iterator for Doubling {
    type Item = i32;

    fn next(&mut self) -> Option<i32> {
        self.calls += 1;
        self.inner.next().map(|n| n * 2)
    }
}
```

`Doubling` هیچ چیزِ خاصی از کتابخانه‌ی استاندارد قرض نگرفته — دقیقاً همان الگویی است که خودت نوشتی، فقط این‌بار یک `calls` هم دارد که هر بار `next()` واقعاً اجرا شود یکی بالا می‌رود. حالا رویش `.take(3)` بگذار — با `.by_ref()`، که یک قرضِ موقت از ایتریتور می‌گیرد تا خودِ `doubling` بعدش هنوز قابلِ‌استفاده بماند:

```rust
let mut doubling = Doubling {
    inner: 0..1_000_000,
    calls: 0,
};
println!("constructed — calls so far: {}", doubling.calls);

let first_three: Vec<i32> = doubling.by_ref().take(3).collect();
println!("first_three: {first_three:?}");
println!("next() actually ran {} times", doubling.calls);
```

```text
constructed — calls so far: 0
first_three: [0, 2, 4]
next() actually ran 3 times
```

بازه‌ی داخلی‌اش یک میلیون تا عدد داشت. `next()` فقط سه بار اجرا شد. `Doubling` همان‌قدر تنبل است که `.map()` است — نه چون کتابخانه‌ی استاندارد یک استثنا برایِ نوعِ خودت قائل شده، بلکه چون **هیچ راهِ دیگری برایِ کار کردنِ `next()` وجود ندارد**: تا کسی صدایش نزند، چیزی حرکت نمی‌کند. این دقیقاً همان مکانیزمی است که زیرِ `.map()`، `.filter()`، و هر آداپتورِ دیگری که تا الان استفاده کرده‌ای نشسته.

```senpai-visual
{"kind":"concept","labels":["take() می‌خواهد","filter می‌خواهد","map می‌خواهد","range می‌سازد","map دوبرابر می‌کند","filter می‌سنجد"]}
```

نکته‌ای که این‌جا جا می‌افتد اینکه پیامِ تقاضا از پایین‌ترین مصرف‌کننده شروع می‌شود و *پایین* می‌رود — تا برسد به منبعِ اصلی — و بعد مقدار، مرحله‌به‌مرحله، *بالا* می‌آید. هیچ مرحله‌ای منتظرِ «همه»ی چیزی نمی‌ماند؛ فقط منتظرِ یک `next()` از پایینِ خودش می‌ماند.

(یک نکته‌ی کناری، برایِ بعد: `Doubling` این‌جا فقط دورِ یک `Range<i32>` مشخص پیچیده شده، نه دورِ *هر* ایتریتوری. نوشتنِ نسخه‌ای که دورِ هر ایتریتوری کار کند دقیقاً همان چیزی است که جنریک‌ها برایش ساخته شده‌اند — در [۲.۳.۲](../../03-traits-and-generics/02-generic-functions-and-structs/README.fa.md).)

### تله: `collect()`ی که وسطِ زنجیره جا مانده

حالا سراغِ نقطه‌ی مقابلِ همین قدرت برو. یک منبعِ صد-هزارتایی بساز، هر عنصر را دوبرابر کن، ولی این‌بار — سهواً — همین‌جا `.collect()` را صدا بزن:

```rust
const SOURCE_LEN: i32 = 100_000;

let mut before_map_calls = 0u32;
let mut before_filter_calls = 0u32;
let doubled: Vec<i32> = (1..=SOURCE_LEN)
    .map(|n| {
        before_map_calls += 1;
        n * 2
    })
    .collect();
println!(
    "doubled: {} elements (map ran {before_map_calls} times)",
    doubled.len()
);
```

```text
doubled: 100000 elements (map ran 100000 times)
```

هنوز هیچ اتفاقِ عجیبی نیفتاده — این `.collect()` عمدی است، یک `Vec` کامل از صد هزار عنصر ساخته، و `map` هم دقیقاً به همان تعداد اجرا شده، چون قرار بود اجرا شود. مشکل از خطِ *بعدی* شروع می‌شود، وقتی از رویِ همین `doubled` فقط دنبالِ سه‌تا مضربِ ۳ می‌گردی:

```rust
let first_three: Vec<i32> = doubled
    .into_iter()
    .filter(|n| {
        before_filter_calls += 1;
        n % 3 == 0
    })
    .take(3)
    .collect();
println!("first_three: {first_three:?} (filter ran {before_filter_calls} times)");
```

```text
first_three: [6, 12, 18] (filter ran 9 times)
```

`filter` فقط ۹ بار اجرا شد — همان‌قدر تنبل که همیشه بود؛ زنجیره‌ی دومی هیچ مشکلی ندارد. **مشکل، خطِ اولی است که رویِ `doubled` نشسته.** آن `.collect()` اول، هیچ ربطی به «سه‌تا مضربِ ۳ پیدا کن» نداشت — فقط یک قدمِ میانی بود که خودش را کاملاً غیرِتنبل کرده: صد هزار بار `map` را اجرا کرد، یک `Vec` صد-هزارتایی واقعاً تخصیص داد، و بعد از همه‌ی آن، دوباره از اولِ همان `Vec` شروع کرد به گشتن — با اینکه فقط ۹ عنصرِ اولش لازم بود.

حالا همان کار را بدونِ آن `.collect()`ِ میانی بنویس — یک زنجیره‌ی پیوسته، از اول تا آخر:

```rust
let mut after_map_calls = 0u32;
let after: Vec<i32> = (1..=SOURCE_LEN)
    .map(|n| {
        after_map_calls += 1;
        n * 2
    })
    .filter(|n| n % 3 == 0)
    .take(3)
    .collect();
println!("after: {after:?} (map ran {after_map_calls} times)");
```

```text
after: [6, 12, 18] (map ran 9 times)
```

همان جواب، ۶، ۱۲، ۱۸ — ولی `map` این‌بار فقط ۹ بار اجرا شد، نه صد هزار بار، و هیچ `Vec`ِ میانی‌ای — نه `doubled`ی، نه چیزِ دیگری — اصلاً به‌وجود نیامد. تنها فرقِ این دو نسخه یک `.collect()`ِ اضافه در وسط بود؛ همان یک خط، صد هزار برابر کارِ بیشتر خرید. راهِ حلش هم عمداً ساده است: **`.collect()` را فقط در انتهایِ زنجیره‌ای که واقعاً می‌خواهی همان‌جا تمام شود بگذار — نه هرجا که "به یک `Vec` نیاز داری".**

### پاداشِ مستقیم: ایتریتورهایی که هرگز تمام نمی‌شوند

حالا برگرد به همان `(1..)`ای که چند زیربخش قبل استفاده کردی — یک بازه‌ی باز، بدونِ کرانِ بالا. چیزِ عجیبی در موردش نگفتیم، ولی واقعاً عجیب است: زیرِ یک مدلِ **مشتاق (eager)** — جایی که هر آداپتور بلافاصله کارش را تمام می‌کند — نوشتنِ `(1..)` اصلاً امکان‌پذیر نبود. یک `Vec` با بی‌نهایت عنصر را نمی‌شود ساخت؛ حافظه تمام می‌شود، برنامه هیچ‌وقت به خطِ بعدی نمی‌رسد. اینجا کار می‌کند چون `(1..)` فقط یک *توصیف* است — «هر بار یکی بیشتر از قبلی» — و تا کسی، از طریقِ `.take()` یا هر مصرف‌کننده‌ی محدودِ دیگری، واقعاً یک مقدار نخواهد، هیچ عددی تولید نمی‌شود.

کتابخانه‌ی استاندارد همین ایده را در چند شکلِ آماده هم می‌دهد:

```rust
let repeated: Vec<&str> = std::iter::repeat("frieren").take(4).collect();
println!("{repeated:?}");

let pattern = [1, 2, 3];
let cycled: Vec<i32> = pattern.iter().copied().cycle().take(8).collect();
println!("{cycled:?}");

let empty: Vec<i32> = Vec::new();
let cycled_empty: Vec<i32> = empty.iter().copied().cycle().take(5).collect();
println!("{cycled_empty:?}");
```

```text
["frieren", "frieren", "frieren", "frieren"]
[1, 2, 3, 1, 2, 3, 1, 2]
[]
```

`std::iter::repeat(x)` همان مقدار را بی‌نهایت بار تکرار می‌کند. `.cycle()` یک ایتریتورِ **محدود** را می‌گیرد و از اولش، بی‌نهایت بار، دوباره شروعش می‌کند — با یک استثنایِ منطقی: رویِ یک منبعِ خالی، `.cycle()` هم خودش خالی می‌ماند (چیزی برایِ تکرار نیست، پس چیزی هم برنمی‌گردد؛ گیر نمی‌کند). هیچ‌کدام این‌ها یک لحظه هم زودتر از آنچه `.take(...)` بخواهد کار نمی‌کنند — دقیقاً همان چیزی که این کدها را اصلاً *قابلِ نوشتن* می‌کند.

سومین ابزار، وقتی مقدارِ بعدی از رویِ مقدارِ قبلی ساخته می‌شود، بیشتر به‌کارت می‌آید:

```rust
let powers: Vec<u32> = std::iter::successors(Some(1u32), |&x| Some(x * 2))
    .take(6)
    .collect();
println!("{powers:?}");
```

```text
[1, 2, 4, 8, 16, 32]
```

`std::iter::successors(first, next)` با `first` شروع می‌کند و هر بار `next` را رویِ آخرین مقدار صدا می‌زند تا مقدارِ بعدی را بسازد؛ وقتی `next` یک بار `None` برگرداند، توالی همان‌جا تمام می‌شود — ولی اینجا هیچ‌وقت `None` برنمی‌گردانَد، پس اگر `.take(6)` نبود، تا ابد ادامه می‌داد. این را یک **مولد (generator)** فکر کن: توصیفِ «چطور مقدارِ بعدی را از مقدارِ فعلی بسازم»، نه فهرستی که از قبل نوشته شده باشد.

---

## دست‌به‌کد

```sh
cargo run -p p2-02-05-laziness-and-performance --example 01-nothing-runs-until-you-consume
cargo run -p p2-02-05-laziness-and-performance --example 02-demand-driven-and-loop-equivalence
cargo run -p p2-02-05-laziness-and-performance --example 03-mid-chain-collect-trap
cargo run -p p2-02-05-laziness-and-performance --example 04-infinite-iterator-family
cargo run -p p2-02-05-laziness-and-performance --example 05-custom-iterator-is-lazy-too
```

بعد دوتای خراب:

```sh
cargo run -p p2-02-05-laziness-and-performance --example 06-len-on-infinite-iterator --features broken
cargo run -p p2-02-05-laziness-and-performance --example 07-cycle-needs-clone --features broken
```

بعد این‌ها را امتحان کن:

۱. در `02-demand-driven-and-loop-equivalence`، `.take(3)` را به `.take(1)` عوض کن. قبل از اجرا حدس بزن `map` و `filter` چند بار اجرا می‌شوند و نتیجه چیست — بعد بررسی کن.
۲. در `03-mid-chain-collect-trap`، `SOURCE_LEN` را از `100_000` به `1_000_000` عوض کن. عددِ `map ran ... times`ِ نسخه‌ی BEFORE چطور عوض می‌شود؟ عددِ نسخه‌ی AFTER چطور؟
۳. در `04-infinite-iterator-family`، `pattern.iter().copied().cycle().take(8)` را به `.take(2)` عوض کن. آیا اصلاً لازم بود `pattern` را دوباره از اول شروع کند؟

---

## خطاهایی که خواهی دید

### هیچ خطایی نیست — `.collect()`ِ وسطِ زنجیره کار می‌کند، فقط گران است

همان چیزی که در «تله» دیدی، به‌عنوانِ یک خطا: کدِ نسخه‌ی BEFORE کاملاً کامپایل می‌شود، کاملاً اجرا می‌شود، و جوابِ کاملاً درستی هم می‌دهد — `[6, 12, 18]`، عینِ نسخه‌ی AFTER.

**کامپایلر به چه اعتراض دارد:** به هیچ‌چیز. هیچ‌جا نگفته «این `.collect()` زیادی است» — از نظرِ نوع، کاملاً معتبر است: یک `Vec<i32>` ساختی، بعد رویِ آن `Vec` یک زنجیره‌ی دیگر زدی. کامپایلر نمی‌داند تو *قصد* داشتی زنجیره ادامه پیدا کند؛ فقط دو تکه کدِ درست می‌بیند.

**راه‌حل:** آن `.collect()`ِ میانی را بردار و همه را یک زنجیره کن:

```rust
let after: Vec<i32> = (1..=100_000)
    .map(|n| n * 2)
    .filter(|n| n % 3 == 0)
    .take(3)
    .collect();
```

**چرا این راه‌حل است:** با یک `.collect()`، `map` فقط ۹ بار اجرا می‌شود، نه صد هزار بار — دقیقاً همان عددی که در «تله» با شمردنِ واقعی دیدی. هیچ نشانه‌ی ظاهری‌ای این تفاوت را نشان نمی‌دهد؛ کدِ هر دو نسخه تقریباً یک شکل است، خروجی‌شان کاملاً یکسان است. تنها راهِ گرفتنِ این دسته از باگ، دانستنِ این قاعده است: **`.collect()` را فقط جایی بگذار که واقعاً می‌خواهی زنجیره همان‌جا تمام شود، نه وسطِ راه.**

### `E0599` — رویِ یک ایتریتورِ بی‌نهایت، `.len()` معنا ندارد

```text
error[E0599]: no method named `len` found for struct `std::iter::Repeat<A>` in the current scope
    --> phase2-intermediate\02-iterators-and-closures\05-laziness-and-performance\examples\06-len-on-infinite-iterator.rs:11:28
     |
  11 |     println!("{}", forever.len());
     |                            ^^^
     |
help: there is a method `le` with a similar name, but with different arguments
    --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\iter\traits\iterator.rs:3977:5
     |
3977 | /     fn le<I>(self, other: I) -> bool
3978 | |     where
3979 | |         I: IntoIterator,
3980 | |         Self::Item: PartialOrd<I::Item>,
3981 | |         Self: Sized,
     | |____________________^

For more information about this error, try `rustc --explain E0599`.
```

**کامپایلر به چه اعتراض دارد:** `.len()` اصلاً روی خودِ `Iterator` تعریف نشده — فقط صفتِ `ExactSizeIterator` آن را دارد، برایِ ایتریتورهایی که از قبل دقیقاً می‌دانند چند عنصرِ باقی‌مانده دارند (مثلِ یک `Range` یا `.iter()`ِ یک `Vec`). `std::iter::Repeat` هرگز تمام نمی‌شود، پس چیزی به اسمِ «طولِ باقی‌مانده» ندارد که گزارش بدهد — و اصلاً `ExactSizeIterator` را پیاده‌سازی نمی‌کند. کامپایلر حتی یک متدِ شبیه («`le`») پیشنهاد داده، دقیقاً چون هیچ متدِ واقعی‌ای به این اسم پیدا نکرده.

**راه‌حل:** اگر واقعاً به یک عدد نیاز داری، اول محدودش کن:

```rust
let forever = std::iter::repeat(1);
let taken: Vec<i32> = forever.take(5).collect();
println!("{}", taken.len());
```

```text
5
```

**چرا این راه‌حل است:** «چند تا مانده؟» برایِ چیزی که هرگز تمام نمی‌شود، اصلاً یک سؤالِ معنادار نیست — کامپایلر هم دقیقاً به همین دلیل رد کرد، نه به‌خاطرِ یک محدودیتِ الکی. سؤالِ معناداری که می‌شود پرسید این است: «از این *تکه‌ای* که برداشتم، چند تاست؟» — و آن تکه، بعد از `.take(5)`، دیگر یک `Vec`ِ معمولی و کاملاً محدود است.

### `E0277` و `E0599` — `.cycle()` بدونِ `Clone` کار نمی‌کند

```text
error[E0277]: the trait bound `Doubling: Clone` is not satisfied
    --> phase2-intermediate\02-iterators-and-closures\05-laziness-and-performance\examples\07-cycle-needs-clone.rs:30:37
     |
  30 |     let cycled: Vec<i32> = doubling.cycle().take(5).collect();
     |                                     ^^^^^ the trait `Clone` is not implemented for `Doubling`
     |
note: required by a bound in `cycle`
    --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\iter\traits\iterator.rs:3592:23
     |
3590 |     fn cycle(self) -> Cycle<Self>
     |        ----- required by a bound in this associated function
3591 |     where
3592 |         Self: Sized + [const] Clone,
     |                       ^^^^^^^^^^^^^ required by this bound in `Iterator::cycle`
help: consider annotating `Doubling` with `#[derive(Clone)]`
     |
  11 + #[derive(Clone)]
  12 | struct Doubling {
     |

error[E0599]: the method `take` exists for struct `Cycle<Doubling>`, but its trait bounds were not satisfied
  --> phase2-intermediate\02-iterators-and-closures\05-laziness-and-performance\examples\07-cycle-needs-clone.rs:30:45
   |
11 | struct Doubling {
   | --------------- doesn't satisfy `Doubling: Clone`
...
30 |     let cycled: Vec<i32> = doubling.cycle().take(5).collect();
   |                                             ^^^^ method cannot be called on `Cycle<Doubling>` due to unsatisfied trait bounds
   |
  ::: C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\iter\adapters\cycle.rs:15:1
   |
15 | pub struct Cycle<I> {
   | ------------------- doesn't satisfy `Cycle<Doubling>: Iterator`
   |
   = note: the following trait bounds were not satisfied:
           `Doubling: Clone`
           which is required by `Cycle<Doubling>: Iterator`
           `Cycle<Doubling>: Iterator`
           which is required by `&mut Cycle<Doubling>: Iterator`
help: consider annotating `Doubling` with `#[derive(Clone)]`
   |
11 + #[derive(Clone)]
12 | struct Doubling {
   |

Some errors have detailed explanations: E0277, E0599.
For more information about an error, try `rustc --explain E0277`.
```

**کامپایلر به چه اعتراض دارد:** دو خطا، ولی یک ریشه. `.cycle()` وقتی ایتریتورِ اصلی تمام شد، باید بتواند از نو، از همان نقطه‌ی شروع، دوباره‌اش کند — و تنها راهش این است که یک کپیِ تازه از حالتِ اولیه نگه دارد. برایِ همین، امضایِ خودِ `cycle` می‌گوید `Self: Clone`. `Doubling` هیچ‌وقت `#[derive(Clone)]` نگرفت، پس این شرط برقرار نیست؛ خطایِ اول همین را می‌گوید. خطایِ دوم فقط زنجیره‌ای از همان مشکل است: چون `Cycle<Doubling>` اصلاً `Iterator` نیست (دقیقاً به‌خاطرِ همان شرطِ ناقص)، `.take()` هم رویش پیدا نمی‌شود — رفعِ خطایِ اول، دومی را هم با خودش می‌برد.

**راه‌حل:** دقیقاً همان چیزی که کامپایلر پیشنهاد داد:

```rust
#[derive(Clone)]
struct Doubling {
    inner: std::ops::Range<i32>,
    calls: u32,
}
```

```text
[0, 2, 4, 0, 2]
```

**چرا این راه‌حل است:** هر دو فیلدِ `Doubling` — یک `Range<i32>` و یک `u32` — خودشان از قبل `Clone`اند، پس `#[derive(Clone)]` هیچ کارِ اضافه‌ای لازم ندارد؛ فقط به کامپایلر اجازه می‌دهد همان چیزی را که خودش پیشنهاد داد بسازد. حالا `.cycle()` هر بار که `Doubling` تمام شود، یک کپیِ تازه از حالتِ *اولیه*‌اش (`inner: 0..3, calls: 0`) می‌سازد و از نو شروع می‌کند — و همین چیزی است که `[0, 2, 4, 0, 2]` را توضیح می‌دهد: دور اول `0..3` را می‌دهد (دوبرابرشده: `0, 2, 4`)، دور دوم دوباره از `0..3` شروع می‌کند، نه از جایی که دورِ اول تمام شد.

---

## تمرین

### گرم‌کردن

<details>
<summary>این چه چاپ می‌کند؟</summary>

```rust
let v = vec![1, 2, 3];
let chain = v.iter().map(|n| {
    println!("mapping {n}");
    n * 2
});
println!("done building");
```

</details>

<details>
<summary>پاسخ</summary>

```text
done building
```

همین یک خط. `chain` فقط ساخته شد، هیچ‌وقت مصرف نشد — پس کلوژرِ داخلِ `.map()` اصلاً یک‌بار هم اجرا نشد. «mapping 1» و بقیه هرگز چاپ نمی‌شوند.

</details>

<details>
<summary>در همان زنجیره‌ی <code>(1..).map(...).filter(|n| n % 3 == 0)</code> که در «مفهوم» دیدی، اگر به‌جایِ <code>.take(3)</code> بنویسی <code>.take(1)</code>، نتیجه و تعدادِ اجراهای <code>map</code>/<code>filter</code> چه می‌شود؟</summary>

فکرت را قبل از خواندنِ پاسخ بنویس.

</details>

<details>
<summary>پاسخ</summary>

```text
result=[6] map=3 filter=3
```

اولین مضربِ ۳ که پیدا می‌شود همان `6` است (وقتی `n = 3`)، و همان لحظه `.take(1)` سیر می‌شود. هم `map` هم `filter` دقیقاً سه بار اجرا شده‌اند — همان الگویِ قبلی، فقط با یک هدفِ کوچک‌تر.

</details>

<details>
<summary><code>std::iter::repeat(1).len()</code> کامپایل می‌شود؟</summary>

فکرت را قبل از خواندنِ «خطاهایی که خواهی دید» بنویس.

</details>

<details>
<summary>پاسخ</summary>

نه — `E0599`. `.len()` فقط رویِ `ExactSizeIterator` تعریف شده، و `std::iter::Repeat` هرگز تمام نمی‌شود، پس آن صفت را پیاده‌سازی نمی‌کند.

</details>

<details>
<summary><code>[1, 2].iter().cycle().take(5).copied().collect::&lt;Vec&lt;i32&gt;&gt;()</code> چه چیزی برمی‌گرداند؟</summary>

فکرت را بنویس.

</details>

<details>
<summary>پاسخ</summary>

```text
[1, 2, 1, 2, 1]
```

`.cycle()` بعد از `2`، دوباره از `1` شروع می‌کند؛ `.take(5)` درست وسطِ دورِ سوم متوقفش می‌کند.

</details>

<details>
<summary>یک <code>struct</code> که <code>Iterator</code> را پیاده‌سازی کرده ولی <code>#[derive(Clone)]</code> ندارد، رویِ <code>.cycle()</code> کامپایل می‌شود؟</summary>

فکرت را بنویس.

</details>

<details>
<summary>پاسخ</summary>

نه. `.cycle()` نیاز دارد `Self: Clone` باشد — باید بتواند هر بار که تمام شد، یک کپیِ تازه از حالتِ اولیه بسازد. بدونِ `Clone`، `E0277` می‌گیری.

</details>

### تعمیر

هر دو مثالِ خراب را درست کن:

۱. `examples/06-len-on-infinite-iterator.rs` را طوری درست کن که کامپایل شود — بدونِ اینکه فرض کنی `forever` یک طولِ مشخص دارد. اگر واقعاً به یک عدد نیاز داری، اول محدودش کن.
۲. `examples/07-cycle-needs-clone.rs` را طوری درست کن که کامپایل شود و اجرا شود — با اضافه‌کردنِ دقیقاً همان چیزی که پیامِ کمکِ کامپایلر پیشنهاد داد.

و این یکی را هم — با اینکه نه پنیک می‌گیرد و نه رد می‌شود، همان‌قدر واقعی است:

۳. یک نسخه از `examples/03-mid-chain-collect-trap.rs` بنویس که نسخه‌ی BEFORE‌اش را هم به یک زنجیره‌ی پیوسته تبدیل کند — یعنی آن `.collect()`ِ اول را کاملاً حذف کن — و دوباره اجرا کن. عددِ `map ran ... times` چطور عوض می‌شود؟

### پیاده‌سازی

پنج تابع در `src/lib.rs`، هرکدام یکی از ابزارهایِ همین درس:

```sh
cargo test -p p2-02-05-laziness-and-performance
```

هیچ‌کدام نیاز به جنریک، `Box` یا نوعِ برگشتیِ `impl Trait` ندارند — همه رویِ نوع‌های ملموسی کار می‌کنند که از قبل می‌شناسی. کامنتِ مستنداتِ هر تابع دقیقاً می‌گوید چه چیزی برمی‌گرداند؛ چیزی را حدس نزن.

### بساز

یک `struct` بنویس که `Iterator` را پیاده‌سازی می‌کند — هر تبدیلِ ساده‌ای که خودت انتخاب کنی (سه‌برابر کردن، به‌توانِ‌دو رساندن، منفی کردن، تبدیل به `String`، هرچی) — با یک فیلدِ `calls: u32` که هر بار `next()` واقعاً اجرا شود یکی بالا برود. یک نمونه بساز، رویِ یک منبعِ بزرگ (چند صدهزار عنصر)، فقط چندتای اولش را با `.by_ref().take(n)` بگیر، و با چاپِ `calls` بعدش ثابت کن که فقط به همان تعداد که لازم بود اجرا شده — نه یکی بیشتر. در یک کامنت بنویس چرا این نتیجه، با توجه به این‌که `Iterator` چطور کار می‌کند، از قبل قابلِ‌پیش‌بینی بود.

### چالش (اختیاری)

`examples/02-demand-driven-and-loop-equivalence.rs` را طوری عوض کن که به‌جایِ شمردنِ صداها، با `std::time::Instant` زمانِ واقعیِ اجرا را هم برایِ نسخه‌ی زنجیره و هم برایِ نسخه‌ی حلقه اندازه بگیرد، رویِ یک `.take(n)` با `n` خیلی بزرگ‌تر (چند میلیون). آیا زمان‌ها همان‌قدر نزدیک‌اند که تعدادِ صداها بودند؟ اندازه‌گیریِ درست و تکرارپذیرِ این نوع کارایی، بدونِ نویزِ سیستم‌عامل، دقیقاً کاری است که [۲.۷.۵ — بنچمارک‌کردن با criterion](../../07-project-structure-and-testing/05-benchmarking-with-criterion/README.fa.md) یادت می‌دهد.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| ایتریتورِ تنبل (ارزیابیِ تقاضامحور) | آداپتور به‌تنهایی کاری نمی‌کند؛ هر عنصر یکی‌یکی، فقط وقتی چیزی پایین‌دست `.next()` بخواهد، از کلِ زنجیره عبور می‌کند | فهمِ اینکه چرا زنجیره‌های طولانی هزینه‌ی اضافه ندارند |
| انتزاعِ رایگان (zero-cost abstraction) | یک زنجیره‌ی ایتریتور دقیقاً به‌اندازه‌ی نسخه‌ی دستی‌اش کار انجام می‌دهد | تصمیم‌گیری بدونِ نگرانیِ بی‌جا از کارایی |
| ایتریتورِ بی‌نهایت | ایتریتوری بدونِ پایانِ معلوم؛ امن است چون هیچ‌چیز اجرا نمی‌شود تا چیزِ محدودی مثلِ `.take()` مقدار بخواهد | `std::iter::repeat`، `.cycle()`، بازه‌ی باز (`1..`) |
| مولد (generator) | شرحِ چگونگیِ ساختنِ مقدارِ بعدی از رویِ مقدارِ فعلی، نه فهرستی از پیش نوشته‌شده | همان چیزی که `std::iter::successors` است |
| `std::iter::successors` | مولدی که مقدارِ بعدی را از رویِ مقدارِ فعلی می‌سازد، تا `None` برگردد | دنباله‌هایی که قاعده‌شان معلوم است ولی طولشان نه |
| `.by_ref()` | قرضِ موقتِ یک ایتریتور، تا خودش بعدش هنوز قابلِ‌استفاده بماند | برداشتنِ چند عنصرِ اول، بدونِ از دست دادنِ بقیه |
| تله‌ی `.collect()`ِ میانی | یک `Vec`ِ کاملاً بی‌ربط که فقط برایِ عبور از یک قدمِ میانی ساخته می‌شود | تشخیصِ کدی که «کار می‌کند» ولی بی‌دلیل گران است |

### الان می‌دانی

- یک زنجیره‌ی آداپتور فقط یک مقدار است، نه چیزی که از قبل اجرا شده باشد؛ کار فقط سرِ صدا زدنِ یک متدِ مصرف‌کننده شروع می‌شود.
- از یک منبعِ نامحدود، فقط به تعدادی که یک `.take(n)` بخواهد عنصر واقعاً از کلِ زنجیره رد می‌شود — نه یک عنصرِ اضافه.
- زنجیره و حلقه‌ی دستیِ معادلش دقیقاً یک تعداد کار انجام می‌دهند؛ همین است که آن‌ها را «انتزاعِ رایگان» می‌کند.
- این تنبلی شانسی نیست: هر آداپتور فقط یک `next()`ِ دیگر است که از ایتریتورِ درونی‌اش `next()` می‌خواهد — دقیقاً همان مکانیزمی که در [۲.۲.۴](../04-implementing-iterator/README.fa.md) با دستِ خودت نوشتی.
- یک `.collect()` که سهواً وسطِ زنجیره جا مانده، هیچ خطایی نمی‌دهد — فقط یک `Vec`ِ کاملاً غیرِلازم می‌سازد و یک پاسِ کاملِ اضافه می‌خرد.
- `std::iter::repeat`، `.cycle()` و `std::iter::successors` فقط زیرِ همین مدلِ تنبل معنا دارند؛ زیرِ یک مدلِ مشتاق، نوشتنشان از اساس ممکن نبود.

### بعداً کامل‌تر می‌بینی

- **جنریک‌ها — نوشتنِ یک `Doubling` که دورِ هر ایتریتوری کار کند، نه فقط `Range<i32>`** — [۲.۳.۲ — توابع و ساختارهای جنریک](../../03-traits-and-generics/02-generic-functions-and-structs/README.fa.md)
- **`impl Trait` در جایگاهِ خروجی — برگرداندنِ خودِ ایتریتورِ تنبل از یک تابع، بدونِ `.collect()` کردنش** — [۲.۳.۷ — دیسپچِ استاتیک در برابرِ دینامیک](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.fa.md)
- **اندازه‌گیریِ دقیقِ کارایی، با `criterion`** — [۲.۷.۵ — بنچمارک‌کردن با criterion](../../07-project-structure-and-testing/05-benchmarking-with-criterion/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا ساختنِ یک زنجیره‌ی `.map().filter()` به‌تنهایی هیچ کلوژری را اجرا نمی‌کند؟
- وقتی `.take(3)` رویِ یک منبعِ نامحدود می‌گذاری، تقاضا از کجا شروع می‌شود و به کدام سمت حرکت می‌کند؟
- چرا یک زنجیره‌ی ایتریتور و حلقه‌ی دستیِ معادلش را «هم‌ارزِ کارایی» می‌دانیم؟
- چرا `Doubling`ی که خودت نوشتی، دقیقاً به همان دلیلِ `.map()` تنبل است؟
- یک `.collect()`ِ وسطِ زنجیره چه چیزی را از دستِ برنامه می‌گیرد، با اینکه هیچ خطایی هم نمی‌دهد؟
- چرا `std::iter::repeat(1)` زیرِ یک مدلِ مشتاق اصلاً نمی‌توانست وجود داشته باشد؟

---

## بیشتر

- [مستنداتِ `std::iter`](https://doc.rust-lang.org/std/iter/index.html) — فهرستِ کاملِ ابزارهایِ ساختِ ایتریتور، شاملِ `repeat`، `successors`، `once` و بقیه.
- [مستنداتِ صفتِ `Iterator`](https://doc.rust-lang.org/std/iter/trait.Iterator.html) — امضایِ خودِ `next()`، و اینکه `.cycle()`، `.take()` و بقیه دقیقاً چه چیزی رویِ `Self` می‌خواهند.
- [کتابِ Rust — مقایسه‌ی کاراییِ حلقه و ایتریتور](https://doc.rust-lang.org/book/ch13-04-performance.html) — نسخه‌ی رسمیِ همان ادعایی که در «همان مقدار کار، فقط با اسمی دیگر» با شمردن ثابتش کردی.
- [مستنداتِ `std::iter::Cycle`](https://doc.rust-lang.org/std/iter/struct.Cycle.html) — دقیقاً همان ساختاری که `.cycle()` برمی‌گرداند؛ همان‌جا نوشته چرا `Self: Clone` شرطش است.

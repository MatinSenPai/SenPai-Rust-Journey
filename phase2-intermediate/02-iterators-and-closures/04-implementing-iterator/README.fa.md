# ۲.۲.۴ — پیاده‌سازیِ `Iterator` و `IntoIterator` برای نوعِ خودت

## در یک نگاه

بعد از این درس می‌توانی:

- توضیح بدهی صفتِ `Iterator` دقیقاً به چه چیزی نیاز دارد — همان یک متدِ اجباری، `next(&mut self) -> Option<Self::Item>` — و چرا همان یکی کافی است تا هر آداپتور و مصرف‌کننده‌ای که [۲.۲.۲](../02-iterator-adapters/README.fa.md) و [۲.۲.۳](../03-consuming-and-collecting/README.fa.md) یادت دادند، روی نوعِ خودت هم رایگان کار کند.
- یک پیمایشگرِ سفارشیِ خودت را با حالتِ واقعی (state) بسازی — نوعی که هر بارِ `next()`اش، از رویِ داده‌ای که خودِ ساختار نگه می‌دارد، مقدارِ بعدی را حساب و همان داده را برایِ صدای بعدی جلو می‌برد.
- صفتِ `IntoIterator` را برایِ نوعِ خودت پیاده کنی تا `for x in your_value` قانونی شود، سه‌شکلیِ رایج‌اش (با مقدار / با ارجاعِ اشتراکی / با ارجاعِ تغییرپذیر) را از هم تشخیص بدهی، و بگویی چرا `for x in v` خودِ `v` را منتقل می‌کند — همان معناشناسیِ انتقال که [۱.۲.۲](../../../phase1-fundamentals/02-ownership-and-memory/02-move-semantics/README.fa.md) یادت داد.

**زمان:** حدود ۷۵ دقیقه · **پیش‌نیاز:** [۲.۲.۳ — مصرف و جمع‌آوری](../03-consuming-and-collecting/README.fa.md)، و به‌طورِ خاص [۲.۲.۲ — آداپتورهای Iterator](../02-iterator-adapters/README.fa.md) و [۱.۲.۲ — معناشناسیِ حرکت](../../../phase1-fundamentals/02-ownership-and-memory/02-move-semantics/README.fa.md)

---

## چرا اهمیت دارد

تا اینجا، تو همیشه **مصرف‌کننده‌ی** `Iterator` بوده‌ای، نه سازنده‌اش. [۲.۲.۲](../02-iterator-adapters/README.fa.md) یادت داد `.map()`، `.filter()`، `.take()` و بقیه‌ی آداپتورها را زنجیر کنی. [۲.۲.۳](../03-consuming-and-collecting/README.fa.md) یادت داد آن زنجیره را با `.collect()`، `.fold()`، `.sum()` به یک نتیجه برسانی. هر دوتایش روی چیزهایی بود که کتابخانه‌ی استاندارد از قبل برایت `Iterator`شان کرده بود — `Vec`، `HashMap`، برش‌ها.

اما کدِ واقعی مدام به چیزی نیاز دارد که از قبل در قالبِ `Vec` وجود ندارد: یک تولیدکننده‌ی دنباله (یک ژنراتورِ فیبوناچی، یک شمارنده)؛ لایه‌ای رویِ نتایجِ یک API که صفحه‌به‌صفحه می‌آیند، نه همه‌شان یک‌جا در حافظه؛ یک نوعِ سفارشیِ خودت — یک صفِ تماشا، یک لاگِ رخداد — که دلت می‌خواهد دقیقاً همان رفتاری را داشته باشد که `Vec` دارد: بشود رویش `.filter()` زد، بشود با `for` پیمایشش کرد.

خبرِ خوب این است: این قدرت رایگان نیست، ولی یک‌بار به‌دستش می‌آوری و برایِ همیشه مالِ توست. `Iterator` فقط **یک** متد از تو می‌خواهد. همان یکی. هر آداپتور و هر مصرف‌کننده‌ای که [۲.۲.۲](../02-iterator-adapters/README.fa.md) و [۲.۲.۳](../03-consuming-and-collecting/README.fa.md) یادت دادند — ده‌ها متد — از قبل، روی همان یک متد، نوشته شده‌اند. بنویسش، و همه‌شان، بدونِ استثنا، روی نوعِ خودت هم کار می‌کنند.

---

## مفهوم

### صفتِ `Iterator`: یک متد، `next`

یک ژنراتورِ فیبوناچی بساز — نوعی که هر بار صداکردنش، عددِ بعدیِ دنباله را می‌دهد. حالتی که برایِ این کار لازم است — عددِ فعلی، و عددِ بعدی — همین‌جا، توی خودِ ساختار زندگی می‌کند:

```rust
struct Fibonacci {
    current: u64,
    next: u64,
}
impl Iterator for Fibonacci {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        let value = self.current;
        let new_next = self.current + self.next;
        self.current = self.next;
        self.next = new_next;
        Some(value)
    }
}
```

حالا صدایش بزن، دستی، هشت بار:

```rust
let mut fib = Fibonacci { current: 0, next: 1 };
for _ in 0..8 {
    println!("{:?}", fib.next());
}
```

```text
Some(0)
Some(1)
Some(1)
Some(2)
Some(3)
Some(5)
Some(8)
Some(13)
```

همین کافی بود. `Iterator` دقیقاً دو چیز از پیاده‌سازش می‌خواهد: یک **نوعِ وابسته (associated type)** به اسمِ `Item` — می‌گوید هر صدا زدنِ `next` چه نوعی تولید می‌کند — و یک متدِ اجباری، `next(&mut self) -> Option<Self::Item>`. نوعِ وابسته باعث نمی‌شود خودِ `Iterator` جنریک شود؛ هر پیاده‌ساز فقط می‌گوید «مالِ من `Item`ش این است»، و همین کافی است. [۲.۳.۵](../../03-traits-and-generics/05-associated-types/README.fa.md) این مکانیزم را کاملاً باز می‌کند؛ همین یک پاراگراف امروز کافی است.

`&mut self` را جدی بگیر — `next` باید حالت را *تغییر* بدهد تا صدایِ بعدی چیزِ دیگری برگرداند؛ با `&self` اصلاً کامپایل نمی‌شود (خطایش در «خطاهایی که خواهی دید» است). و یک نکته‌ی کوچک: این‌جا ساختار یک فیلد به اسمِ `next` دارد *و* متدِ `next()` هم دارد. Rust این دو را کاملاً از هم جدا می‌بیند — `self.next` یعنی فیلد، `self.next()` یعنی صدا زدنِ متد — ولی اگر گیج‌کننده به نظر می‌رسد، اسمِ فیلد را در کدِ خودت هرچیزِ دیگری بگذار؛ صفت به اسمِ فیلدهایت کاری ندارد.

> **پلِ پایتون:** اگر از پایتون آمده‌ای، این را قبلاً یک‌بار دیگر، با اسمِ دیگری، نوشته‌ای: پروتکلِ `__iter__`/`__next__`. `__next__` مقدارِ بعدی را برمی‌گرداند یا `StopIteration` را *پرتاب* می‌کند تا بگوید تمام شد. Rust همان ایده را دارد، با یک تفاوتِ واقعی: پایان یک استثنا نیست، یک مقدارِ معمولی است — `None`، همان چیزی که سیستمِ نوع از اول مجبورت می‌کند مدیریتش کنی. هیچ چیزِ استثناییِ در جریان نیست، فقط یک `Option` که تهی شده.

### حلقه‌ی `for` چیزی جز `loop` و `match` رویِ `next` نیست

از آن‌جا که `Fibonacci` هیچ‌وقت `None` برنمی‌گرداند (دنباله‌اش بی‌پایان است)، اول با `.take(6)` محدودش کن. حالا همان شش مقدار را دستی، با یک `loop`، بگیر:

```rust
let mut manual = Fibonacci { current: 0, next: 1 }.take(6);
loop {
    match manual.next() {
        Some(value) => println!("{value}"),
        None => break,
    }
}
```

و همین شش مقدار را با `for`:

```rust
for value in (Fibonacci { current: 0, next: 1 }).take(6) {
    println!("{value}");
}
```

```text
0
1
1
2
3
5
```

هر دو، دقیقاً همین خروجی را می‌دهند — چون هر دو، دقیقاً همین کار را می‌کنند. `for value in iter { BODY }` چیزی نیست جز خلاصه‌نویسیِ همان `loop`/`match` بالا: کامپایلر خودش این را برایت می‌نویسد، هر بار که `for` تایپ می‌کنی. هیچ جادویی نیست؛ فقط یک صدا زدنِ `.next()` است که خودش را تکرار می‌کند تا `None` بگیرد.

```senpai-visual
{"kind":"concept","labels":["حالت: current, next","next() فراخوانی می‌شود","مقدار خوانده می‌شود","حالتِ تازه ذخیره می‌شود","Some(مقدار) برمی‌گردد"]}
```

### همه‌چیز رایگان: آداپتورها و مصرف‌کننده‌ها

`Fibonacci` تا این‌جا فقط `next()` را نوشته. حالا رویش هر چیزی را که [۲.۲.۲](../02-iterator-adapters/README.fa.md) و [۲.۲.۳](../03-consuming-and-collecting/README.fa.md) یادت دادند امتحان کن — `.take()`، `.filter()`، `.map()`، `.collect()`، `.sum()`، `.enumerate()`:

```rust
let first_ten: Vec<u64> = (Fibonacci { current: 0, next: 1 }).take(10).collect();
let evens: Vec<u64> = (Fibonacci { current: 0, next: 1 })
    .take(10)
    .filter(|n| n % 2 == 0)
    .collect();
let sum: u64 = (Fibonacci { current: 0, next: 1 }).take(10).sum();
```

```text
first 10:      [0, 1, 1, 2, 3, 5, 8, 13, 21, 34]
even, of those: [0, 2, 8, 34]
sum of first 10: 88
```

هیچ‌کدامِ این‌ها را برایِ `Fibonacci` ننوشتی. `.take()`، `.filter()`، `.collect()`، `.sum()` — همه‌شان متدهایی‌اند با پیاده‌سازیِ پیش‌فرض، رویِ خودِ صفتِ `Iterator`، ساخته‌شده روی همان یک متدی که تو نوشتی. کتابخانه‌ی استاندارد آن‌ها را یک‌بار، برایِ *هر* نوعی که `next()` دارد، نوشته — این دقیقاً همان قولی است که بخشِ قبل داد.

### صفتِ `IntoIterator`: چیزی که `for` را قانونی می‌کند

`Iterator` می‌گوید یک نوع چطور مقدار تولید کند. `IntoIterator` چیزِ دیگری‌ست: صفتی که `for x in value` واقعاً صدا می‌زند. بدونِ آن، آن خط اصلاً کامپایل نمی‌شود — مهم نیست `value` خودش چقدر شبیهِ یک مجموعه به‌نظر برسد.

یک نوعِ سفارشی بساز — یک صفِ تماشا، پیچیده‌شده دورِ یک `Vec<String>` — و `IntoIterator` را برایش پیاده کن:

```rust
struct WatchList(Vec<String>);

impl IntoIterator for WatchList {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
```

دو نوعِ وابسته لازم بود: `Item` (چه چیزی تولید می‌شود — این‌جا `String`، نه `&String`، چون `self` بدونِ `&` گرفته شده) و `IntoIter` (خودِ نوعِ پیمایشگر — این‌جا همان چیزی که `Vec<String>::into_iter()` از قبل برمی‌گرداند؛ کارِ اضافه‌ای لازم نبود، فقط تحویلش دادیم). حالا:

```rust
let queue = WatchList(vec!["Frieren".to_string(), "Bocchi the Rock!".to_string()]);
for title in queue {
    println!("now watching: {title}");
}
```

```text
now watching: Frieren
now watching: Bocchi the Rock!
```

این خط قبلاً کامپایل نمی‌شد. حالا می‌شود، فقط چون `IntoIterator` را پیاده کردی.

### قراردادِ سه‌شکلی: با مقدار، با ارجاعِ اشتراکی، با ارجاعِ تغییرپذیر

`WatchList` بالا فقط **یک** شکل از `IntoIterator` را پیاده کرد: با مقدار. ولی یک نوعِ خوش‌رفتار — دقیقاً مثلِ `Vec` — این صفت را **سه بار** پیاده می‌کند، هرکدام برایِ یک جورِ متفاوتِ دسترسی. `Vec` را از [۱.۱.۶](../../../phase1-fundamentals/01-foundations/06-vec-and-string-basics/README.fa.md) به بعد همین‌طور استفاده کرده‌ای، بدونِ اینکه اسمِ رسمیِ این سه‌تا را بدانی:

```rust
let titles = vec!["Frieren".to_string(), "Bocchi the Rock!".to_string()];

for t in &titles {
    println!("{t}");
}
println!("titles still usable, {} entries", titles.len());
```

```text
Frieren
Bocchi the Rock!
titles still usable, 2 entries
```

| نوشته می‌شود | صدا می‌زند | `Item` چیست | `titles` بعدش |
|---|---|---|---|
| `for t in &titles` | `IntoIterator for &Vec<T>` | `&T` | همچنان قابلِ‌استفاده |
| `for t in &mut titles` | `IntoIterator for &mut Vec<T>` | `&mut T` | همچنان قابلِ‌استفاده |
| `for t in titles` | `IntoIterator for Vec<T>` | `T` | **رفته** — منتقل شد |

سه‌تا `impl` جدا، رویِ سه‌تا نوعِ جدا (`Vec<T>` خودش، `&Vec<T>`، `&mut Vec<T>`) — نه یک `impl` که هر سه حالت را با هم پوشش بدهد. `WatchList` بالا فقط ردیفِ سوم را دارد؛ ردیفِ اول و دوم را عمداً ننوشتیم — علتش را بخشِ بعد می‌گوید، و «خطاهایی که خواهی دید» نشانت می‌دهد این نبودن دقیقاً چه شکلی است.

```senpai-visual
{"kind":"ownership","labels":["`for x in v` نوشته می‌شود","IntoIterator::into_iter(v) صدا می‌شود","self بدونِ & — یعنی مالکیت","v منتقل می‌شود","v دیگر قابلِ‌استفاده نیست"]}
```

### چرا `for x in v` خودِ `v` را می‌بَرَد

جدولِ بالا را دوباره نگاه کن: ردیفِ سوم، `fn into_iter(self) -> ...` را با `self` می‌گیرد — نه `&self`، نه `&mut self`. همان امضایی که خودِ `WatchList` بالا نوشت. `self` بدونِ `&` یعنی همان چیزی که [۱.۲.۲](../../../phase1-fundamentals/02-ownership-and-memory/02-move-semantics/README.fa.md) از اول یادت داد: مالکیت منتقل می‌شود، نه قرض گرفته. `for x in v` هیچ نحوِ خاصی ندارد — فقط یک صدا زدنِ معمولیِ `IntoIterator::into_iter(v)` است، و آن تابع `v` را با مقدار می‌گیرد. بعدِ آن خط، `v` همان‌قدر رفته است که بعدِ هر تابعِ دیگری که آرگومانش را با مقدار می‌گرفت.

این دقیقاً چرایِ اینکه `for t in &titles` یک چیزِ دیگر است: آن‌جا آرگومان `&titles` است، نه `titles` — یک ارجاع صدا زده می‌شود، `into_iter(self: &Vec<T>)` (ردیفِ اول جدول)، و آن `self` فقط قرض می‌گیرد. سه‌شکلی‌ای که بخشِ قبل نشان داد، دقیقاً همین‌جا اثرش را می‌گذارد: کدام `impl` صدا زده می‌شود، همان چیزی است که تعیین می‌کند `v` بعدش هنوز مالِ توست یا نه.

---

## دست‌به‌کد

```sh
cargo run -p p2-02-04-implementing-iterator --example 01-fibonacci-manual-next
cargo run -p p2-02-04-implementing-iterator --example 02-for-loop-is-next-in-a-loop
cargo run -p p2-02-04-implementing-iterator --example 03-fibonacci-adapters-and-consumers-free
cargo run -p p2-02-04-implementing-iterator --example 04-watchlist-into-iterator-by-value
cargo run -p p2-02-04-implementing-iterator --example 05-vec-three-forms-recap
```

بعد چهارتای خراب:

```sh
cargo run -p p2-02-04-implementing-iterator --example 06-missing-next-method --features broken
cargo run -p p2-02-04-implementing-iterator --example 07-next-wrong-self-mutability --features broken
cargo run -p p2-02-04-implementing-iterator --example 08-use-after-move --features broken
cargo run -p p2-02-04-implementing-iterator --example 09-reference-not-into-iterator --features broken
```

بعد این‌ها را امتحان کن:

۱. در `01-fibonacci-manual-next.rs`، مقدارِ شروع را از `current: 0, next: 1` به `current: 1, next: 1` عوض کن. دنباله چطور فرق می‌کند؟ آیا هنوز فیبوناچیِ واقعی است؟
۲. در `03-fibonacci-adapters-and-consumers-free.rs`، یک خطِ جدید اضافه کن که با `.map(|n| n * n)` مربعِ پنج تای اول را بگیرد و `.collect()` کند. چیزِ تازه‌ای برایِ این کار لازم بود بنویسی؟
۳. در `05-vec-three-forms-recap.rs`، پیش از حلقه‌ی آخر (`for t in titles`)، سعی کن `titles.len()` را هم چاپ کنی. کامپایل می‌شود؟ چرا همین سؤال، برایِ حلقه‌ی *اول* (`for t in &titles`) جوابِ دیگری دارد؟

---

## خطاهایی که خواهی دید

### `E0046` — یک آیتمِ صفت پیاده نشده: `next`

```rust
struct Fibonacci {
    current: u64,
    next: u64,
}
impl Iterator for Fibonacci {
    type Item = u64;
}
```

```text
error[E0046]: not all trait items implemented, missing: `next`
  --> phase2-intermediate\02-iterators-and-closures\04-implementing-iterator\examples\06-missing-next-method.rs:23:1
   |
23 | impl Iterator for Fibonacci {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^ missing `next` in implementation
   |
   = help: implement the missing item: `fn next(&mut self) -> Option<<Self as Iterator>::Item> { todo!() }`

For more information about this error, try `rustc --explain E0046`.
```

**کامپایلر به چه اعتراض دارد:** نوعِ وابسته‌ی `Item` را نوشتی، ولی `Iterator` یک چیزِ دیگر هم می‌خواهد — همان یک متدِ اجباری — و آن‌جا نیست. برایِ تفاوت با آداپتورهایی مثلِ `.map()` که پیاده‌سازیِ پیش‌فرض دارند، `next` هیچ پیش‌فرضی ندارد؛ اصلاً چیزی نیست که رویش پیش‌فرض بسازی، چون خودش پایه‌ی همه‌ی بقیه است.

**راه‌حل:** `next` را بنویس.

**چرا این راه‌حل است:** پیامِ کمکِ کامپایلر خودش امضایِ دقیق را می‌دهد — `fn next(&mut self) -> Option<<Self as Iterator>::Item>`. همین یک متد را اضافه کن، با بدنه‌ای که واقعاً حالت را جلو می‌برد (نه فقط `todo!()` که پیشنهاد کرده)، و خطا می‌رود.

### `E0053` — امضایِ `next` با صفت جور در نمی‌آید

```rust
impl Iterator for Fibonacci {
    type Item = u64;

    fn next(&self) -> Option<u64> {
        Some(self.current)
    }
}
```

```text
error[E0053]: method `next` has an incompatible type for trait
  --> phase2-intermediate\02-iterators-and-closures\04-implementing-iterator\examples\07-next-wrong-self-mutability.rs:26:13
   |
26 |     fn next(&self) -> Option<u64> {
   |             ^^^^^ types differ in mutability
   |
   = note: expected signature `fn(&mut Fibonacci) -> Option<_>`
              found signature `fn(&Fibonacci) -> Option<_>`
help: change the self-receiver type to match the trait
   |
26 |     fn next(&mut self) -> Option<u64> {
   |              +++

For more information about this error, try `rustc --explain E0053`.
```

**کامپایلر به چه اعتراض دارد:** `Iterator` امضایی دقیق برایِ `next` تعریف کرده — `&mut self`، نه `&self` — و پیاده‌سازیِ تو با آن جور نیست. این یک تایپوی خیلی معمولی است: `&mut` را فراموش کردن، دقیقاً همان‌جایی که حالت باید تغییر کند.

**راه‌حل:** `&mut self` بنویس.

**چرا این راه‌حل است:** `next` بدونِ توانِ تغییرِ حالت، اصلاً نمی‌تواند دنباله را جلو ببرد — دفعه‌ی دوم دقیقاً همان چیزی را برمی‌گرداند که دفعه‌ی اول برگرداند. `&mut self` دقیقاً همان اجازه‌ای است که برایِ نوشتنِ `self.current = ...` لازم داری؛ همان چیزی که در همین درس، بخشِ اولِ «مفهوم» از اول رویش تأکید کرد.

### `E0382` — قرض از یک مقدارِ منتقل‌شده: `queue`

```rust
let queue = WatchList(vec!["Frieren".to_string(), "Bocchi the Rock!".to_string()]);

for title in queue {
    println!("now watching: {title}");
}

println!("queue again: {queue:?}");
```

```text
error[E0382]: borrow of moved value: `queue`
   --> phase2-intermediate\02-iterators-and-closures\04-implementing-iterator\examples\08-use-after-move.rs:28:29
    |
 22 |     let queue = WatchList(vec!["Frieren".to_string(), "Bocchi the Rock!".to_string()]);
    |         ----- move occurs because `queue` has type `WatchList`, which does not implement the `Copy` trait
 23 |
 24 |     for title in queue {
    |                  ----- `queue` moved due to this implicit call to `.into_iter()`
...
 28 |     println!("queue again: {queue:?}");
    |                             ^^^^^ value borrowed here after move
    |
note: `into_iter` takes ownership of the receiver `self`, which moves `queue`
   --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\iter\traits\collect.rs:312:18
    |
312 |     fn into_iter(self) -> Self::IntoIter;
    |                  ^^^^

For more information about this error, try `rustc --explain E0382`.
```

**کامپایلر به چه اعتراض دارد:** خودِ پیام، اسمِ دقیقِ مکانیزم را می‌گوید — «`queue` moved due to this implicit call to `.into_iter()`». این همان چیزی است که بخشِ «چرا `for x in v` خودِ `v` را می‌بَرَد» توضیح داد، این‌بار روی نوعِ خودت، نه روی `Vec`. `for title in queue` صدا زد `WatchList::into_iter(queue)`، آن متد `self` را با مقدار گرفت، و `queue` منتقل شد. خطِ آخر می‌خواهد دوباره از رویِ `queue` بخواند — ولی چیزی برایِ خواندن نمانده.

**راه‌حل:** به چیزی که از `queue` لازم داری، پیش از حلقه یا در حینِ حلقه برس — نه بعدش:

```rust
println!("queue before: {queue:?}");
for title in queue {
    println!("now watching: {title}");
}
```

**چرا این راه‌حل است:** `queue` تا پیش از حلقه هنوز کاملاً مالِ توست؛ فقط بعدِ آن رفته. جابه‌جا کردنِ ترتیب — یا جمع‌کردنِ هرچه لازم داری توی یک متغیرِ تازه، در حینِ حلقه — دقیقاً همان قاعده‌ای است که [۱.۲.۲](../../../phase1-fundamentals/02-ownership-and-memory/02-move-semantics/README.fa.md) از اول یادت داد: بعدِ یک انتقال، فقط مقصدِ جدید معتبر است.

### `E0277` — `&WatchList` یک پیمایشگر نیست

```rust
let queue = WatchList(vec!["Frieren".to_string()]);

for title in &queue {
    println!("now watching: {title}");
}
```

```text
error[E0277]: `&WatchList` is not an iterator
  --> phase2-intermediate\02-iterators-and-closures\04-implementing-iterator\examples\09-reference-not-into-iterator.rs:24:18
   |
24 |     for title in &queue {
   |                  ^^^^^^ `&WatchList` is not an iterator
   |
   = help: the trait `Iterator` is not implemented for `&WatchList`
   = note: required for `&WatchList` to implement `IntoIterator`
help: consider removing the leading `&`-reference
   |
24 -     for title in &queue {
24 +     for title in queue {
   |

For more information about this error, try `rustc --explain E0277`.
```

**کامپایلر به چه اعتراض دارد:** این‌جا دقیقاً همان درسِ «قراردادِ سه‌شکلی» به واقعیت برخورد می‌کند. `impl IntoIterator for WatchList` را نوشتیم — ردیفِ سوم آن جدول — ولی `impl IntoIterator for &WatchList` (ردیفِ اول) را هرگز ننوشتیم. این دو `impl` کاملاً جدا از همند؛ پیاده‌سازیِ یکی، دیگری را با خودش نمی‌آورد. `&WatchList` یک نوعِ متفاوت است، و صفتش را کسی برایش ننوشته.

**راه‌حل:** با همان چیزی که واقعاً پیاده کردیم کار کن — با مقدار، نه با ارجاع:

```rust
for title in queue {
    println!("now watching: {title}");
}
```

**چرا این راه‌حل است:** خودِ کامپایلر همین راه‌حل را پیشنهاد داد — «consider removing the leading `&`-reference». نوشتنِ `impl IntoIterator for &WatchList` هم راهِ درستی‌ست، ولی به یک ابزار نیاز دارد که این درس هنوز به تو نداده: یک طول‌عمرِ اسم‌دار روی خودِ `impl`، چون آن‌جا باید بگویی مقدارهایِ قرض‌گرفته‌شده تا کِی معتبرند. [۲.۴.۱](../../04-lifetimes-and-conversion/01-lifetime-basics-and-elision/README.fa.md) دقیقاً همان ابزار را می‌دهد.

---

## تمرین

### گرم‌کردن

<details>
<summary>یک ساختارِ <code>Doubler { value: u32 }</code> با این پیاده‌سازی تصور کن — <code>fn next(&mut self) -> Option&lt;u32&gt; { self.value *= 2; Some(self.value) }</code>. اگر <code>Doubler { value: 3 }</code> بسازی و سه بار <code>.next()</code> را صدا بزنی، چه برمی‌گردد؟</summary>

فکرت را قبل از خواندنِ پاسخ بنویس.

</details>

<details>
<summary>پاسخ</summary>

```text
Some(6)
Some(12)
Some(24)
```

هر صدا، مقدار را دوبرابر می‌کند و همان را برمی‌گرداند — و چون هیچ‌جا `None` تولید نمی‌شود، این پیمایشگر هم، درست مثلِ `Fibonacci`، بی‌پایان است.

</details>

<details>
<summary>همان <code>Doubler</code>، ولی این‌بار <code>fn next(&mut self) -> u32 { self.value *= 2; self.value }</code> — یعنی بدونِ <code>Option</code>. کامپایل می‌شود؟</summary>

فکرت را قبل از خواندنِ پاسخ بنویس.

</details>

<details>
<summary>پاسخ</summary>

نه — `E0053`، همان خانواده‌ی خطایی که در «خطاهایی که خواهی دید» دیدی، این‌بار روی نوعِ برگشتی نه روی `self`. `Iterator::next` باید دقیقاً `Option<Self::Item>` برگرداند؛ برگرداندنِ خودِ مقدار، بدونِ `Option`، امضا را جور نمی‌کند.

</details>

<details>
<summary><code>let v = vec![1, 2, 3]; for x in v {} println!("{v:?}");</code> کامپایل می‌شود؟</summary>

فکرت را قبل از خواندنِ پاسخ بنویس.

</details>

<details>
<summary>پاسخ</summary>

نه — `E0382`. `for x in v` صدا می‌زند `IntoIterator::into_iter(v)` که `self` را با مقدار می‌گیرد؛ `v` منتقل می‌شود و خطِ `println!` بعدش دیگر چیزی برایِ خواندن ندارد.

</details>

<details>
<summary>همان کد، ولی <code>for x in &v {}</code> به‌جایِ <code>for x in v {}</code>. حالا چطور؟</summary>

فکرت را قبل از خواندنِ پاسخ بنویس.

</details>

<details>
<summary>پاسخ</summary>

بله، کامپایل می‌شود. `&v` صدا می‌زند `IntoIterator for &Vec<T>` که `self` را فقط قرض می‌گیرد؛ `v` بعدِ حلقه هنوز کاملاً مالِ توست.

</details>

<details>
<summary>درست یا غلط: برایِ اینکه <code>.filter()</code> روی نوعِ خودت کار کند، باید خودت <code>.filter()</code> را برایش بنویسی.</summary>

فکرت را قبل از خواندنِ پاسخ بنویس.

</details>

<details>
<summary>پاسخ</summary>

غلط. تنها چیزی که باید بنویسی `next()` است. `.filter()` — مثلِ `.map()`، `.take()`، `.collect()` و ده‌ها متدِ دیگر — پیاده‌سازیِ پیش‌فرضی دارد که کتابخانه‌ی استاندارد، یک‌بار، رویِ خودِ صفتِ `Iterator` نوشته؛ هر نوعی که `next()` داشته باشد، همه‌شان را مفت می‌گیرد.

</details>

### تعمیر

هر چهار مثالِ خراب را درست کن:

۱. `examples/06-missing-next-method.rs` — متدِ کم را اضافه کن؛ باید همان دنباله‌ی فیبوناچیِ مثال‌هایِ ۰۱ و ۰۲ را تولید کند.
۲. `examples/07-next-wrong-self-mutability.rs` — امضایِ `next` را با صفت جور کن، و بدنه‌اش را طوری کامل کن که واقعاً حالت را جلو ببرد، نه فقط `current` را برگرداند.
۳. `examples/08-use-after-move.rs` — بدونِ حذف‌کردنِ چیزی که `println!` آخر نشان می‌دهد، کد را طوری بازچین که کامپایل شود. (راهنمایی: چیزی که لازم داری را پیش از حلقه چاپ کن، یا در حینِ حلقه جمعش کن.)
۴. `examples/09-reference-not-into-iterator.rs` — خطِ فراخوان را طوری عوض کن که با همان چیزی که `WatchList` واقعاً پیاده کرده کار کند.

### پیاده‌سازی

دو نوع در `src/lib.rs`، هرکدام یکی از دو صفتِ همین درس:

```sh
cargo test -p p2-02-04-implementing-iterator
```

`Collatz` صفتِ `Iterator` را می‌خواهد (فقط `next`؛ ساختار و `new` از قبل نوشته شده‌اند)، و `EpisodeLog` صفتِ `IntoIterator` را (فقط `into_iter`). کامنتِ مستنداتِ هرکدام دقیقاً می‌گوید چه رفتاری لازم است؛ چیزی را حدس نزن.

### بساز

یک پیمایشگرِ سفارشیِ تازه، مالِ خودت، به `src/lib.rs` اضافه کن — هر چیزی که دوست داری (توانی از ۲، شمارنده‌ای که هربار یک قدم می‌پرد، هرچیزِ دیگر)، با این دو شرط: (۱) واقعاً یک ماشینِ حالت باشد — مقدارِ بعدی از رویِ چیزی که در خودِ ساختار ذخیره شده حساب شود، نه از هیچ؛ (۲) در کامنتِ مستنداتش، در یکی‌دو جمله، بگویی دنباله‌اش چیست و چرا حالت باید بینِ صداها بماند. بعد یک `#[test]` بنویس که رویِ همین نوع، دستِ‌کم یک آداپتور و یک مصرف‌کننده صدا بزند که خودت ننوشتی — تا با چشمِ خودت ببینی رایگان کار می‌کنند.

### چالش (اختیاری)

`IntoIterator for &EpisodeLog` را پیاده کن تا `for title in &log` هم کامپایل شود، بدونِ اینکه `log` را ببرد. یک تفاوتِ واقعی با هرچیزی که تا این‌جا نوشتی: امضایش باید چیزی شبیهِ `impl<'a> IntoIterator for &'a EpisodeLog` باشد — یک طول‌عمرِ اسم‌دار روی خودِ `impl`. این ابزار رسماً درسِ [۲.۴.۱](../../04-lifetimes-and-conversion/01-lifetime-basics-and-elision/README.fa.md) است؛ اگر همین حالا امتحانش کنی، پیام‌هایِ کامپایلر — قدم‌به‌قدم — تقریباً خودشان راه را نشانت می‌دهند. اگر ترجیح می‌دهی صبر کنی، آن درس همین ابزار را کامل و درست یادت می‌دهد و می‌توانی همین‌جا برگردی.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| `Iterator` | صفتی با یک متدِ اجباری، `next` | ساختنِ نوعِ خودت که `map`/`filter`/`collect` رویش کار کند |
| `next(&mut self) -> Option<Self::Item>` | تنها متدی که باید بنویسی | هر پیمایشگرِ سفارشی |
| نوعِ وابسته (`type Item`) | نوعی که پیاده‌سازِ صفت تعیین می‌کند | گفتنِ اینکه `next` چه چیزی تولید می‌کند |
| `IntoIterator` | صفتی که `for x in value` را قانونی می‌کند | نوشتنِ نوعی که در `for` قابلِ‌استفاده باشد |
| قراردادِ سه‌شکلی | با مقدار / با ارجاعِ اشتراکی / با ارجاعِ تغییرپذیر | تصمیم اینکه `for` روی نوعِ تو چطور رفتار کند |

### الان می‌دانی

- `Iterator` فقط یک متدِ اجباری دارد، `next(&mut self) -> Option<Self::Item>`، به‌علاوه‌ی یک نوعِ وابسته، `Item`؛ همین دو چیز، هر آداپتور و مصرف‌کننده‌ای که [۲.۲.۲](../02-iterator-adapters/README.fa.md) و [۲.۲.۳](../03-consuming-and-collecting/README.fa.md) یادت دادند را رایگان می‌آورد.
- `for value in iter { BODY }` چیزی نیست جز یک `loop` که `.next()` را صدا می‌زند و رویِ اولین `None` می‌شکند — کامپایلر خودش این را می‌نویسد.
- `IntoIterator` صفتِ دیگری‌ست: چیزی که `for x in value` واقعاً صدا می‌زند. `Iterator` داشتن، خودکار `IntoIterator` نمی‌سازد؛ باید جدا پیاده شود.
- یک نوعِ خوش‌رفتار `IntoIterator` را سه‌بار پیاده می‌کند — با مقدار، با ارجاعِ اشتراکی، با ارجاعِ تغییرپذیر — رویِ سه نوعِ جدا (`T`، `&T`، `&mut T`)؛ پیاده‌سازیِ یکی، بقیه را با خودش نمی‌آورد.
- `for x in v` خودِ `v` را می‌بَرَد چون آن حلقه صدا می‌زند `IntoIterator::into_iter(v)`، و آن متد `self` را با مقدار می‌گیرد — همان معناشناسیِ انتقالی که [۱.۲.۲](../../../phase1-fundamentals/02-ownership-and-memory/02-move-semantics/README.fa.md) یادت داده بود، حالا با اسمِ دقیقِ مکانیزمش.

### بعداً کامل‌تر می‌بینی

- **نوع‌هایِ وابسته، به‌طورِ کامل** — [۲.۳.۵ — نوع‌های وابسته در برابرِ پارامترهای جنریک](../../03-traits-and-generics/05-associated-types/README.fa.md)
- **نوشتنِ صفتِ خودت** (نه فقط پیاده‌سازیِ صفتِ آماده) — [۲.۳.۱ — تعریف و پیاده‌سازی صفت‌ها](../../03-traits-and-generics/01-defining-and-implementing-traits/README.fa.md)
- **طول‌عمرها، برایِ پیاده‌سازیِ `IntoIterator for &EpisodeLog`** — [۲.۴.۱ — مبانی طول‌عمر و elision](../../04-lifetimes-and-conversion/01-lifetime-basics-and-elision/README.fa.md)
- **پیاده‌سازیِ فراگیر (blanket impl)، به‌طورِ رسمی — همان دلیلِ اینکه هر `Iterator` خودش هم یک `IntoIterator` است** — [۲.۳.۶ — ابرصفت‌ها، پیاده‌سازیِ فراگیر، قاعده‌ی یتیم](../../03-traits-and-generics/06-supertraits-blanket-impls-orphan-rule/README.fa.md)
- **تنبلی و کاراییِ پیمایشگرها، در عمق** — [۲.۲.۵ — تنبلی و کاراییِ ایتریتورها](../05-laziness-and-performance/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا فقط با نوشتنِ `next()`، متدهایی مثلِ `.map()` و `.filter()` و `.collect()` هم روی نوعِ خودت کار می‌کنند؟ داخلِ Rust واقعاً چه اتفاقی می‌افتد؟
- `IntoIterator` دقیقاً چه چیزی را قانونی می‌کند که `Iterator` به‌تنهایی قانونی نمی‌کرد؟
- چرا `for x in v` خودِ `v` را می‌بَرَد، ولی `for x in &v` نه؟ این به کدام متد و کدام امضا برمی‌گردد؟
- چرا پیاده‌سازیِ `IntoIterator` برایِ `WatchList`، خودکار `IntoIterator` را برایِ `&WatchList` هم نمی‌سازد؟
- `type Item` چه کاری می‌کند که `Iterator` را جنریک نمی‌کند؟

---

## بیشتر

- [مستنداتِ صفتِ `Iterator`](https://doc.rust-lang.org/std/iter/trait.Iterator.html) — فهرستِ کاملِ ده‌ها متدی که همین امروز، با نوشتنِ یک `next()`، مفت گرفتی.
- [مستنداتِ صفتِ `IntoIterator`](https://doc.rust-lang.org/std/iter/trait.IntoIterator.html) — از جمله همان پیاده‌سازیِ فراگیری که هر `Iterator` را خودش هم `IntoIterator` می‌کند.
- [ماژولِ `std::iter`](https://doc.rust-lang.org/std/iter/index.html) — توضیحِ رسمیِ همان چیزی که در «حلقه‌ی `for` چیزی جز `loop` و `match` نیست» دیدی.
- [کتابِ Rust، فصلِ ۱۳.۲ — پردازشِ یک سری آیتم با پیمایشگرها](https://doc.rust-lang.org/book/ch13-02-iterators.html)

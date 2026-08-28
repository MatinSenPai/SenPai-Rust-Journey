# ۲.۳.۲ — توابع و ساختارهای جنریک، کران‌ها، `where`

## در یک نگاه

بعد از این درس می‌توانی:

- توضیح بدهی چرا `fn largest<T: PartialOrd>(list: &[T]) -> &T` بدونِ آن کرانِ `PartialOrd` اصلاً کامپایل نمی‌شود، و خودت خطایِ دقیقش را بخوانی و رفعش کنی.
- برایِ یک ساختارِ جنریک تصمیم بگیری یک کران را کجا بگذاری — رویِ کلِ بلوکِ `impl` یا رویِ فقط یک متد — و بینِ نوشتنِ درون‌خطی و `where` یکی را برایِ یک امضایِ واقعی انتخاب کنی.
- بگویی `largest::<i32>` و `largest::<&str>` بعد از کامپایل دقیقاً دو تکه‌کدِ جداگانه‌اند، و همین چرا جنریک‌ها در Rust سرِ اجرا هیچ هزینه‌ای ندارند.

**زمان:** حدود ۶۵ دقیقه · **پیش‌نیاز:** [۲.۳.۱ — تعریف و پیاده‌سازیِ صفت‌ها](../01-defining-and-implementing-traits/README.fa.md)، و به‌طورِ خاص [۲.۲.۱ — کلوژرها، `Fn`/`FnMut`/`FnOnce` و `move`](../../02-iterators-and-closures/01-closures-and-fn-traits/README.fa.md) برایِ کرانِ `Fn`

---

## چرا اهمیت دارد

از همان درسِ اولِ فازِ ۱، هر روز داشتی از جنریک‌ها استفاده می‌کردی — فقط تا امروز اسمش را نمی‌دانستی:

- [۱.۱.۶](../../../phase1-fundamentals/01-foundations/06-vec-and-string-basics/README.fa.md) — `Vec<i32>` را به کار بردی، و `Vec<T>` هیچ‌وقت برایِ هیچ نوعِ عنصرِ دیگری بازنویسی نخواست. همیشه همان یک تعریف.
- [۱.۶.۱](../../../phase1-fundamentals/06-absence-and-failure/01-option-and-null-safety/README.fa.md) — `Option<T>` را دیدی: `Option<u32>`، `Option<String>`، `Option<Box<T>>`. باز همان یک تعریف.
- [۲.۱.۲](../../01-collections/02-hashmap-in-depth/README.fa.md) — `HashMap<K, V>` دو تا از این پارامترها را همزمان گرفت.
- [۲.۲.۱](../../02-iterators-and-closures/01-closures-and-fn-traits/README.fa.md) — حتی امضایِ `fn apply_twice<F: Fn(i32) -> i32>(f: F, x: i32) -> i32` را نوشتی، و آن درس صریح گفت: «جنریک‌ها را ماژولِ ۲.۳ کامل یاد می‌دهد.»

این همان لحظه است. امروز خودِ آن `<T>` را باز می‌کنیم: چرا آنجاست، چه قولی از تو می‌خواهد، و چطور کامپایلر از رویِ همان قول، کدِ واقعی و بدونِ هیچ هزینه‌ای در زمانِ اجرا می‌سازد.

بدونِ جنریک‌ها، تابعی مثلِ «بزرگ‌ترینِ یک لیست را پیدا کن» را باید یک‌بار برایِ `u32` می‌نوشتی، یک‌بار برایِ `f64`، یک‌بار برایِ `String` — سه تابعِ تقریباً یکسان، فقط با نوع فرق‌شان. این دقیقاً همان تکرارِ کدی است که زبان‌های برنامه‌نویسی اصلاً برایِ حذف‌کردنش به وجود آمده‌اند.

---

## مفهوم

### مشکلی که جنریک‌ها حلش می‌کنند

فرض کن یک تابع لازم داری که بزرگ‌ترین عنصرِ یک برش (slice) را پیدا کند. یک نسخه برایِ `u32`، یک نسخه برایِ `f64`، یک نسخه برایِ `String` — بدنه‌ی هر سه کلمه‌به‌کلمه یکی است، فقط نوع فرق می‌کند. تو پایتون این مشکل اصلاً وجود ندارد: پایتون نوعِ آرگومان‌ها را سرِ *اجرا* چک می‌کند نه سرِ نوشتنِ کد، پس یک `def largest(items): ...` که تویِ بدنه‌اش از `>` استفاده می‌کند، رویِ هر چیزی که از `>` پشتیبانی کند کار می‌کند — بدونِ اینکه تو کاری بکنی. بهایش این است: اگر یک روز `largest` را با چیزهایی صدا بزنی که قابلِ‌مقایسه نیستند، پایتون همان لحظه‌ای که آن خط واقعاً اجرا شود می‌فهمد، نه زودتر — و آن لحظه می‌تواند وسطِ یک درخواستِ واقعیِ کاربر باشد.

Rust نوع را سرِ کامپایل چک می‌کند، پس یک امضایِ معمولی مثلِ `fn largest(list: &[u32]) -> &u32` تا ابد فقط رویِ `u32` قفل می‌ماند. جوابِ Rust جنریک (generic) است: تابع را فقط یک‌بار بنویس، به‌جایِ یک نوعِ ثابت از یک نوعِ جایگزین (placeholder) استفاده کن، و بگذار کامپایلر خودش نسخه‌های واقعی را برایت بسازد:

```rust
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}
```

```rust
let episode_counts = [12, 24, 13, 64, 51];
println!("most episodes: {}", largest(&episode_counts));

let titles = ["Frieren", "Bocchi the Rock!", "Made in Abyss"];
println!("alphabetically last: {}", largest(&titles));
```

```text
most episodes: 64
alphabetically last: Made in Abyss
```

همان یک تعریف، دو نوعِ کاملاً متفاوت — یکی برایِ عددها، یکی برایِ رشته‌ها. `<T: PartialOrd>` را «برایِ یک نوعِ نامعلوم به اسمِ `T`» بخوان: خودِ حرفِ `T` فقط یک نوعِ جایگزین (placeholder) است — قراردادِ Rust یک حرفِ بزرگِ تکی، به‌جایِ «هر نوعِ ملموسی که فراخواننده به‌کار می‌برد».

### چرا کران اجباری است

حالا کرانِ `: PartialOrd` را از جلویِ `T` در همان امضا پاک کن و دوباره کامپایل کن. رویِ همان خطِ `if item > largest` گیر می‌کنی: کامپایلر می‌گوید عملگرِ `>` را نمی‌تواند رویِ نوعِ `T` به‌کار ببرد. خطایِ کاملش، با کدِ دقیقش (`E0369`)، در «خطاهایی که خواهی دید» است — همین الان می‌توانی خودت تویِ `examples/04-missing-bound.rs` تولیدش کنی.

نکته این‌جاست: کامپایلر رویِ خودِ `T` هیچ فرضی نمی‌گذارد — نه اینکه *بعضی* فرض‌ها را بگذارد و بعضی را نه. برایِ یک `T`ِ کاملاً نامعلوم، فقط همان کاری را می‌توانی بکنی که *هر* نوعی می‌تواند: بگیریش، پسش بدهی، تویِ یک متغیر بگذاریش. مقایسه‌کردن، چاپ‌کردن، کلون‌کردن — هیچ‌کدام رایگان نیست، مگر خودت با یک کران بهش قول بدهی. `T: PartialOrd` دقیقاً همین قول است: «هر `T`ای که این تابع باهاش صدا زده شود، مقایسه‌هایِ ترتیبی را پشتیبانی می‌کند.» با همین یک جمله، هم داخلِ بدنه‌ی تابع اجازه‌ی استفاده از `>` را می‌گیری، هم — دقیقاً همان‌قدر مهم — سرِ محلِ فراخوانی، اگر کسی `largest` را با نوعی صدا بزند که این قول را نمی‌دهد، کامپایلر همان‌جا جلویش را می‌گیرد. باگ از «چیزی که یک کاربر ممکن است سرِ اجرا باعثش بشود» تبدیل می‌شود به «چیزی که همین حالا رویِ دستگاهِ خودت جلویِ `cargo build` را می‌گیرد.»

به این محدودیت — `T: PartialOrd` — کران (trait bound) می‌گویند. از این به بعد همین کلمه را به‌کار می‌بریم.

### ساختارهای جنریک

ساختارها هم می‌توانند یک پارامترِ نوع بگیرند — دقیقاً همان `<T>` که رویِ تابع دیدی. یک `Shelf<T>` بساز — یک «قفسه» که هر چیزی از یک نوع رویش می‌گذاری:

```rust
struct Shelf<T> {
    items: Vec<T>,
}

impl<T> Shelf<T> {
    fn new() -> Self {
        Shelf { items: Vec::new() }
    }
    fn add(&mut self, item: T) {
        self.items.push(item);
    }
    fn len(&self) -> usize {
        self.items.len()
    }
}
```

```rust
let mut ratings: Shelf<u32> = Shelf::new();
ratings.add(7);
ratings.add(9);
ratings.add(6);
println!("ratings on shelf: {}", ratings.len());
```

```text
ratings on shelf: 3
```

`impl<T> Shelf<T>` هیچ کرانی ندارد — این سه متد رویِ *هر* `T`ای کار می‌کنند، حتی نوعی که هیچ صفتی پیاده نکرده باشد. `Shelf<u32>` و بعداً `Shelf<String>`، هر دو استفاده‌ی کاملاً معتبری از همین یک تعریف‌اند.

### یک متد با کرانِ خودش، در برابرِ کرانِ کلِ بلوکِ `impl`

حالا یک قابلیتِ تازه اضافه کن: پیدا کردنِ بزرگ‌ترین موردِ رویِ قفسه. این به یک قولِ اضافه نیاز دارد — که بشود دو تا `T` را با `>` مقایسه کرد. به‌جایِ اینکه این قول را رویِ کلِ بلوکِ `impl` بگذاریم، رویِ خودِ متد می‌گذاریمش، با یک `where` بعد از امضا (اینجا برایِ فشرده‌ماندنِ متن جدا نشانش می‌دهیم؛ تویِ `examples/02-shelf.rs` همه‌ی متدها تویِ یک `impl<T> Shelf<T>` هستند):

```rust
impl<T> Shelf<T> {
    fn highest(&self) -> Option<&T>
    where
        T: PartialOrd,
    {
        let mut best = self.items.first()?;
        for item in &self.items {
            if item > best {
                best = item;
            }
        }
        Some(best)
    }
}
```

```rust
println!("highest rating: {:?}", ratings.highest());
```

```text
highest rating: Some(9)
```

راهِ دیگری هم بود: می‌شد همان کران را رویِ کلِ بلوکِ `impl` گذاشت — `impl<T: PartialOrd> Shelf<T> { ... }` — به‌جایِ فقط رویِ `highest`. با همین دو متد (`new` و `highest`) تویِ آن بلوک، دو نوشتار تقریباً یک نتیجه می‌دهند. فرق وقتی خودش را نشان می‌دهد که یک متدِ *دیگر* هم به همان بلوک اضافه کنی — متدی که اصلاً کاری به مقایسه ندارد، مثلِ همان `new`. اگر `new` هم تویِ بلوکِ `impl<T: PartialOrd> Shelf<T>` باشد، حتی برایِ ساختنِ یک `Shelf<T>`ِ خالی از نوعی که `PartialOrd` ندارد، دیگر نمی‌توانی — با اینکه `new` هیچ‌وقت چیزی را مقایسه نمی‌کند. `examples/05-impl-block-bound.rs` دقیقاً همین را می‌سازد؛ کامپایلش کن و خطایِ `E0277` را در «خطاهایی که خواهی دید» بخوان.

این است فرقِ واقعیِ کران‌رویِ‌متد در برابرِ کران‌رویِ‌بلوک: کران‌رویِ‌متد فقط همان متد را محدود می‌کند؛ کران‌رویِ‌بلوک هر چیزی که بعداً داخلِ همان بلوک بنویسی را هم محدود می‌کند — حتی اگر آن متدِ تازه هیچ ربطی به آن کران نداشته باشد.

### کران به دو نوشتار: درون‌خطی و با `where`

هرجا تا اینجا کران نوشتیم، یک چیز را دو جور نوشتیم. این هم همان `largest`، این‌بار با `where`:

```rust
fn largest<T>(list: &[T]) -> &T
where
    T: PartialOrd,
{
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}
```

```rust
println!("{}", largest(&episode_counts));
```

```text
64
```

همان جواب — چون این دقیقاً همان تابع است، فقط کرانش بعد از امضا آمده، نه داخلِ `<>`. نسخه‌ی کاملش، با هر دو نوشتار کنارِ هم و یک `println!` برایِ هرکدام، تویِ `examples/03-bounds-and-where.rs` است.

وقتی فقط یک کران رویِ یک پارامتر داری، فرقی نمی‌کند کدام نوشتار را انتخاب کنی — سلیقه است. `where` اختیاری‌بودنش را از همان لحظه‌ای از دست می‌دهد که یک متد به یک کران رویِ پارامتری نیاز پیدا کند که *از قبل*، جایی بالاتر، معرفی شده — نه همین‌جا. رویِ همان `Shelf<T>`، یک متدِ تازه بنویس: پیدا کردنِ اولین موردی که یک شرطِ دلخواه را برآورده می‌کند، و پس‌دادنِ یک کپیِ مالکِ آن:

```rust
impl<T> Shelf<T> {
    fn find_and_clone<F>(&self, matches: F) -> Option<T>
    where
        T: Clone,
        F: Fn(&T) -> bool,
    {
        self.items.iter().find(|item| matches(item)).cloned()
    }
}
```

```rust
let found = ratings.find_and_clone(|&r| r >= 8);
println!("first rating >= 8: {found:?}");
```

```text
first rating >= 8: Some(9)
```

اینجا `where` واقعاً «اختیاری» نیست. `F` تازه دارد رویِ همین متد معرفی می‌شود، پس کرانش را می‌شد درون‌خطی هم نوشت: `<F: Fn(&T) -> bool>`. ولی `T` قبلاً، بالاتر، رویِ خودِ `impl<T> Shelf<T>`، معرفی شده — اینجا فقط داری یک قولِ *اضافه* برایِ همین یک متد بهش اضافه می‌کنی (`Clone`)، و برایِ اضافه‌کردنِ کران به یک پارامترِ از قبل معرفی‌شده، `where` تنها راه است. و حتی اگر `T` هم همین‌جا تازه معرفی می‌شد، وقتی دو پارامتر (`T` و `F`) هرکدام کرانِ خودشان را دارند، امضا با نوشتارِ درون‌خطی این‌قدر شلوغ می‌شود که خواندنش زحمت دارد — همه‌چیز رویِ یک خط، قبل از اینکه اصلاً به پرانتزِ پارامترها برسی. `where` همین را رویِ چند خط پخش می‌کند، هرکدام یک کران، هرکدام جدا از بقیه قابلِ‌خواندن.

### چند کران رویِ یک پارامتر

یک پارامتر می‌تواند بیش از یک کران داشته باشد — با `+` بینشان:

```rust
use std::fmt::Display;

fn announce<T: Display + Clone>(item: T) -> (String, T) {
    let headline = format!("now airing: {item}");
    (headline, item.clone())
}
```

```rust
let (headline, kept) = announce(String::from("Frieren"));
println!("{headline}");
println!("kept a copy: {kept}");
```

```text
now airing: Frieren
kept a copy: Frieren
```

`Display` برایِ `{item}` تویِ `format!` لازم است؛ `Clone` برایِ `item.clone()`. این دو قولِ کاملاً مستقل‌اند — یکی دربارهٔ چاپ‌کردن، یکی دربارهٔ تکثیرکردن — و هر دو، هم‌زمان، رویِ همان `T`. اگر یکی‌شان را برداری، آن بخشی از بدنه که به آن قول تکیه کرده بود، دیگر کامپایل نمی‌شود: `examples/06-missing-one-of-two-bounds.rs` دقیقاً `Clone` را برمی‌دارد و `E0599` می‌گیرد — تویِ «خطاهایی که خواهی دید» می‌بینی‌اش.

### تک‌ریختی‌سازی

برگرد به همان دو صدایِ `largest` که اولِ درس دیدی — یکی رویِ `[i32]`، یکی رویِ `[&str]`. یک تعریف، دو نوعِ کاملاً متفاوت. اما این به این معنی نیست که سرِ اجرا یک نسخه‌ی «عمومیِ» `largest` وجود دارد که هر بار چک می‌کند الان با چه نوعی طرف است — کاری که پایتون واقعاً همین‌جوری انجام می‌دهد. Rust این کار را سرِ *کامپایل* انجام می‌دهد: برایِ هر نوعِ ملموسی که `largest` واقعاً باهاش صدا زده می‌شود، یک نسخه‌ی کاملاً جدا و کاملاً ملموس می‌سازد — دقیقاً انگار خودت این توابع را دستی نوشته باشی:

```rust
fn largest_i32(list: &[i32]) -> &i32 {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}
```

```rust
println!("{}", largest_i32(&episode_counts));
```

```text
64
```

همان ۶۴ که بالاتر هم دیدی — چون این واقعاً همان کاری است که کامپایلر پشتِ صحنه برایت انجام می‌دهد. یک نسخه‌ی دیگر، با `T` جایگزین‌شده با نوعِ رشته، همین‌الان هم جایی تویِ باینریِ همان برنامه نشسته. به این فرآیند — تولیدِ یک نسخه‌ی کاملاً جداگانه از تابع یا ساختارِ جنریک، به‌ازایِ هر نوعی که واقعاً استفاده شده، سرِ کامپایل — تک‌ریختی‌سازی (monomorphization) می‌گویند.

```senpai-visual
{"kind":"concept","labels":["منبعِ جنریکِ largest","نمونه‌سازی برایِ i32","نمونه‌سازی برایِ رشته","دو تابعِ جداگانه در باینری","بدونِ بررسیِ نوع سرِ اجرا"]}
```

همین است دلیلِ مکانیکیِ ادعایِ «رایگان» (zero-cost) دربارهٔ جنریک‌ها — نه یک شعارِ بازاریابی، یک واقعیتِ کامپایلی: تا وقتی برنامه شروع به اجرا کند، هیچ `T`ِ نامعلومی دیگر باقی نمانده؛ هر فراخوانی، از قبل، به یک تابعِ کاملاً ملموس گره خورده. همان بررسیِ نوعی که پایتون سرِ هر فراخوانی انجام می‌دهد، اینجا کلاً وجود ندارد — نه ارزان‌تر شده، از اساس نیست.

راهِ دیگری هم برایِ انتخابِ رفتار بر اساسِ صفت هست: به‌جایِ اینکه کامپایلر سرِ کامپایل تصمیم بگیرد کدام نسخه اجرا شود، می‌شود این تصمیم را تا سرِ اجرا عقب انداخت — با یک شیءِ صفتی (`dyn Trait`). آن یک هزینه‌ی کوچکِ سرِ اجرا دارد، در برابرِ صفرِ کاملِ جنریک‌ها؛ همین مبادله موضوعِ کاملِ [۲.۳.۷](../07-static-vs-dynamic-dispatch/README.fa.md) است.

---

## دست‌به‌کد

```sh
cargo run -p p2-03-02-generic-functions-and-structs --example 01-largest
cargo run -p p2-03-02-generic-functions-and-structs --example 02-shelf
cargo run -p p2-03-02-generic-functions-and-structs --example 03-bounds-and-where
```

بعد سه‌تای خراب:

```sh
cargo run -p p2-03-02-generic-functions-and-structs --example 04-missing-bound --features broken
cargo run -p p2-03-02-generic-functions-and-structs --example 05-impl-block-bound --features broken
cargo run -p p2-03-02-generic-functions-and-structs --example 06-missing-one-of-two-bounds --features broken
```

بعد این‌ها را امتحان کن:

۱. در `01-largest.rs`، یک آرایه از `f64` هم بساز و `largest` را رویش صدا بزن — بدونِ اینکه چیزی در تعریفِ تابع عوض کنی.
۲. در `02-shelf.rs`، پیش‌بینی‌شرطِ `find_and_clone` را به چیزی تغییر بده که هیچ ریتینگی برآوردش نمی‌کند (مثلاً `r > 100`). قبل از اجرا حدس بزن خروجی چه می‌شود، بعد چک کن.
۳. در `03-bounds-and-where.rs`، یک تابعِ سوم اضافه کن، `announce_where`، که امضایِ `announce` را با `where` به‌جایِ `Display + Clone`ِ درون‌خطی بنویسد. رویِ همان ورودی، همان جواب را می‌دهد؟

---

## خطاهایی که خواهی دید

### `E0369` — نمی‌شود دو تا `T` را با `>` مقایسه کرد

```text
error[E0369]: binary operation `>` cannot be applied to type `&T`
  --> phase2-intermediate\03-traits-and-generics\02-generic-functions-and-structs\examples\04-missing-bound.rs:12:17
   |
12 |         if item > largest {
   |            ---- ^ ------- &T
   |            |
   |            &T
   |
help: consider restricting type parameter `T` with trait `PartialOrd`
   |
 9 | fn largest<T: std::cmp::PartialOrd>(list: &[T]) -> &T {
   |             ++++++++++++++++++++++

For more information about this error, try `rustc --explain E0369`.
```

**کامپایلر به چه اعتراض دارد:** `T` تویِ این تابع کاملاً نامعلوم است — می‌تواند هر نوعی باشد. کامپایلر رویِ آن هیچ فرضی نمی‌گذارد، پس وقتی به خطِ `item > largest` می‌رسد، نمی‌داند عملگرِ `>` اصلاً برایِ `T` معنی دارد یا نه. توجه کن خودِ خطا رویِ `&T` است نه `T` — چون `item` و `largest` هر دو ارجاع‌اند (`list` از نوعِ `&[T]` است) — ولی این تفاوت به مشکلِ اصلی ربطی ندارد: مشکل نبودنِ کران است، نه ارجاع‌بودن.

**راه‌حل:** دقیقاً همان چیزی که کامپایلر خودش پیشنهاد داد — کرانِ `PartialOrd` را برگردان:

```rust
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}
```

**چرا این راه‌حل است:** `PartialOrd` دقیقاً همان صفتی است که `<`، `>`، `<=` و `>=` را تعریف می‌کند. اضافه‌کردنش به کامپایلر می‌گوید هر `T`ای که این تابع باهاش صدا زده شود، قولِ مقایسه‌شدن را می‌دهد — و آن‌وقت `item > largest` معنی پیدا می‌کند.

### `E0277` — کرانِ رویِ بلوکِ `impl` هر متدِ داخلش را هم می‌گیرد

```text
error[E0277]: can't compare `MangaVolume` with `MangaVolume`
  --> phase2-intermediate\03-traits-and-generics\02-generic-functions-and-structs\examples\05-impl-block-bound.rs:25:38
   |
25 |     let _shelf: Shelf<MangaVolume> = Shelf::new();
   |                                      ^^^^^^^^^^^^ no implementation for `MangaVolume < MangaVolume` and `MangaVolume > MangaVolume`
   |
   = help: the trait `PartialOrd` is not implemented for `MangaVolume`
note: required by a bound in `Shelf::<T>::new`
  --> phase2-intermediate\03-traits-and-generics\02-generic-functions-and-structs\examples\05-impl-block-bound.rs:14:9
   |
14 | impl<T: PartialOrd> Shelf<T> {
   |         ^^^^^^^^^^ required by this bound in `Shelf::<T>::new`
15 |     fn new() -> Self {
   |        --- required by a bound in this associated function
help: consider annotating `MangaVolume` with `#[derive(PartialOrd)]`
   |
20 + #[derive(PartialOrd)]
21 | struct MangaVolume {
   |

For more information about this error, try `rustc --explain E0277`.
```

**کامپایلر به چه اعتراض دارد:** `new` هیچ‌وقت چیزی را مقایسه نمی‌کند — فقط یک `Vec` خالی می‌سازد. ولی چون `new` تویِ همان بلوکِ `impl<T: PartialOrd> Shelf<T>` نوشته شده، کامپایلر همان قولی را از `T` می‌خواهد که یک متدِ مقایسه‌کننده لازم دارد، حتی برایِ این متدی که اصلاً به مقایسه ربطی ندارد. `MangaVolume` این قول را نداده (`PartialOrd` را پیاده نکرده)، پس حتی خودِ `Shelf::new()` هم برایش کامپایل نمی‌شود.

**راه‌حل:** کران را از کلِ بلوک بردار و فقط رویِ متدی بگذار که واقعاً بهش نیاز دارد:

```rust
impl<T> Shelf<T> {
    fn new() -> Self {
        Shelf { items: Vec::new() }
    }
}
```

**چرا این راه‌حل است:** حالا `new` رویِ هر `T`ای کار می‌کند — همان‌طور که همیشه باید می‌کرد — و فقط متدی که واقعاً مقایسه می‌کند کرانِ خودش را جداگانه می‌گیرد. کامپایلر هم موافق است: با این تغییر، همین برنامه با `MangaVolume` کامپایل می‌شود.

### `E0599` — یکی از دو کران را برداری، همان یکی گم می‌شود

```text
error[E0599]: no method named `clone` found for type parameter `T` in the current scope
  --> phase2-intermediate\03-traits-and-generics\02-generic-functions-and-structs\examples\06-missing-one-of-two-bounds.rs:13:21
   |
11 | fn announce<T: Display>(item: T) -> (String, T) {
   |             - method `clone` not found for this type parameter
12 |     let headline = format!("now airing: {item}");
13 |     (headline, item.clone())
   |                     ^^^^^ method not found in `T`
   |
   = help: items from traits can only be used if the type parameter is bounded by the trait
help: the following trait defines an item `clone`, perhaps you need to restrict type parameter `T` with it:
   |
11 | fn announce<T: Display + Clone>(item: T) -> (String, T) {
   |                        +++++++

For more information about this error, try `rustc --explain E0599`.
```

**کامپایلر به چه اعتراض دارد:** `T: Display` فقط قولِ چاپ‌شدن را می‌دهد؛ `.clone()` یک قولِ کاملاً دیگر است (`Clone`)، و کسی این یکی را نداده. کامپایلر دقیقاً می‌گوید کدام صفت این متد را تعریف می‌کند و پیشنهاد می‌دهد همان را اضافه کنی.

**راه‌حل:** کرانِ دوم را برگردان:

```rust
fn announce<T: Display + Clone>(item: T) -> (String, T) {
    let headline = format!("now airing: {item}");
    (headline, item.clone())
}
```

**چرا این راه‌حل است:** هر خط از بدنه‌ی تابع قولِ خودش را می‌خواهد — `format!` به `Display`، `.clone()` به `Clone`. دو کران رویِ یک پارامتر یعنی هر دو قول، هم‌زمان، لازم‌اند؛ برداشتنِ یکی، همان یکی را از بدنه می‌گیرد.

---

## تمرین

### گرم‌کردن

<details>
<summary>این کامپایل می‌شود؟</summary>

```rust
fn first<T>(list: &[T]) -> &T {
    &list[0]
}
```

</details>

<details>
<summary>پاسخ</summary>

بله. این تابع هیچ کاری با `T` نمی‌کند جز نگه‌داشتنش و پس‌دادنش — نه مقایسه، نه چاپ، نه کلون. برایِ همین هیچ کرانی لازم ندارد؛ `T` می‌تواند هر نوعی باشد.

</details>

<details>
<summary>این کامپایل می‌شود؟</summary>

```rust
fn largest<T>(list: &[T]) -> &T {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}
```

</details>

<details>
<summary>پاسخ</summary>

نه. `item > largest` به کرانِ `PartialOrd` نیاز دارد که اینجا نیست — همان `E0369`ای که در «خطاهایی که خواهی دید» دیدی.

</details>

<details>
<summary>آیا <code>f</code> و <code>g</code> رویِ دقیقاً یک مجموعه از نوع‌ها کار می‌کنند؟</summary>

```rust
fn f<T: Clone>(x: T) -> T {
    x.clone()
}

fn g<T>(x: T) -> T
where
    T: Clone,
{
    x.clone()
}
```

</details>

<details>
<summary>پاسخ</summary>

بله، کاملاً. `where` فقط نوشتارِ دیگری برایِ همان کران است — همان قرارداد، دو جور نوشته‌شده. هر نوعی که `f` قبول کند، `g` هم قبول می‌کند، و برعکس.

</details>

<details>
<summary>این کامپایل می‌شود، با اینکه <code>Pair&lt;T&gt;</code> هیچ کرانی ندارد؟</summary>

```rust
struct Pair<T> {
    a: T,
    b: T,
}

impl<T> Pair<T> {
    fn describe(&self) -> &str {
        "a pair"
    }
}
```

</details>

<details>
<summary>پاسخ</summary>

بله. `describe` هیچ کاری با `a` یا `b` نمی‌کند، فقط یک رشته‌ی ثابت برمی‌گرداند — برایِ همین به هیچ قولی از `T` نیاز ندارد.

</details>

### تعمیر

هر سه مثالِ خراب را درست کن:

۱. `examples/04-missing-bound.rs` را طوری درست کن که کامپایل شود — کرانی که واقعاً لازم است را برگردان.
۲. `examples/05-impl-block-bound.rs` را طوری درست کن که بشود `Shelf<MangaVolume>` ساخت — بدونِ اینکه `MangaVolume` را تغییر بدهی. (سرنخ: کران باید کجا برود؟)
۳. `examples/06-missing-one-of-two-bounds.rs` را طوری درست کن که `.clone()` دوباره کار کند.

### پیاده‌سازی

چهار امضا در `src/lib.rs`:

```sh
cargo test -p p2-03-02-generic-functions-and-structs
```

### بساز

یک `pub fn describe_all<T: Display>(items: &[T]) -> String` بنویس که خروجیِ `Display` هر عنصر را تویِ یک رشته کنارِ هم بگذارد — با فرمتی که خودت انتخاب می‌کنی و در کامنتِ مستنداتِ تابع می‌نویسی.

بعد یک نسخه‌ی دوم بنویس، `describe_matching`، که فقط عنصرهایی را نشان بدهد که یک `predicate` رویشان `true` می‌دهد — امضایِ کاملِ تابع، و کرانی که `predicate` باید داشته باشد، این‌بار انتخابِ خودت است.

### چالش (اختیاری)

**بخشِ یک.** همان `Pair<T>`ای که در «پیاده‌سازی» ساختی را با یک متدِ تازه‌ی `swap(&mut self)` گسترش بده که دو مقدارش را جابه‌جا کند — بدونِ اینکه هیچ کرانی اضافه کنی. چرا این متد اصلاً به کران نیاز ندارد؟

**بخشِ دو.** (این یکی جلوتر را نگاه می‌کند.) فرض کن می‌خواهی یک `Vec` بسازی که بتواند هم `AnimeSeries` و هم `MangaVolume` را کنارِ هم نگه دارد — به شرطِ اینکه هر دو یک متدِ مشترک مثلِ `summarize()` داشته باشند. با چیزی که امروز یاد گرفتی (یک `Vec<T>` با یک `T`ِ جنریک) این ممکن نیست: هر `Vec<T>` فقط یک `T`ِ ملموس را همزمان نگه می‌دارد، هرچقدر هم کرانش را عوض کنی. رویِ کاغذ بنویس چرا این محدودیت وجود دارد، و یک جمله حدس بزن Rust چه راهِ‌حلی برایش دارد — جوابت را در [۲.۳.۷](../07-static-vs-dynamic-dispatch/README.fa.md) بررسی کن.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| جنریک (generic) | کدی که یک‌بار نوشته و رویِ یک نوع پارامتری شده | هر تابع یا ساختاری که باید رویِ چند نوع کار کند |
| کران (trait bound) | قولی که یک پارامترِ جنریک می‌دهد: «هر نوعی که اینجا بگذاری، فلان صفت را دارد» | `<T: PartialOrd>`، `<T: Display + Clone>` |
| `where` | همان کران، نوشته‌شده بعد از امضا به‌جایِ داخلِ `<>` | وقتی چند کران رویِ چند پارامتر جمع می‌شوند، یا کران رویِ پارامترِ از‌قبل‌معرفی‌شده لازم است |
| تک‌ریختی‌سازی (monomorphization) | تولیدِ یک نسخه‌ی کاملاً جداگانه از تابع/ساختارِ جنریک، به‌ازایِ هر نوعِ واقعاً استفاده‌شده، سرِ کامپایل | دلیلِ رایگان‌بودنِ جنریک‌ها در زمانِ اجرا |

### الان می‌دانی

- امضایِ `fn largest<T: PartialOrd>(list: &[T]) -> &T` را می‌خوانی و می‌دانی چرا آن کران آنجاست.
- بدونِ کران، چرا کامپایلر `E0369` می‌دهد و خودت این خطا را رفع می‌کنی.
- یک ساختارِ جنریک با یک بلوکِ `impl` بدونِ کران می‌نویسی، و یک متد با کرانِ خودش بهش اضافه می‌کنی.
- بینِ گذاشتنِ کران رویِ کلِ بلوکِ `impl` و رویِ فقط یک متد، آگاهانه انتخاب می‌کنی — و می‌دانی بهایِ انتخابِ اول چیست.
- کران را هم درون‌خطی و هم با `where` می‌نویسی، و می‌دانی کِی دومی نه‌فقط خواناتر بلکه تنها راه است.
- چند کران را رویِ یک پارامتر با `+` می‌گذاری.
- می‌گویی تک‌ریختی‌سازی دقیقاً چه کاری، و چه وقتی، انجام می‌دهد — و چرا همین چیزی است که جنریک‌ها را در Rust رایگان می‌کند.

### بعداً کامل‌تر می‌بینی

- **صفتِ `Display`/`Debug` را برایِ نوعِ خودت بنویسی، و `PartialOrd`/`Ord`/`Hash` را برایِ نوعِ خودت درایو کنی** — [۲.۳.۴ — مشتق‌های استاندارد، دستی پیاده‌سازی‌شده](../04-standard-derives-by-hand/README.fa.md)
- **نوع‌هایِ وابسته، جایگزینِ پارامترهایِ جنریک** — [۲.۳.۵ — نوع‌های وابسته در برابرِ پارامترهای جنریک](../05-associated-types/README.fa.md)
- **`dyn Trait` و ارسالِ پویا، در برابرِ همین چیزی که امروز دیدی** — [۲.۳.۷ — ارسالِ ایستا در برابرِ پویا](../07-static-vs-dynamic-dispatch/README.fa.md)
- **طولِ عمرِ آن `&T` که `largest` برمی‌گرداند، به‌طورِ کامل** — [۲.۴.۱ — پایه‌های طولِ عمر و elision](../../04-lifetimes-and-conversion/01-lifetime-basics-and-elision/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا `fn largest<T>(list: &[T]) -> &T` بدونِ کران کامپایل نمی‌شود؟ دقیقاً کدام خط از بدنه گیر می‌کند؟
- فرقِ گذاشتنِ یک کران رویِ کلِ بلوکِ `impl` با گذاشتنش رویِ فقط یک متد چیست؟ یک مثال از خودت بزن.
- `<T: Display + Clone>` و `where T: Display + Clone` چه فرقی با هم دارند؟
- چرا `matches_count` (تویِ `src/lib.rs`) به هیچ کرانی رویِ `T` نیاز ندارد؟
- تک‌ریختی‌سازی دقیقاً کِی اتفاق می‌افتد — سرِ کامپایل یا سرِ اجرا؟ و همین چرا جنریک‌ها را «رایگان» می‌کند؟
- `dyn Trait` با جنریک چه فرقی دارد؟ (فقط یک جمله؛ جزئیاتش را ۲.۳.۷ می‌گوید.)

---

## بیشتر

- [کتابِ Rust — جنریک‌ها، صفت‌ها و طولِ عمر](https://doc.rust-lang.org/book/ch10-00-generics.html) — همین زمین، رسمی و کامل‌تر.
- [وبلاگِ رسمیِ Rust — «Abstraction without overhead: traits in Rust»](https://blog.rust-lang.org/2015/05/11/traits.html) — همان ادعایِ «رایگان»، از زبانِ خودِ تیمِ Rust.
- [`std::cmp::PartialOrd`](https://doc.rust-lang.org/std/cmp/trait.PartialOrd.html) — همان صفتی که امروز بارها کران زدیش؛ فهرستِ کاملِ چیزی که قول می‌دهد.
- [مرجعِ Rust — پارامترهایِ جنریک](https://doc.rust-lang.org/reference/items/generics.html) — نوشتارِ رسمی و دقیقِ همه‌ی چیزی که امروز دیدی.

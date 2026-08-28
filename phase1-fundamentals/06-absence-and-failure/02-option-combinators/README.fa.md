# ۱.۶.۲ — ترکیب‌گرهای `Option`

## در یک نگاه

بعد از این درس می‌توانی:

- بگویی `.map()` و `.and_then()` چه فرقی دارند، و چرا استفاده از یکی به‌جای دیگری یک `Option<Option<T>>` تودرتو می‌سازد.
- یک تکه منطق را هم به‌شکل `match` و هم به‌شکل زنجیره‌ی ترکیب‌گر بنویسی، و صادقانه بگویی کدام‌شان اینجا خواناتر است.
- با شاهدِ واقعی — نه حدس — بگویی چرا `.unwrap_or_else(|| expensive())` می‌تواند از `.unwrap_or(expensive())` ارزان‌تر باشد.
- بین حدودِ ده متدِ `Option` — `.filter()`، `.or()`، `.take()`، `.zip()` و بقیه — همانی را انتخاب کنی که دقیقاً همان کاری را می‌کند که می‌خواهی.

**زمان:** حدود ۷۰ دقیقه · **پیش‌نیاز:** [۱.۶.۱ — نوع `Option` و ایمنی null](../01-option-and-null-safety/README.fa.md)

---

## چرا اهمیت دارد

درسِ قبل به تو یک ابزار داد: `match` روی `Option`. با آن هرکاری می‌شود کرد — کامپایلر مجبورت می‌کند هر دو حالت `Some` و `None` را بپوشانی، و همین ایمنی‌اش را می‌سازد.

اما یک تجربه‌ی کوچک را امتحان کن: کدی بنویس که یک `Option<u32>` را بگیرد، اگر بود یکی به آن اضافه کند، و اگر نبود صفر برگرداند. با `match` این سه خط می‌شود — `Some(x) => x + 1` و `None => 0` و پرانتزها. حالا تصور کن پنج تا از این‌ها را پشتِ سرِ هم داری: بگیر، فیلتر کن، تبدیل کن، جایگزین بگذار. با `match` خالص، تو در تو می‌شوی.

کدِ واقعیِ Rust این کار را با زنجیره‌ای از **ترکیب‌گرها (combinators)** می‌کند — متدهایی مثلِ `.filter()`، `.map()` و `.unwrap_or()` که می‌شود پشتِ سرِ هم گذاشت: `opt.filter(...).map(...).unwrap_or(...)`. کوتاه‌تر است، و وقتی درست استفاده شود خواناتر هم هست. اما دو جای رایج برای زمین‌خوردن دارد که این درس دقیقاً برایشان است:

اول، **`.map()` و `.and_then()` قابلِ تعویض نیستند**، و اشتباه‌گرفتنشان محصولی می‌سازد که کامپایلر رد می‌کند — یک `Option` تویِ یک `Option` دیگر. این پرتکرارترین گیرِ تازه‌کارها با این بخش از زبان است.

دوم، بعضی از این متدها استدلالشان را **همیشه** اجرا می‌کنند، حتی وقتی لازم نیست، و بعضی دیگر فقط **وقتی لازم شود**. این تفاوت هزینه‌ی واقعی دارد و در امضای متد پنهان است — باید بدانی کدام‌یک کدام است.

پس این درس دو چیز به تو می‌دهد: واژگانِ ترکیب‌گرها، و قضاوتِ اینکه کِی از آن‌ها استفاده کنی و کِی نه.

---

## مفهوم

### یادآوری: `match` روی `Option`

```rust
let age: Option<u32> = Some(25);
let described = match age {
    Some(a) => format!("{a} years old"),
    None => "unknown age".to_string(),
};
println!("{described}");
```

```text
25 years old
```

این همانی است که در ۱.۶.۱ یاد گرفتی: دو شاخه، هردو الزامی. کاملاً درست است. سؤالِ این درس این نیست که آیا `match` درست است — همیشه هست — سؤال این است که آیا برای *این* تبدیلِ کوچک بهترین ابزار است.

### کلوژر، در یک جمله

قبل از این‌که به سراغِ ترکیب‌گرها برویم، یک چیز کوچک لازم داری: هر ترکیب‌گری که می‌بینی یک **کلوژر (closure)** به‌عنوانِ آرگومان می‌گیرد.

```rust
let double = |x: i32| x * 2;
println!("double(21) = {}", double(21));
```

```text
double(21) = 42
```

کلوژر فقط یک تابعِ درجا است که به‌عنوانِ یک مقدار پاس می‌دهی — بدونِ اسم، بدونِ تعریفِ جدا. `|x: i32| x * 2` یعنی «تابعی که یک `i32` می‌گیرد و دو برابرش را برمی‌گرداند». همین‌قدر برای این درس کافی است؛ کلوژرها قواعدِ خودشان را دارند — چه چیزی را از محیط اطرافشان قرض می‌گیرند، چه چیزی را مالک می‌شوند — که درسِ کاملِ خودشان را در فاز ۲ می‌گیرند.

### `.map()` — تبدیلِ مقدارِ داخلِ `Some`

```rust
let n: Option<i32> = Some(4);
let doubled: Option<i32> = n.map(|x| x * 2);
println!("{doubled:?}");
```

```text
Some(8)
```

`.map()` کلوژرت را *فقط اگر* مقدار `Some` باشد اجرا می‌کند، نتیجه را دوباره در `Some` می‌گذارد، و اگر `None` بود دست‌نخورده برش می‌گرداند. دقیقاً همان کاری که در ۱.۶.۱ دیدی، فقط این بار به‌جای نوشتنِ خودِ `match`، اسمش را صدا می‌زنی.

### وقتی خودِ تبدیل ممکن است شکست بخورد

مشکل از جایی شروع می‌شود که تابعی که به `.map()` می‌دهی خودش یک `Option` برمی‌گرداند:

```rust
fn double_if_positive(n: i32) -> Option<i32> {
    if n > 0 { Some(n * 2) } else { None }
}

let n: Option<i32> = Some(4);
let mapped: Option<Option<i32>> = n.map(double_if_positive);
println!("{mapped:?}");
```

```text
Some(Some(8))
```

این را ببین: **`Some(Some(8))`**. یک `Option` تویِ یک `Option` دیگر. `.map()` دقیقاً کارِ خودش را کرد — کلوژر را اجرا کرد و نتیجه را در `Some` گذاشت — اما این بار نتیجه‌ی کلوژر خودش از قبل یک `Option` بود، پس یک لایه‌ی اضافه گرفتی.

راه‌حل `.and_then()` است:

```rust
let chained: Option<i32> = n.and_then(double_if_positive);
println!("{chained:?}");
```

```text
Some(8)
```

`.and_then()` هم دقیقاً همان کاری را می‌کند که `.map()` می‌کرد — کلوژر را روی مقدارِ داخلِ `Some` اجرا می‌کند — با یک فرق: به جای این‌که نتیجه را دوباره در `Some` بپیچد، نتیجه را همان‌طور که هست برمی‌گرداند. چون تابعت خودش قبلاً `Option` برگردانده، چیزی برای پیچیدنِ دوباره نیست. به این می‌گویند **تخت‌کردن (flattening)**: `.and_then()` یک لایه از تودرتویی را که `.map()` می‌ساخت، برمی‌دارد.

```senpai-visual
{"kind":"concept","labels":["Some(n)",".map(f)","Some(Some(_))",".and_then(f)","Some(_)"]}
```

قاعده‌ی عملی این است: به امضای تابعی که داری صدا می‌زنی نگاه کن.

| اگر تابعت برمی‌گرداند | استفاده کن از |
|---|---|
| یک مقدارِ ساده (`T`) | `.map()` |
| خودش یک `Option<T>` | `.and_then()` |

اگر مطمئن نیستی، نوعِ اعلام‌شده کمکت می‌کند: اگر `let x: Option<i32> = ...` نوشتی و کامپایلر گفت `Option<Option<i32>>` پیدا کرده، همان لحظه جوابت را گرفتی.

### `.filter()` — نگه‌داشتنِ مشروط

```rust
let n: Option<i32> = Some(8);
println!("{:?}", n.filter(|v| *v % 2 == 0));
println!("{:?}", n.filter(|v| *v % 2 != 0));
```

```text
Some(8)
None
```

`.filter()` یک شرط می‌گیرد. اگر مقدار `Some` باشد و شرط رویش `true` بدهد، همان `Some` برمی‌گردد؛ در غیرِ این صورت — چه `None` بود، چه شرط `false` داد — نتیجه `None` می‌شود.

یک نکته‌ی ظریف: کلوژرِ `.filter()` مقدار را نمی‌گیرد، **ارجاعی** به آن می‌گیرد — `v` اینجا از نوعِ `&i32` است، نه `i32`. برای همین `*v % 2` نوشتیم. این دقیقاً همان `Option<&T>`ای است که در ۱.۶.۱ دیدی: `.filter()` باید بتواند مقدار را، اگر شرط رد شد، هنوز داشته باشد تا در `None`اش بگذارد — پس فقط نگاهش می‌اندازد.

### `.or()` و `.or_else()` — جایگزینِ یک `Option` دیگر

```rust
let primary: Option<i32> = None;
println!("{:?}", primary.or(Some(99)));
println!("{:?}", primary.or_else(|| Some(100)));
```

```text
Some(99)
Some(100)
```

اگر `primary` از قبل `Some` بود، همان برمی‌گشت و آرگومان اصلاً به کار نمی‌آمد. این دو با `.unwrap_or()`/`.unwrap_or_else()` که الان می‌بینی خواهر‌ند: یکی جایگزینِ *مقدار* می‌دهد اگر `None` باشی، این دو جایگزینِ یک *`Option` کامل* می‌دهند.

### `.unwrap_or()`، `.unwrap_or_else()` و `.unwrap_or_default()` — مشتاق در برابر تنبل

اینجا جایی است که بی‌دقتی هزینه‌ی واقعی دارد. این دو خط را با هم مقایسه کن:

```rust
fn expensive_default() -> i32 {
    println!("    ... expensive_default() ran ...");
    42
}

let present: Option<i32> = Some(7);
let a = present.unwrap_or(expensive_default());
println!("unwrap_or:      {a}");
let b = present.unwrap_or_else(expensive_default);
println!("unwrap_or_else: {b}");
```

```text
    ... expensive_default() ran ...
unwrap_or:      7
unwrap_or_else: 7
```

هر دو خط جوابِ درست را دادند: `7`. ولی به خطِ اولِ خروجی نگاه کن — فقط یک‌بار چاپ شد، و آن هم برای `unwrap_or`، نه `unwrap_or_else`. `present` از قبل `Some(7)` بود؛ هیچ‌کدام از این دو واقعاً به مقدارِ پیش‌فرض احتیاج نداشت.

دلیلش در امضای متدهاست:

| متد | آرگومان | وقتی اجرا می‌شود |
|---|---|---|
| `.unwrap_or(x)` | یک **مقدار** | همیشه — قبل از این‌که `.unwrap_or` حتی صدا زده شود |
| `.unwrap_or_else(f)` | یک **کلوژر** | فقط اگر `Option` واقعاً `None` باشد |

`present.unwrap_or(expensive_default())` را که Rust می‌بیند، اول باید `expensive_default()` را اجرا کند تا آرگومان را داشته باشد — کارِ توابع همیشه همین‌طور است، مهم نیست خروجی‌اش بعداً دور ریخته شود. این را می‌گویند ارزیابیِ **مشتاق (eager)**. `.unwrap_or_else()` به‌جایش خودِ *تابع* را می‌گیرد، نه نتیجه‌اش، و فقط وقتی `None` را ببیند صدایش می‌زند — ارزیابیِ **تنبل (lazy)**.

برای `0` یا `""` فرقی حس نمی‌کنی. برای چیزی که یک پرس‌وجوی پایگاه‌داده است، یک فایل می‌خواند، یا فقط یک `Vec` بزرگ می‌سازد، فرق واقعی است — و کامپایلر دربارهٔ آن هیچ هشداری نمی‌دهد، چون هر دو کد کاملاً معتبرند.

> **قاعده‌ی عملی:** اگر مقدارِ پیش‌فرضت یک لیترالِ ساده است (`0`، `""`، `Vec::new()`)، `.unwrap_or()` خوب است. اگر پشتِ یک صدا‌زدنِ تابع است، `.unwrap_or_else(|| ...)` بنویس.

یک خواهرِ سوم هم هست، برای وقتی که پیش‌فرض همان چیزی است که نوع خودش پیش‌فرض می‌داند:

```rust
let n: Option<i32> = None;
println!("{}", n.unwrap_or_default());
```

```text
0
```

`.unwrap_or_default()` هیچ آرگومانی نمی‌گیرد — از صفتِ `Default` نوع استفاده می‌کند (برای `i32` همان صفر). نه مشتاق است نه تنبل به معنایی که بالا گفتیم؛ چیزی برای اجرا کردن نیست.

### `.ok_or()` و `.ok_or_else()` — پل به سمتِ `Result`

یک نگاه، بدونِ این‌که وارد جزئیاتش شویم:

```text
Some(8).ok_or("missing")            -> Ok(8)
None.ok_or("missing")               -> Err("missing")
None.ok_or_else(|| "missing".into()) -> Err("missing")
```

`.ok_or()` یک `Option<T>` را به یک `Result<T, E>` تبدیل می‌کند: `Some` می‌شود `Ok`، و `None` می‌شود `Err` با هر مقداری که بدهی. `.ok_or_else()` همان جفتِ تنبلش است — دقیقاً همان الگوی `unwrap_or`/`unwrap_or_else` که بالا دیدی. داستانِ کاملِ `Result` مالِ [۱.۶.۳](../03-result-and-question-mark/README.fa.md) است؛ همین‌قدر بدان که این پل وجود دارد.

### `.take()` و `.replace()` — تغییر در جا

این دو یک `&mut Option<T>` می‌خواهند، نه یک `Option<T>` — روی خودِ متغیر کار می‌کنند:

```rust
let mut slot: Option<String> = Some("draft".to_string());
let taken = slot.take();
println!("taken={taken:?} slot={slot:?}");
```

```text
taken=Some("draft") slot=None
```

`.take()` هرچه در `slot` بود را بیرون می‌کشد و `None` جایش می‌گذارد. `.replace()` برعکس: مقدارِ تازه می‌گذارد و آنچه قبلاً بود را برمی‌گرداند:

```rust
let mut counter: Option<u32> = Some(1);
let previous = counter.replace(2);
println!("previous={previous:?} counter={counter:?}");
```

```text
previous=Some(1) counter=Some(2)
```

هردو بدونِ کلون کردن کار می‌کنند — فقط جابه‌جاییِ مالکیت هستند، همان چیزی که در ۱.۲.۲ دیدی.

### `.as_ref()`، `.as_mut()`، `.cloned()` و `.copied()` — نگاه بدونِ گرفتنِ مالکیت

```rust
let owned: Option<String> = Some("hello".to_string());
let length: Option<usize> = owned.as_ref().map(|s| s.len());
println!("length={length:?} owned still usable: {owned:?}");
```

```text
length=Some(5) owned still usable: Some("hello")
```

`owned.as_ref()` یک `Option<&String>` می‌دهد — نگاهی به داخل، بدونِ حرکت دادنِ `String`. برای همین بعد از `.map()` هنوز می‌توانی `owned` را استفاده کنی. این دقیقاً همان `Option<&T>`ی است که در ۱.۶.۱ معرفی شد، اینجا کاربردش را می‌بینی.

`.as_mut()` همان کار را با دسترسیِ قابل‌تغییر می‌کند:

```rust
let mut editable: Option<String> = Some("hi".to_string());
if let Some(s) = editable.as_mut() {
    s.push('!');
}
println!("{editable:?}");
```

```text
Some("hi!")
```

و وقتی یک `Option<&T>` داری ولی یک `Option<T>` مالکیتی می‌خواهی، `.cloned()` (که کلون می‌کند) و `.copied()` (که کپی می‌کند، برای نوع‌های `Copy`) این راه را برعکس طی می‌کنند:

```rust
let cloned: Option<String> = owned.as_ref().cloned();
println!("{cloned:?}");
```

```text
Some("hello")
```

### `.zip()` — دو `Option` را یکی کن

```rust
let x: Option<i32> = Some(3);
let y: Option<i32> = Some(4);
println!("{:?}", x.zip(y));
println!("{:?}", x.zip(None::<i32>));
```

```text
Some((3, 4))
None
```

`.zip()` دو `Option` را در یک تاپل جفت می‌کند، اما فقط اگر **هردو** `Some` باشند. اگر یکی‌شان `None` باشد، کلِ نتیجه `None` است — تودرتویی در کار نیست، چون هیچ‌کدام از دو طرف تابعی برنمی‌گرداند که خودش `Option` باشد.

### `.is_some_and()` — یک شرط، بدونِ بازکردن

```rust
let n: Option<i32> = Some(8);
println!("{}", n.is_some_and(|v| v > 5));
println!("{}", n.is_some_and(|v| v > 100));
```

```text
true
false
```

`.is_some_and()` یک `bool` می‌دهد: `true` فقط اگر مقدار `Some` باشد *و* شرط رویش برقرار باشد. کوتاه‌تر از `matches!(n, Some(v) if v > 5)` است وقتی فقط یک `bool` می‌خواهی، نه خودِ مقدار.

### `match` یا زنجیره؟ قضاوت با توست

حالا که واژگان را داری، سؤالِ اصلی می‌ماند. این دو تابع دقیقاً یک کار می‌کنند:

```rust
fn greeting_match(name: Option<&str>) -> String {
    match name {
        Some(n) if !n.is_empty() => format!("Hello, {n}!"),
        _ => "Hello, stranger!".to_string(),
    }
}

fn greeting_combinator(name: Option<&str>) -> String {
    name.filter(|n| !n.is_empty())
        .map(|n| format!("Hello, {n}!"))
        .unwrap_or_else(|| "Hello, stranger!".to_string())
}
```

```text
name=Some("Sam") -> match="Hello, Sam!" combinator="Hello, Sam!" (equal: true)
name=Some("") -> match="Hello, stranger!" combinator="Hello, stranger!" (equal: true)
name=None -> match="Hello, stranger!" combinator="Hello, stranger!" (equal: true)
```

اینجا زنجیره برنده است: هر خط دقیقاً یک قدم است — فیلتر کن، تبدیل کن، پیش‌فرض بگذار — و در یک نگاه خوانده می‌شود.

ولی این یکی را ببین:

```rust
fn shipping_cost_combinator(weight_kg: Option<f64>, express: bool) -> f64 {
    weight_kg
        .map(|w| {
            if w <= 0.0 {
                0.0
            } else if express {
                w * 2.5 + 10.0
            } else {
                w * 1.2
            }
        })
        .unwrap_or(5.0)
}
```

سه قانونِ متفاوت — وزنِ صفر، ارسالِ فوری، ارسالِ عادی — همه داخلِ یک کلوژر تلنبار شده‌اند، و `express` هم باید از بیرون گرفته شود. نسخه‌ی `match`ش را ببین:

```rust
fn shipping_cost_match(weight_kg: Option<f64>, express: bool) -> f64 {
    match weight_kg {
        Some(w) if w <= 0.0 => 0.0,
        Some(w) if express => w * 2.5 + 10.0,
        Some(w) => w * 1.2,
        None => 5.0,
    }
}
```

```text
weight=Some(0.0), express=false -> match=0 combinator=0 (equal: true)
weight=Some(2.0), express=true -> match=15 combinator=15 (equal: true)
weight=Some(2.0), express=false -> match=2.4 combinator=2.4 (equal: true)
weight=None, express=true -> match=5 combinator=5 (equal: true)
```

هر دو یک جواب می‌دهند. ولی در نسخه‌ی `match`، هر قانون کنارِ شرطش نشسته — یک نگاه و می‌دانی چه‌وقت چه اتفاقی می‌افتد. زنجیره فقط منطق را جابه‌جا کرده، ساده‌اش نکرده. **وقتی چند قانونِ مستقل داری، به‌خصوص اگر یکی‌شان به چیزی بیرون از `Option` نیاز دارد، `match` برنده است.**

قاعده‌ی کلی: یک تبدیل با یک پیش‌فرض → زنجیره. چند قانونِ مجزا، یا شاخه‌هایی که با هم فرق ماهوی دارند → `match`. اگر مطمئن نیستی، هر دو را بنویس و آن‌که کوتاه‌تر *و* واضح‌تر است را نگه دار.

---

## دست‌به‌کد

```sh
cargo run -p p1-06-02-option-combinators --example 01-map-and-and-then
cargo run -p p1-06-02-option-combinators --example 02-match-vs-combinator
cargo run -p p1-06-02-option-combinators --example 03-eager-vs-lazy
cargo run -p p1-06-02-option-combinators --example 04-mutating-and-borrowing
```

بعد دو تای خراب:

```sh
cargo run -p p1-06-02-option-combinators --example 05-map-instead-of-and-then --features broken
cargo run -p p1-06-02-option-combinators --example 06-map-moves-the-option --features broken
```

بعد این‌ها را امتحان کن:

۱. در `03-eager-vs-lazy`، یک خطِ دیگر اضافه کن: `absent.unwrap_or_else(expensive_default)` را صدا بزن و ببین این‌بار پیام چاپ می‌شود یا نه، و چرا.
۲. در `04-mutating-and-borrowing`، به‌جای `x.zip(None::<i32>)`، بنویس `x.zip(y).zip(Some(true))`. نوعِ نتیجه چیست؟
۳. در `02-match-vs-combinator`، تابعِ `shipping_cost_combinator` را طوری بازنویسی کن که به‌جای یک `if`/`else if` تودرتو، دو `.filter()` پشتِ سرِ هم بگذارد. آیا خواناتر شد؟

---

## خطاهایی که خواهی دید

### `E0308` — `.map()` جایی که `.and_then()` لازم بود

```text
error[E0308]: mismatched types
  --> phase1-fundamentals\06-absence-and-failure\02-option-combinators\examples\05-map-instead-of-and-then.rs:22:32
   |
22 |     let timeout: Option<i32> = find_setting("timeout").map(double_if_positive);
   |                  -----------   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `Option<i32>`, found `Option<Option<i32>>`
   |                  |
   |                  expected due to this
   |
   = note: expected enum `Option<i32>`
              found enum `Option<Option<i32>>`
help: consider using `Option::expect` to unwrap the `Option<Option<i32>>` value, panicking if the value is an `Option::None`
   |
22 |     let timeout: Option<i32> = find_setting("timeout").map(double_if_positive).expect("REASON");
   |                                                                               +++++++++++++++++
```

**کامپایلر به چه اعتراض دارد:** خطِ اعلامِ نوع می‌گوید `timeout` باید `Option<i32>` باشد. سمتِ راست، `find_setting("timeout")` یک `Option<i32>` می‌دهد، و `.map(double_if_positive)` رویش اجرا می‌شود — ولی `double_if_positive` خودش `Option<i32>` برمی‌گرداند، نه `i32`. پس `.map()` آن `Option<i32>` را در یک `Some` دیگر می‌گذارد و نتیجه‌ی نهایی `Option<Option<i32>>` می‌شود. دو نوع با هم جور در نمی‌آیند.

**راه‌حل ۱:** `.map()` را به `.and_then()` تغییر بده — دقیقاً همان چیزی که چند بخش قبل دیدی.

**راه‌حل ۲:** `.map()` را نگه دار، ولی یک `.flatten()` بعدش اضافه کن — متدی که دقیقاً همین کار را می‌کند: یک `Option<Option<T>>` را به `Option<T>` تخت می‌کند.

**چرا این راه‌حل‌هاست:** `help`ی که کامپایلر می‌دهد — `.expect("REASON")` — کارِ درستی نیست؛ فقط کامپایلر را ساکت می‌کند و یک پنیکِ پنهان می‌گذارد جای مشکل. کامپایلر نمی‌داند *چرا* یک `Option` تودرتو ساختی، فقط می‌داند چطور می‌شود از شرش خلاص شد. تصمیمِ واقعی — `.and_then()` یا `.flatten()` — مالِ توست، چون فقط تو می‌دانی `double_if_positive` قرار بود چه‌کاره باشد.

### `E0382` — `.map()` مالکِ `Option` می‌شود

```text
error[E0382]: borrow of moved value: `name`
    --> phase1-fundamentals\06-absence-and-failure\02-option-combinators\examples\06-map-moves-the-option.rs:8:22
     |
   6 |     let name: Option<String> = Some("Sam".to_string());
     |         ---- move occurs because `name` has type `Option<String>`, which does not implement the `Copy` trait
   7 |     let length: Option<usize> = name.map(|s| s.len());
     |                                      ---------------- `name` moved due to this method call
   8 |     println!("name: {name:?}, length: {length:?}");
     |                      ^^^^ value borrowed here after move
     |
note: `Option::<T>::map` takes ownership of the receiver `self`, which moves `name`
help: consider calling `.as_ref()` to borrow the value's contents
     |
   7 |     let length: Option<usize> = name.as_ref().map(|s| s.len());
     |                                     +++++++++
help: consider calling `.as_mut()` to mutably borrow the value's contents
     |
   7 |     let length: Option<usize> = name.as_mut().map(|s| s.len());
     |                                     +++++++++
help: you can `clone` the value and consume it, but this might not be your desired behavior
     |
   7 |     let length: Option<usize> = name.clone().map(|s| s.len());
     |                                     ++++++++
```

(یک `note` که به فایلِ سورسِ محلیِ کتابخانه‌ی استاندارد اشاره می‌کرد اینجا حذف شده — مسیرش روی هر ماشین فرق می‌کند، ولی خودِ پیام همین‌جاست.)

**کامپایلر به چه اعتراض دارد:** این همان `E0382`ی است که در [۱.۲.۲](../../02-ownership-and-memory/02-move-semantics/README.fa.md) دیدی — استفاده بعدِ حرکت — ولی این‌بار مقصر یک متد است نه یک انتساب. `Option::map` مالکیتِ `self` را می‌گیرد. وقتی `name.map(...)` را صدا زدی، `name` **حرکت کرد** داخلِ آن صدا‌زدن، و دیگر چیزی برای `println!` نمانده که بخواندش.

**راه‌حل:** دقیقاً همان جدولی که در ۱.۲.۳ دیدی، اینجا هم صادق است — و کامپایلر خودش هر سه گزینه را پیشنهاد داده:

| اگر می‌خواستی | بنویس |
|---|---|
| فقط نگاهش کنی، مالکیتش را نگه داری | `name.as_ref().map(...)` |
| ویرایشش کنی در جا | `name.as_mut().map(...)` |
| یک نسخه‌ی جدا برای مصرف‌کردن | `name.clone().map(...)` |

**چرا این راه‌حل است:** توجه کن این خطا خودش دارد دقیقاً همان چیزی را که چند بخش قبل خواندی، به‌کار می‌گیرد — `.as_ref()` وقتی هنوز به مقدارِ اصلی احتیاج داری. این تصادف نیست: `.map()` روی `Option<String>` (یا هر نوعِ غیرِ `Copy`ای) همیشه مالکیت می‌گیرد، و این دقیقاً همان لحظه‌ای است که `.as_ref()` برایش ساخته شده.

---

## تمرین

### گرم‌کردن

<details>
<summary><code>n.map(f)</code> وقتی <code>f</code> خودش <code>Option&lt;T&gt;</code> برمی‌گرداند، چه نوعی برمی‌گرداند؟</summary>

`Option<Option<T>>`. `.map()` هرچه کلوژر برگرداند را دوباره در `Some` می‌گذارد، حتی اگر خودش از قبل یک `Option` بود.

</details>

<details>
<summary><code>opt.filter(|v| *v > 0)</code> چرا به <code>*v</code> نیاز دارد نه <code>v</code>؟</summary>

چون کلوژرِ `.filter()` یک ارجاع می‌گیرد (`&T`)، نه خودِ مقدار — باید بتواند مقدار را اگر شرط رد شد، هنوز داشته باشد.

</details>

<details>
<summary>در <code>present.unwrap_or(compute())</code>، آیا <code>compute()</code> اجرا می‌شود حتی اگر <code>present</code> از قبل <code>Some</code> باشد؟</summary>

بله، همیشه. `.unwrap_or()` یک مقدار می‌گیرد نه یک کلوژر، پس Rust باید آن آرگومان را قبل از صدا‌زدنِ خودِ متد بسازد — چه استفاده شود چه نه.

</details>

<details>
<summary>این کد کامپایل می‌شود؟

```rust
let a: Option<i32> = Some(5);
let b = a.map(|x| x + 1);
println!("{a:?} {b:?}");
```
</summary>

بله. `i32` نوعِ `Copy` است، پس `.map()` یک کپی از مقدارِ داخلِ `a` می‌گیرد، نه خودِ `a` را حرکت می‌دهد. `a` بعدش هنوز کار می‌کند — برخلافِ `Option<String>` که در بخشِ خطاها دیدی.

</details>

<details>
<summary>چرا <code>.zip()</code> هیچ‌وقت یک <code>Option</code> تودرتو نمی‌سازد؟</summary>

چون هیچ‌کدام از دو طرف تابعی نیست که خودش `Option` برگرداند — `.zip()` دو مقدار را مستقیم در یک تاپل می‌گذارد. مشکلِ تودرتویی فقط وقتی پیش می‌آید که کلوژری که به یک ترکیب‌گر می‌دهی، خودش از قبل `Option` برمی‌گرداند.

</details>

### تعمیر

`examples/05-map-instead-of-and-then.rs` را **دو** جور درست کن:

۱. با تغییرِ `.map()` به `.and_then()`.
۲. با نگه‌داشتنِ `.map()` و اضافه‌کردنِ `.flatten()` بعدش.

بعد `examples/06-map-moves-the-option.rs` را درست کن — با `.as_ref()`، طوری که خطِ `println!` هم به `name` و هم به `length` دسترسی داشته باشد.

### پیاده‌سازی

شش تابع در `src/lib.rs`:

```sh
cargo test -p p1-06-02-option-combinators
```

هر تابع طبیعتاً یک ترکیب‌گر است (یا زنجیره‌ای کوتاه از آن‌ها)، نه یک `match`. به `safe_half` با دقت نگاه کن: باید بین `.map()` و `.and_then()` انتخاب کنی، و انتخابِ اشتباه کامپایل نمی‌شود — دقیقاً همان خطایی که در بخشِ قبل دیدی.

### بساز

یک `pub fn describe_stock(count: Option<u32>) -> String` بنویس، با یک زنجیره‌ی ترکیب‌گر (نه `match`):

- اگر `count` غایب باشد یا صفر باشد: `"out of stock"`.
- اگر بینِ ۱ تا ۵ باشد: `"low stock: N left"` (با عددِ واقعی به‌جای `N`).
- در غیرِ این صورت: `"in stock"`.

بعد یک جمله بنویس: آیا این یکی را هم به‌شکلِ `match` می‌نوشتی؟ چرا این‌جا و نه در `greeting_match`/`greeting_combinator` بالا؟

### چالش (اختیاری)

**بخشِ یک.** یک `pub fn access_level(name: Option<&str>) -> String` بنویس: اگر `name` غایب یا خالی باشد `"guest"` برمی‌گرداند، وگرنه `"welcome, NAME"`. مقدارِ پیش‌فرض را از یک تابعِ جدا بگیر که وقتی صدا زده می‌شود چاپ می‌کند — و با چشمِ خودت ببین که این تابع فقط برای ورودی‌های غایب/خالی صدا زده می‌شود، نه برای هر ورودی.

**بخشِ دو.** یکی از توابعِ `src/lib.rs` را — هر کدام دوست داشتی — هم به‌شکلِ زنجیره (که همین الان نوشتی) و هم به‌شکلِ `match` بنویس. یک جمله بگو کدام را در یک ریویوی واقعی نگه می‌داشتی، و چرا.

**بخشِ سه.** این را حدس بزن، بعد اجرا کن: `Some(3).and_then(|x| Some(x).filter(|v| *v > 5)).or(Some(0))` چه می‌شود؟ سه ترکیب‌گر را زنجیر کرده — هر قدم را جدا دنبال کن.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به کارت می‌آید |
|---|---|---|
| کلوژر (closure) | تابعِ درجا که به‌عنوانِ مقدار پاس می‌دهی | آرگومانِ هر ترکیب‌گر |
| ترکیب‌گر (combinator) | متدی که رفتار روی `Option`/`Result` را بدونِ `match` می‌سازد | `.map()`، `.and_then()` و بقیه |
| `.map()` | تبدیل می‌کند، دوباره در `Some` می‌گذارد | وقتی تابع یک مقدارِ ساده برمی‌گرداند |
| `.and_then()` | تبدیل می‌کند، تودرتو نمی‌سازد | وقتی تابع خودش `Option` برمی‌گرداند |
| تخت‌کردن (flattening) | برداشتنِ یک لایه‌ی `Option<Option<T>>` | همان کاری که `.and_then()`/`.flatten()` می‌کنند |
| مشتاق (eager) | آرگومان همیشه اجرا می‌شود | `.unwrap_or()`، `.or()` |
| تنبل (lazy) | آرگومان فقط وقتی لازم شود اجرا می‌شود | `.unwrap_or_else()`، `.or_else()`، `.ok_or_else()` |
| `E0382` روی `.map()` | متد مالکیت گرفت، نه فقط انتساب | `.as_ref()`/`.as_mut()`/`.clone()` جواب‌های ممکن‌اند |

### الان می‌دانی

- `.map()` نتیجه‌ی کلوژر را در `Some` می‌پیچد؛ `.and_then()` نمی‌پیچد، چون کلوژرش خودش قبلاً پیچیده.
- کلوژر همان تابعِ درجاست که به ترکیب‌گرها پاس می‌دهی — `Fn`/`FnMut`/`FnOnce` و قواعدِ قرض‌گیری‌اش مالِ فاز ۲ است.
- `.unwrap_or(x)` همیشه `x` را می‌سازد؛ `.unwrap_or_else(f)` فقط وقتی `None` باشی `f` را صدا می‌زند.
- `.filter()` کلوژرش یک `&T` می‌گیرد، نه `T`.
- `.take()`/`.replace()` روی خودِ متغیر کار می‌کنند (`&mut Option<T>`)، نه یک کپی.
- `.as_ref()`/`.as_mut()` نگاه می‌اندازند بدونِ گرفتنِ مالکیت؛ `.cloned()`/`.copied()` مسیر را برعکس طی می‌کنند.
- `.ok_or()` پلِ `Option` به `Result` است — داستانش در ۱.۶.۳ کامل می‌شود.
- یک تبدیل با یک پیش‌فرض معمولاً زنجیره می‌خواهد؛ چند قانونِ مستقل معمولاً `match` می‌خواهد.

### بعداً کامل‌تر می‌بینی

- **`Result<T, E>` و عملگرِ `?`** — [۱.۶.۳](../03-result-and-question-mark/README.fa.md)
- **کلوژرها، `Fn`/`FnMut`/`FnOnce` و قرض‌گیریِ محیط** — [فاز ۲ — کلوژرها و صفت‌های Fn](../../../phase2-intermediate/02-iterators-and-closures/01-closures-and-fn-traits/README.fa.md)
- **`.map()`/`.filter()` روی ایتریتورها** (اسمِ متد یکی است، جنسش فرق می‌کند) — [فاز ۲ — آداپتورهای ایتریتور](../../../phase2-intermediate/02-iterators-and-closures/02-iterator-adapters/README.fa.md)
- **`.parse()` برای تبدیلِ رشته به عدد** — [۱.۶.۳](../03-result-and-question-mark/README.fa.md)
- **سیاستِ پنیک: کِی `panic!` درست است و کِی نه** — [۱.۶.۴](../04-panic-vs-result/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا `n.map(f)` وقتی `f` خودش `Option` برمی‌گرداند، یک `Option<Option<T>>` می‌سازد؟
- کلوژر در یک جمله چیست؟
- چرا `present.unwrap_or(compute())` مقدارِ `compute()` را می‌سازد حتی اگر لازمش نداشته باشد؟
- چرا کلوژرِ `.filter()` یک `&T` می‌گیرد نه `T`؟
- یک وضعیت بگو که `match` را به زنجیره ترجیح می‌دهی، و بگو چرا.
- `.ok_or()` چه چیزی را به چه چیزی تبدیل می‌کند؟

---

## بیشتر

- [مستنداتِ `std::option::Option`](https://doc.rust-lang.org/std/option/enum.Option.html) — لیستِ کاملِ متدها، با امضای دقیقِ هرکدام.
- [کتابِ Rust — کلوژرها](https://doc.rust-lang.org/book/ch13-01-closures.html) — همین الان مقدمه‌اش را دیدی؛ نسخه‌ی کامل مالِ فاز ۲ است، ولی خواندنش الان هم ضرری ندارد.
- [`clippy::manual_map`](https://rust-lang.github.io/rust-clippy/master/#manual_map) و [`clippy::unnecessary_lazy_evaluations`](https://rust-lang.github.io/rust-clippy/master/#unnecessary_lazy_evaluations) — دو لینتی که دقیقاً همین قضاوت‌های امروز را خودکار می‌کنند.

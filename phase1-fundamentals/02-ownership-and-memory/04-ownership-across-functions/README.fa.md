# ۱.۲.۴ — مالکیت در عبور از توابع

## در یک نگاه

بعد از این درس می‌توانی:

- بگویی وقتی یک `String` را به یک تابع پاس می‌دهی چه اتفاقی می‌افتد، و چرا بعدش دیگر مالِ تو نیست.
- تابعی بنویسی که مالکیت را می‌گیرد، و تابعی که مالکیت را می‌سازد و می‌دهد.
- بگویی `fn f(x: String)` و `fn f(x: &str)` هرکدام از صداکننده چه می‌خواهند.
- بگویی چرا الگوی «مقدار را بگیر و پسش بده» کار می‌کند و چرا هیچ‌کس نمی‌نویسدش.

**زمان:** حدود ۴۰ دقیقه · **پیش‌نیاز:** [۱.۲.۳ — `Clone` و `Copy`](../03-clone-and-copy/README.fa.md)

---

## چرا اهمیت دارد

هیچ قاعده‌ی تازه‌ای در این درس نیست.

پاس دادنِ یک مقدار به تابع دقیقاً همان چیزی است که `let b = a;` بود: یک حرکت. برگرداندنش از تابع هم همان است، در جهتِ عکس. تمام. زبان برای آرگومان‌ها مفهومِ جداگانه‌ای ندارد.

پس چرا یک درسِ کامل؟ چون **همین‌جاست که مالکیت از یک ایده تبدیل می‌شود به چیزی که در هر خطی که می‌نویسی تصمیم می‌خواهد.** هر امضایی که می‌نویسی یک مطالبه از صداکننده است، و تا اینجا هیچ‌وقت مجبور نبوده‌ای دقتِ آن مطالبه را انتخاب کنی.

و یک چیزِ دوم: این درس عمداً به یک بن‌بست می‌رسد. با ابزارهایی که تا اینجا داری، تابعی که فقط می‌خواهد چیزی را *بخواند* مجبور است مالکیتش را بگیرد و بعد پسش بدهد. کار می‌کند و زشت است. **آن زشتی همان استدلالِ ماژولِ بعدی است**، و بهتر است قبل از دیدنِ راه‌حل، خودِ مسئله را حس کنی.

---

## مفهوم

### پاس دادن یک حرکت است

```rust
let name = String::from("Matin");
let length = consume(name);
```

```text
length:     5
```

`consume` یک `String` را با مقدار می‌گیرد، پس این فراخوانی `name` را به داخلش **حرکت** می‌دهد. بعد از این خط `name` دیگر یک اتصالِ معتبر نیست.

هیچ چیزِ تازه‌ای اینجا نیست. اگر `let moved = name;` نوشته بودی هم همین می‌شد. آرگومانِ تابع فقط یک اتصالِ دیگر است که مقدار به آن حرکت می‌کند.

و همان استثنا برقرار است:

```rust
let count = 5_i32;
let doubled = double(count);
println!("count:      {count}");
```

```text
count:      5
doubled:    10
```

`count` بعدش هنوز کار می‌کند، چون `i32` نوعِ `Copy` است و مالکِ هیچ چیزی نیست.

### تابعی که مقدار را می‌بلعد

```rust
fn consume(text: String) -> usize {
    text.len()
} // <- `text` اینجا رها می‌شود
```

آن آکولادِ بسته مهم است. `text` مالِ این تابع است، تابع تمام می‌شود، پس `text` رها می‌شود و بافرِ رویِ هیپش آزاد می‌شود — **داخلِ `consume`**، نه در `main`.

```rust
let temporary = String::from("this will not survive the call");
consume(temporary);
```

```text
that String was freed inside `consume`
```

این نشت نیست و اتفاقی هم نیست. تابع مالک شد، پس تابع مسئولِ آزاد کردنش شد.

### تابعی که مالکیت می‌سازد

```rust
fn build() -> String {
    String::from("made inside build()")
}
```

```text
built:      made inside build()
built   @:  0x269ef3ca270
```

اینجا هیچ تشریفاتی نیست و همین نکته است. مقدار داخلِ تابع ساخته می‌شود و مالکیتش به بیرون حرکت می‌کند. **هر مقدارِ تازه‌ای در برنامه‌ات از یک تابعِ این‌شکلی می‌آید.**

و طبیعتاً می‌شود هر دو را کرد: یکی را گرفت و چیزِ دیگری پس داد.

```rust
fn shout(text: String) -> String {
    text.to_uppercase()
}
```

### و حالا بن‌بست

فرض کن طولِ یک رشته را می‌خواهی **و** می‌خواهی خودِ رشته را هم نگه داری. با ابزارهای این ماژول، تابع مجبور است پسش بدهد:

```rust
fn measure_and_return(text: String) -> (usize, String) {
    let length = text.len();
    (length, text)
}
```

```rust
let (length, name) = measure_and_return(name);
```

```text
length:     5
still ours: Matin
```

کار می‌کند. حالا دو بار صدایش بزن:

```rust
let (length, name) = measure_and_return(name);
let (bytes, name) = measure_and_return(name);
```

```text
again:      5 5 Matin
```

هر بار یک تاپلِ تازه، یک بازکردنِ تازه، یک اتصالِ تازه — برای تابعی که فقط می‌خواست بخواند.

و با دو آرگومان از قابلِ تحمل بودن هم می‌افتد:

```rust
fn total_length(first: String, second: String) -> (usize, String, String) {
    let total = first.len() + second.len();
    (total, first, second)
}
```

```text
total:      9
both back:  alpha beta

every one of those returns exists only to give the value back
```

سه مقدار تو، سه مقدار بیرون، برای یک جمعِ ساده.

> **این کد کار می‌کند و هیچ برنامه‌نویسِ Rustی نمی‌نویسدش.** هر کدام از آن مقدارهای بازگشتی فقط برای پس دادن آنجاست. ماژولِ بعدی دقیقاً همین را حذف می‌کند، و راه‌حلش را همین حالا هم در امضاها دیده‌ای: `&`.

### امضا چه می‌گوید

سه شکل، سه مطالبه‌ی متفاوت از صداکننده:

```rust
fn by_view(text: &str) -> usize { text.len() }
fn by_value(mut text: String) -> String { text.push('!'); text }
```

```text
by_view:    5
by_view:    9
still ours: hello
by_value:   goodbye!
```

| امضا | از صداکننده چه می‌خواهد | کِی درست است |
|---|---|---|
| `&str` | «فقط می‌خواهم نگاه کنم؛ نگهش دار» | خواندن — پیش‌فرضِ توست |
| `String` | «باید نگهش دارم یا عوضش کنم» | ذخیره کردن، تغییر دادن، یا برگرداندنِ نسخه‌ی عوض‌شده |
| `&String` | تقریباً هیچ‌وقت | همان `&str` است با محدودیتِ اضافه |

آن سطرِ آخر ارزشِ توضیح دارد: تابعی که `&String` می‌گیرد، یک رشته‌ی ثابت را قبول نمی‌کند — صداکننده باید اول یک `String` بسازد. تابعی که `&str` می‌گیرد هر دو را قبول می‌کند. **`&String` هیچ چیزی نمی‌خرد و یک دسته صداکننده را کنار می‌گذارد.**

و هزینه‌ی انتخابِ اشتباه واقعی است:

```rust
println!("wasteful:   {}", by_value(owned.clone()));
```

```text
wasteful:   hello!
still ours: hello
```

چون `by_value` مالکیت خواست و صداکننده می‌خواست مقدارش را نگه دارد، مجبور شد کلون کند. یک تخصیص، صرفاً به این دلیل که امضا بیش از نیازش خواسته بود.

> **قاعده‌ای که با خودت می‌بری:** کوچک‌ترین چیزی را بخواه که کار را راه می‌اندازد. اگر فقط می‌خوانی، `&str` بگیر.

---

## دست‌به‌کد

```sh
cargo run -p p1-02-04-ownership-across-functions --example 01-passing-moves
cargo run -p p1-02-04-ownership-across-functions --example 02-giving-it-back
cargo run -p p1-02-04-ownership-across-functions --example 03-what-the-signature-says
```

بعد دو تای خراب:

```sh
cargo run -p p1-02-04-ownership-across-functions --example 04-used-after-passing --features broken
cargo run -p p1-02-04-ownership-across-functions --example 05-returning-a-local-reference --features broken
```

بعد این‌ها را امتحان کن:

۱. در `01-passing-moves`، خطِ کامنت‌شده‌ی `println!("{name}")` را باز کن. خطا را کامل بخوان — یک `note` دارد که مستقیماً به ماژولِ بعدی اشاره می‌کند.
۲. در `03-what-the-signature-says`، `by_view` را طوری عوض کن که `&String` بگیرد. حالا کدام فراخوانی می‌شکند؟
۳. در `02-giving-it-back`، یک تابعِ سه‌آرگومانی بنویس که هر سه را پس بدهد. تا کجا تحمل می‌کنی؟

---

## خطاهایی که خواهی دید

### `E0382` — استفاده بعد از پاس دادن

```text
error[E0382]: borrow of moved value: `name`
  --> examples\04-used-after-passing.rs:12:25
   |
 6 |     let name = String::from("Matin");
   |         ---- move occurs because `name` has type `String`, which does not implement the `Copy` trait
 7 |
 8 |     println!("length:  {}", consume(name));
   |                                     ---- value moved here
...
12 |     println!("name:    {name}");
   |                         ^^^^ value borrowed here after move
   |
note: consider changing this parameter type in function `consume` to borrow instead if owning the value isn't necessary
  --> examples\04-used-after-passing.rs:15:18
   |
15 | fn consume(text: String) -> usize {
   |    -------       ^^^^^^ this parameter takes ownership of the value
   |    |
   |    in this function
```

**کامپایلر به چه اعتراض دارد:** همان `E0382`ِ آشنا، ولی این بار حرکت یک فراخوانیِ تابع بود.

**راه‌حل:** به آن `note` نگاه کن، چون خودِ کامپایلر دارد جوابِ درست را می‌گوید: *«consider changing this parameter type in function `consume` to borrow instead if owning the value isn't necessary»*.

**چرا این راه‌حل است:** rustc دارد مقصر را از قربانی جدا می‌کند. مشکل جایی نیست که خطا نشان داده شده؛ **در امضای `consume` است.** آن تابع مالکیت خواست و فقط `.len()` صدا زد. دقت کن که `.clone()` را حتی پیشنهاد هم نداد — چون اینجا جوابِ درست نیست.

این `note` عملاً معرفیِ ماژولِ ۱.۳ است، از زبانِ خودِ کامپایلر.

### `E0106` — برگرداندنِ ارجاع به یک متغیرِ محلی

این خطایی است که وقتی می‌گیری که سعی کنی خودت راهِ خروج را بسازی.

```text
error[E0106]: missing lifetime specifier
  --> examples\05-returning-a-local-reference.rs:11:23
   |
11 | fn make_greeting() -> &String {
   |                       ^ expected named lifetime parameter
   |
   = help: this function's return type contains a borrowed value, but there is no value for it to be borrowed from
help: consider using the `'static` lifetime, but this is uncommon unless you're returning a borrowed value from a `const` or a `static`
   |
11 | fn make_greeting() -> &'static String {
   |                        +++++++
help: instead, you are more likely to want to return an owned value
   |
11 - fn make_greeting() -> &String {
11 + fn make_greeting() -> String {
   |
```

**کامپایلر به چه اعتراض دارد:** آن `String` داخلِ تابع ساخته شده و در انتهای تابع رها می‌شود. یک ارجاع به آن، به حافظه‌ای اشاره می‌کرد که همین حالا آزاد شده — همان «استفاده پس از آزادسازی»ِ [۱.۲.۱](../01-stack-and-heap/README.fa.md).

**راه‌حل:** `-> String` بنویس و مالکیت را بده. همان‌طور که پیشنهادِ دومِ کامپایلر می‌گوید.

**چرا این راه‌حل است:** به عبارتِ کلیدی در `help` دقت کن: *«there is no value for it to be borrowed from»*. مسئله این نیست که خواستی چیزی را قرض بدهی؛ این است که **چیزی برای قرض گرفتن از آن وجود ندارد**. مقدار همین‌جا زاده شد و همین‌جا می‌میرد.

آن `'static` که پیشنهاد داده اولین باری است که یک طولِ‌عمر می‌بینی، و کامپایلر خودش می‌گوید تقریباً همیشه جوابِ اشتباهی است. طولِ‌عمرها [فاز ۲](../../../phase2-intermediate/04-error-handling-and-lifetimes/README.fa.md) هستند. فعلاً درسِ عملی این است: **اگر مقدار را خودت ساختی، مالکیتش را بده.**

---

## تمرین

### گرم‌کردن

<details>
<summary>پاس دادنِ یک <code>String</code> به یک تابع چه می‌کند؟</summary>

حرکتش می‌دهد، دقیقاً مثلِ یک انتساب. بعد از آن فراخوانی، اتصالِ صداکننده دیگر معتبر نیست.

</details>

<details>
<summary>یک <code>String</code> که به تابعی پاس داده شده و برنگشته، کِی آزاد می‌شود؟</summary>

سرِ آکولادِ بسته‌ی همان تابع. تابع مالک شد، پس تابع مسئولِ آزاد کردنش است.

</details>

<details>
<summary><code>fn f(x: String)</code> و <code>fn f(x: &amp;str)</code> از صداکننده چه می‌خواهند؟</summary>

اولی می‌خواهد مقدارش را واگذار کند. دومی فقط می‌خواهد نگاه کند و صداکننده نگهش می‌دارد — و یک رشته‌ی ثابت را هم بدونِ هیچ تبدیلی قبول می‌کند.

</details>

<details>
<summary>چرا <code>&amp;String</code> در یک پارامتر تقریباً همیشه اشتباه است؟</summary>

چون همان `&str` است با محدودیتِ اضافه: یک رشته‌ی ثابت را قبول نمی‌کند، پس صداکننده مجبور می‌شود اول یک `String` بسازد. هیچ چیزی نمی‌خرد و صداکننده‌ها را کنار می‌گذارد.

</details>

<details>
<summary>چرا نمی‌شود از تابعی که یک <code>String</code> ساخته، به آن ارجاع برگرداند؟</summary>

چون آن `String` در انتهای تابع رها می‌شود، پس ارجاع به حافظه‌ی آزادشده اشاره می‌کرد. مالکیتش را بده.

</details>

<details>
<summary>وقتی یک <code>E0382</code> از یک فراخوانیِ تابع می‌گیری، اول کجا را نگاه می‌کنی؟</summary>

امضای همان تابع. اگر مالکیت خواسته و فقط دارد می‌خواند، مقصر امضاست نه محلِ فراخوانی — و کامپایلر خودش در یک `note` همین را می‌گوید.

</details>

### تعمیر

`examples/04-used-after-passing.rs` را **دو** جور درست کن:

۱. با کلون کردن سرِ محلِ فراخوانی.
۲. با عوض کردنِ امضای `consume`، همان‌طور که `note` پیشنهاد می‌دهد.

کدام‌شان بهتر است، و چند تخصیص هرکدام صرفه‌جویی می‌کند؟

بعد `examples/05-returning-a-local-reference.rs` را درست کن. کامپایلر دو راه پیشنهاد می‌دهد — آنی را بردار که خودش می‌گوید محتمل‌تر است، و بگو چرا آن یکی اینجا غلط است.

### پیاده‌سازی

پنج تابع در `src/lib.rs`:

```sh
cargo test -p p1-02-04-ownership-across-functions
```

چهار تا مالکیت می‌گیرند و یکی فقط قرض می‌گیرد. تستِ آن یکی کاری می‌کند که تستِ بقیه نمی‌تواند — **پیدایش کن.** همان تفاوت، کلِ استدلالِ ماژولِ ۱.۳ است.

### بساز

یک `pub fn shortest_of(values: Vec<String>) -> String` بنویس که کوتاه‌ترین رشته را برگرداند (بر حسبِ بایت؛ در تساوی اولی). `values` هرگز خالی نیست.

بعد جواب بده: به بقیه‌ی رشته‌ها چه شد؟ در چه نقطه‌ای از تابعت آزاد شدند؟

بعد نسخه‌ای بنویس که به‌جایش `values` را هم برگرداند، و بگو چرا این نسخه به‌طرزِ ناخوشایندی سخت‌تر است.

### چالش (اختیاری)

**بخشِ یک.** این کامپایل می‌شود؟ اگر نه، کدام خط؟

```rust
fn main() {
    let text = String::from("hello");
    let length = measure(&text);
    let owned = consume(text);
    println!("{length} {owned}");
}

fn measure(text: &String) -> usize { text.len() }
fn consume(text: String) -> usize { text.len() }
```

بعد `&String` را به `&str` عوض کن و ببین سرِ محلِ فراخوانی چه چیزی عوض می‌شود — یا نمی‌شود.

**بخشِ دو.** این تابع را بنویس و توضیح بده چرا مجاز است:

```rust
fn hand_it_back(text: String) -> String {
    text
}
```

چند بایت کپی شد؟ چند تخصیص اتفاق افتاد؟ حالا صدایش بزن و نشانیِ `as_ptr()` را قبل و بعدش چاپ کن.

**بخشِ سه.** این را اجرا کن و ترتیبِ خروجی را توضیح بده. (سرنخ: مقدارهایی که به تابع داده شده‌اند کِی رها می‌شوند؟)

```rust
fn main() {
    let a = String::from("a");
    let b = String::from("b");
    println!("before");
    eat(a);
    println!("middle");
    eat(b);
    println!("after");
}

fn eat(text: String) {
    println!("  eating {text}");
}
```

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به کارت می‌آید |
|---|---|---|
| پاس دادن با مقدار | حرکت به داخلِ تابع | `fn f(x: String)` |
| برگرداندنِ مالکیت | حرکت به بیرونِ تابع | هر تابعِ سازنده |
| مصرف کردن | تابع مالک می‌شود و رهایش می‌کند | `fn f(x: String) -> usize` |
| الگوی «پسش بده» | مقدار در تاپلِ بازگشتی برمی‌گردد | کاری که بدونِ قرض گرفتن مجبوری |
| `&str` در پارامتر | «فقط نگاه می‌کنم» | پیش‌فرضِ خواندنِ متن |
| `String` در پارامتر | «باید نگهش دارم» | ذخیره یا تغییر |
| `&String` در پارامتر | تقریباً هیچ‌وقت | `&str` با محدودیتِ اضافه |
| `E0106` | ارجاع به چیزی که نمی‌ماند | برگرداندنِ `&` به یک محلی |

### الان می‌دانی

- پاس دادنِ یک مقدار به تابع همان حرکتِ انتساب است، بدونِ هیچ قاعده‌ی تازه‌ای.
- مقداری که مصرف می‌شود و برنمی‌گردد، در انتهای همان تابع آزاد می‌شود.
- برگرداندن مالکیت را واگذار می‌کند، و مقدارهای تازه از همین‌جا می‌آیند.
- هر امضا یک مطالبه است؛ کوچک‌ترین چیزی را بخواه که کار را راه می‌اندازد.
- `&String` در یک پارامتر تقریباً همیشه باید `&str` باشد.
- نمی‌شود به مقداری که همین حالا در تابع ساخته شده ارجاع برگرداند.

### بعداً کامل‌تر می‌بینی

- **قرض گرفتن، که کلِ الگوی «پسش بده» را حذف می‌کند** — [۱.۳.۱ — ارجاع‌ها](../../03-borrowing-and-references/01-shared-and-mutable-refs/README.fa.md)
- **قواعدی که قرض گرفتن را امن نگه می‌دارند** — [۱.۳.۲](../../03-borrowing-and-references/02-borrow-checker-rules/README.fa.md)
- **کدی که هنگامِ رها شدن اجرا می‌شود** — [۱.۲.۵ — `Drop`](../05-drop-and-raii/README.fa.md)
- **`'static` و طولِ‌عمرهای واقعی** — [فاز ۲](../../../phase2-intermediate/04-error-handling-and-lifetimes/README.fa.md)

### می‌توانی توضیح بدهی؟

- پاس دادنِ یک `String` به تابع چه می‌کند؟
- مقداری که مصرف شده و برنگشته کِی آزاد می‌شود؟
- `fn f(x: String)` و `fn f(x: &str)` از صداکننده چه می‌خواهند؟
- چرا `&String` در یک پارامتر تقریباً همیشه اشتباه است؟
- چرا نمی‌شود به یک متغیرِ محلی ارجاع برگرداند؟
- الگوی «پسش بده» چیست و چرا کسی نمی‌نویسدش؟

---

## بیشتر

- [کتابِ Rust — مالکیت و توابع](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html#ownership-and-functions) — همین زمین، رسمی.
- [راهنمای APIهای Rust — پذیرشِ انعطاف‌پذیر](https://rust-lang.github.io/api-guidelines/flexibility.html) — درباره‌ی همین انتخابِ نوعِ پارامتر، از دیدِ طراحیِ کتابخانه.
- [`rustc --explain E0106`](https://doc.rust-lang.org/error_codes/E0106.html) — کوتاه، و حالا قابلِ فهم.

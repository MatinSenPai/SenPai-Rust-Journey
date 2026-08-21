# ۱.۳.۱ — ارجاع‌های اشتراکی و تغییرپذیر

## در یک نگاه

بعد از این درس می‌توانی:

- تابعی بنویسی که یک مقدار را بخواند بدون اینکه آن را بگیرد، و بگویی چرا صداکننده بعدش هنوز مالکش است.
- بین `T` و `&T` و `&mut T` در امضای یک تابع انتخاب کنی و بگویی هرکدام از صداکننده چه می‌خواهد.
- `E0596` را بخوانی و درست کنی، و بگویی چرا `&mut` اول باید از مالک اجازه بگیرد.
- بگویی `*` کجا لازم است و کجا نقطه‌ی متد خودش کار را انجام می‌دهد.

**زمان:** حدود ۴۵ دقیقه · **پیش‌نیاز:** [۱.۲.۴ — مالکیت در مرزِ تابع‌ها](../../02-ownership-and-memory/04-ownership-across-functions/README.fa.md)

---

## چرا اهمیت دارد

درسِ قبل با یک الگوی زشت تمام شد. می‌خواستی طولِ یک `Vec<String>` را بگیری و خودش را هم نگه داری، و تنها ابزارِ موجود این بود که تابع آن را پس بدهد:

```rust
fn total_length(lines: Vec<String>) -> (usize, Vec<String>) {
    let mut total = 0;
    for line in &lines {
        total += line.len();
    }
    (total, lines)
}

let (total, lines) = total_length(lines);
let (again, lines) = total_length(lines);
println!("total:       {total}");
println!("again:       {again}");
println!("still ours:  {}", lines.len());
```

```text
total:       14
again:       14
still ours:  3
```

جواب درست است و هیچ‌کس این را نمی‌نویسد. هر `Vec` که فقط قرار بود خوانده شود یک مقدارِ برگشتیِ اضافه به امضا اضافه می‌کند، و با دو آرگومان امضا از دست می‌رود.

اگر از پایتون می‌آیی، این کلاً به نظرت عجیب می‌آید: آنجا وقتی یک لیست را به تابعی می‌دهی، لیست از دستت نمی‌رود. همیشه یک ارجاع رد می‌شود. Rust هم همین را دارد — فقط مجبورت می‌کند در امضا بنویسی‌اش، و برایش قاعده می‌گذارد.

**تفاوت مهم با پایتون:** در پایتون هر ارجاعی می‌تواند شیء را عوض کند و هیچ‌کس جلویت را نمی‌گیرد، و شمارنده‌ی ارجاع تصمیم می‌گیرد شیء کِی بمیرد. در Rust ارجاع مالکِ چیزی نیست، در زمانِ کامپایل بررسی می‌شود، و اینکه اجازه‌ی نوشتن دارد یا نه در خودِ نوعش نوشته شده. تشبیه تا همین‌جا درست است.

---

## مفهوم

قرض‌گیری (Borrowing) یعنی استفاده از یک مقدار بدونِ گرفتنش. ابزارش یک کاراکتر است: `&`.

```senpai-visual
{"kind":"borrowing","labels":["مالک","قرض اشتراکی","قرض تغییرپذیر","پایان قرض"]}
```

### قرض دادن به‌جای واگذار کردن

```rust
fn total_length(lines: &Vec<String>) -> usize {
    let mut total = 0;
    for line in lines {
        total += line.len();
    }
    total
}

let lines = vec![String::from("alpha"), String::from("beta"), String::from("gamma")];
println!("total:       {}", total_length(&lines));
println!("still ours:  {}", lines.len());
```

```text
total:       14
still ours:  3
```

`&lines` مقدار را حرکت نمی‌دهد. یک **ارجاع (reference)** می‌سازد: یک فلش به `lines`، نه خودِ `lines`. تابع از روی فلش می‌خواند، فلش تمام می‌شود، و مالکیت اصلاً تکان نخورده.

به امضا نگاه کن: یک مقدارِ برگشتی، نه دو تا. آن `Vec`ای که در نسخه‌ی قبل پس داده می‌شد، از اول گرفته نشده بود.

> یک ارجاع مالکِ چیزی نیست. وقتی تمام می‌شود هیچ‌چیز آزاد نمی‌شود — `drop` کارِ آن یک مالک است، همان‌طور که در [۱.۲.۵](../../02-ownership-and-memory/05-drop-and-raii/README.fa.md) دیدی.

### فلش به مقدارِ خودِ صداکننده اشاره می‌کند

```rust
let view = &lines;
println!("lines   @:   {:p}", lines.as_ptr());
println!("view    @:   {:p}", view.as_ptr());
```

```text
lines   @:   0x1b720b98d30
view    @:   0x1b720b98d30
```

یک نشانی، نه دو تا. قرض گرفتن نه تخصیصی می‌گیرد نه بایتی کپی می‌کند؛ فقط یک فلش می‌سازد که یک کلمه‌ی ماشین است. برای همین است که فرستادنِ `&` به یک تابع، هر چقدر هم داده پشتش باشد، عملاً مجانی است.

### `*` — دنبال کردنِ فلش با دست

```rust
let answer = 42;
let view = &answer;
println!("through *:     {}", *view + 1);
```

```text
through *:     43
```

`*` **بازکردنِ ارجاع (dereference)** است: «مقداری که سرِ این فلش است». `view` یک `&i32` است، `*view` یک `i32`.

برای عملگرهای عددی Rust خودش هم راه دارد — `view + 1` هم کامپایل می‌شود، چون کتابخانه‌ی استاندارد `+` را برای ارجاع‌ها هم پیاده کرده. ولی برای **انتساب** هیچ راهِ فراری نیست: `*counter = 0` تنها شکلِ درست است، و شکلِ غلطش خطای دومِ بخشِ خطاهاست.

### نقطه، فلش را خودش دنبال می‌کند

```rust
let text = String::from("hello");
let look = &text;
println!("look.len():    {}", look.len());
println!("(*look).len(): {}", (*look).len());
```

```text
look.len():    5
(*look).len(): 5
```

دو خطِ یکسان. وقتی متدی صدا می‌زنی، کامپایلر خودش به تعدادِ لازم `*` می‌گذارد؛ به این کار **بازکردنِ خودکارِ ارجاع (auto-deref)** می‌گویند. برای همین در کدِ واقعیِ Rust کمتر `*` می‌بینی: بیشترِ کاری که با یک ارجاع می‌کنی صدا زدنِ متد است.

`{}` در `println!` هم همین کار را می‌کند، هر تعداد فلش که پشتِ هم باشد:

```rust
let arrow_to_arrow = &look;
println!("two arrows:    {arrow_to_arrow}");
println!("still counts:  {}", arrow_to_arrow.len());
```

```text
two arrows:    hello
still counts:  5
```

### یک ارجاعِ اشتراکی `Copy` است

```rust
let first = &text;
let second = first;
println!("both arrows:   {first} / {second}");
println!("one buffer:    {}", first.as_ptr() == second.as_ptr());
```

```text
both arrows:   hello / hello
one buffer:    true
```

این را در [۱.۲.۳](../../02-ownership-and-memory/03-clone-and-copy/README.fa.md) دیدی و حالا به کار می‌آید: `&T` نوعِ `Copy` است، پس انتسابش حرکت نیست. فلش کپی شد، نه چیزی که سرِ فلش است — و آن `true` نشان می‌دهد هر دو فلش هنوز به یک بافر اشاره می‌کنند.

`&mut T` تنها فلشی است که `Copy` **نیست**. چرایش موضوعِ [۱.۳.۲](../02-borrow-checker-rules/README.fa.md) است.

### `&mut T` — فلشی که اجازه‌ی نوشتن دارد

```rust
fn add_exclamation(text: &mut String) {
    text.push('!');
}

let mut greeting = String::from("hello");
add_exclamation(&mut greeting);
println!("after one:     {greeting}");
add_exclamation(&mut greeting);
add_exclamation(&mut greeting);
println!("after three:   {greeting}");
```

```text
after one:     hello!
after three:   hello!!!
```

`add_exclamation` هیچ‌چیز برنمی‌گرداند، چون چیزی برای پس دادن ندارد: `String`ی که عوض شد از اول مالِ `main` بود. **ارجاعِ تغییرپذیر (mutable reference)** همان الگوی «پس دادن» را کاملاً حذف می‌کند.

### `&mut` اول باید از مالک اجازه بگیرد

```rust
fn bump(counter: &mut i32) {
    *counter += 1;
}

let mut count = 10;
bump(&mut count);
bump(&mut count);
println!("count:         {count}");
```

```text
count:         12
```

آن `mut` در `let mut count` اختیاری نیست. برای ساختنِ `&mut count` مالک باید خودش تغییرپذیر باشد — نمی‌توانی مجوزی را قرض بدهی که خودت نداری. بردار‌ش و `E0596` می‌گیری، که مثالِ `04-owner-not-mut` و بخشِ بعدی است.

و دقت کن که `*counter += 1` نوشته شده، نه `counter += 1`. اینجا انتساب است، پس `*` واجب است.

### دو `&mut` هم‌زمان، به دو مقدارِ متفاوت

```rust
fn move_one(from: &mut i32, to: &mut i32) {
    *from -= 1;
    *to += 1;
}

let mut here = 5;
let mut there = 0;
move_one(&mut here, &mut there);
move_one(&mut here, &mut there);
println!("moved:         {here} {there}");
```

```text
moved:         3 2
```

دو ارجاعِ تغییرپذیر در یک لحظه، و کامپایلر ساکت است. قاعده درباره‌ی **یک مقدار** است، نه درباره‌ی شمردنِ فلش‌ها: `here` و `there` دو چیزِ جدا هستند.

### قرض گرفتنِ یک مجموعه، عنصر به عنصر

```rust
fn double_all(values: &mut Vec<i32>) {
    for value in values {
        *value *= 2;
    }
}

let mut values = vec![1, 2, 3];
let buffer = values.as_ptr();
double_all(&mut values);
println!("doubled:       {values:?}");
println!("same buffer:   {}", buffer == values.as_ptr());
```

```text
doubled:       [2, 4, 6]
same buffer:   true
```

`for value in values` روی یک `&mut Vec<i32>` هر دور یک `&mut i32` به تو می‌دهد — یعنی `value` خودش یک فلش است و برای نوشتن باید `*value` بنویسی. همان حلقه روی `&Vec<i32>` هر دور یک `&i32` می‌دهد، که فقط خواندنی است.

و آن `true` مهم است: بافرِ خودِ صداکننده عوض شد. نه کپی‌ای بیرون رفت، نه کپی‌ای برگشت.

### چند خواننده، یا یک نویسنده

```rust
let a = &lines;
let b = &lines;
let c = &lines;
println!("three readers: {} {} {}", a.len(), b.len(), c.len());
```

```text
three readers: 3 3 3
```

هر تعداد `&T` می‌توانند هم‌زمان زنده باشند، چون هیچ‌کدام نمی‌توانند چیزی را عوض کنند. چند خواننده همیشه امن است.

قاعده‌ی کامل یک جمله است و کلِ درسِ بعدی است:

> **در هر لحظه یا هر تعداد `&T` به یک مقدار می‌تواند وجود داشته باشد، یا دقیقاً یک `&mut T` — هرگز هر دو با هم.**

همین‌قدر برای امروز کافی است. اینکه کامپایلر دقیقاً چطور اندازه‌اش می‌گیرد، چه خطاهایی می‌دهد و چرا این قاعده مسابقه‌ی داده را غیرِممکن می‌کند، [۱.۳.۲](../02-borrow-checker-rules/README.fa.md) است.

### کدام‌یک در امضا بیاید

سه انتخاب، سه خواسته‌ی متفاوت از صداکننده:

| در امضا | یعنی | صداکننده بعدش |
|---|---|---|
| `&T` | فقط می‌خواهم نگاه کنم | مالک است و همه‌چیزش سرِ جایش است |
| `&mut T` | می‌خواهم مالِ تو را عوض کنم | مالک است، ولی مقدارش عوض شده |
| `T` | باید مالکش شوم | دیگر ندارَدش |

پیش‌فرض `&T` است. `&mut T` وقتی که واقعاً باید چیزی را عوض کنی. `T` فقط وقتی که تابع باید مقدار را نگه دارد، ذخیره‌اش کند، یا نسخه‌ی تغییریافته‌اش را برگرداند — همان چیزی که [۱.۲.۴](../../02-ownership-and-memory/04-ownership-across-functions/README.fa.md) گفت.

و آن الگوی `-> (usize, Vec<String>)`؟ بازنشسته شد. از این به بعد هر بار که دیدی یک تابع چیزی را فقط برای پس دادن برمی‌گرداند، بدان که یک `&` جا افتاده.

---

## دست‌به‌کد

```sh
cargo run -p p1-03-01-shared-and-mutable-refs --example 01-look-dont-take
cargo run -p p1-03-01-shared-and-mutable-refs --example 02-through-a-mutable-ref
cargo run -p p1-03-01-shared-and-mutable-refs --example 03-following-the-arrow
```

بعد سه تای خراب:

```sh
cargo run -p p1-03-01-shared-and-mutable-refs --example 04-owner-not-mut --features broken
cargo run -p p1-03-01-shared-and-mutable-refs --example 05-forgot-the-star --features broken
cargo run -p p1-03-01-shared-and-mutable-refs --example 06-taking-through-a-shared-ref --features broken
```

بعد این‌ها را امتحان کن:

۱. در `01-look-dont-take`، `&lines` را به `lines` تبدیل کن — یعنی `&` را بردار. چه خطایی می‌گیری، و از کدام درس می‌شناسی‌اش؟
۲. در `02-through-a-mutable-ref`، در `double_all` ستاره را بردار و `value *= 2` بنویس. متنِ خطا را با متنِ `05-forgot-the-star` مقایسه کن.
۳. در `03-following-the-arrow`، این خط را اضافه کن: `println!("{}", *look);`. کامپایل می‌شود. حالا این یکی را اضافه کن: `let owned = *look;`. نمی‌شود. تفاوتِ این دو در چیست؟

---

## خطاهایی که خواهی دید

### `E0596` — نمی‌شود به‌صورت تغییرپذیر قرض گرفت

```text
error[E0596]: cannot borrow `greeting` as mutable, as it is not declared as mutable
  --> examples\04-owner-not-mut.rs:10:21
   |
10 |     add_exclamation(&mut greeting);
   |                     ^^^^^^^^^^^^^ cannot borrow as mutable
   |
help: consider changing this to be mutable
   |
 8 |     let mut greeting = String::from("hello");
   |         +++
```

**کامپایلر به چه اعتراض دارد:** `&mut greeting` یعنی «اجازه‌ی نوشتن روی `greeting` را قرض بده». ولی `greeting` با `let` ساده بایند شده، یعنی خودِ مالک هم اجازه‌ی نوشتن ندارد. مجوزی که نداری قابلِ قرض دادن نیست.

**راه‌حل:**

```rust
let mut greeting = String::from("hello");
```

**چرا این راه‌حل است:** دقت کن خطا به خطِ ۱۰ اشاره می‌کند ولی راهنمایی‌اش به خطِ ۸ است — یعنی کامپایلر برگشته و *بایندِ اصلی* را پیدا کرده. این همان تفکیکی است که [۱.۱.۱](../../01-foundations/01-variables-mutability-shadowing/README.fa.md) گذاشت: `mut` خاصیتِ بایند است، نه خاصیتِ مقدار. و همین‌جا چیزِ خوبی به تو می‌دهد: در امضای هر تابع دقیقاً نوشته شده که آیا مقدارت را عوض می‌کند یا نه.

### `E0308` — انتساب به فلش، نه به مقصدش

```text
error[E0308]: mismatched types
  --> examples\05-forgot-the-star.rs:16:15
   |
15 | fn reset(counter: &mut i32) {
   |                   -------- expected due to this parameter type
16 |     counter = 0;
   |               ^ expected `&mut i32`, found integer
   |
help: consider dereferencing here to assign to the mutably borrowed value
   |
16 |     *counter = 0;
   |     +
```

**کامپایلر به چه اعتراض دارد:** `counter` یک `&mut i32` است، نه یک `i32`. نوشتنِ `counter = 0` یعنی «این فلش را با عددِ صفر عوض کن»، که بی‌معنی است. چیزی که می‌خواستی این بود: «چیزی که سرِ فلش است را صفر کن.»

**راه‌حل:**

```rust
*counter = 0;
```

**چرا این راه‌حل است:** انتساب همان‌جایی است که بازکردنِ خودکارِ ارجاع کمکت نمی‌کند. نقطه‌ی متد می‌تواند حدس بزند که منظورت مقصدِ فلش بوده؛ `=` نمی‌تواند، چون «فلش را عوض کن» هم یک کارِ کاملاً معتبر است. پس خودت باید بگویی کدام را می‌خواهی.

قاعده‌ی عملی: هرجا سمتِ چپِ `=` یک ارجاع است، `*` می‌خواهی.

### `E0507` — نمی‌شود چیزی را از پشتِ یک ارجاعِ اشتراکی بیرون کشید

```text
error[E0507]: cannot move out of `*text` which is behind a shared reference
  --> examples\06-taking-through-a-shared-ref.rs:14:16
   |
14 |     let mine = *text;
   |                ^^^^^ move occurs because `*text` has type `String`, which does not implement the `Copy` trait
   |
help: consider removing the dereference here
   |
14 -     let mine = *text;
14 +     let mine = text;
   |
help: consider cloning the value if the performance cost is acceptable
   |
14 -     let mine = *text;
14 +     let mine = text.clone();
   |
```

**کامپایلر به چه اعتراض دارد:** `*text` یک `String` است، و `String` نوعِ `Copy` نیست، پس `let mine = *text` یک حرکت است. ولی `text` فقط یک قرض است — مالکِ آن `String` جای دیگری است و هنوز آن را می‌خواهد. اگر این حرکت اجازه داشت، دو نفر مسئولِ یک بافر می‌شدند.

**راه‌حل:** بستگی دارد واقعاً چه می‌خواستی.

```rust
let mine = text.clone();
```

**چرا این راه‌حل است:** این خطا معمولاً یعنی سؤال را اشتباه پرسیده‌ای. اگر فقط قرار بود بخوانی‌اش، اصلاً `*` لازم نداشتی و همان `text` کافی بود. اگر واقعاً یک نسخه‌ی مستقل می‌خواهی، `.clone()` جوابِ درست است و [۱.۲.۳](../../02-ownership-and-memory/03-clone-and-copy/README.fa.md) قیمتش را به تو گفته.

و به فرقِ این با `E0507`ِ [۱.۲.۲](../../02-ownership-and-memory/02-move-semantics/README.fa.md) دقت کن: آنجا مقدار از یک مالک بیرون کشیده می‌شد، اینجا از پشتِ یک قرض. متنِ خطا هم دقیقاً همین را می‌گوید — `which is behind a shared reference`.

---

## تمرین

### گرم‌کردن

<details>
<summary>یک ارجاع مالکِ چیزی است؟ وقتی تمام می‌شود چه چیزی آزاد می‌شود؟</summary>

نه، و هیچ‌چیز. `drop` کارِ مالک است. پایانِ یک قرض فقط یعنی آن اجازه دیگر معتبر نیست.

</details>

<details>
<summary><code>&lines</code> چند بایت کپی می‌کند وقتی <code>lines</code> یک <code>Vec</code> با هزار رشته باشد؟</summary>

به اندازه‌ی یک فلش — یک کلمه‌ی ماشین. اندازه‌ی چیزی که سرِ فلش است هیچ ربطی ندارد.

</details>

<details>
<summary>چرا <code>let second = first;</code> وقتی <code>first</code> یک <code>&amp;String</code> است هر دو را زنده نگه می‌دارد؟</summary>

چون `&T` نوعِ `Copy` است. فلش کپی شد، نه `String`ی که سرِ فلش است. این را از [۱.۲.۳](../../02-ownership-and-memory/03-clone-and-copy/README.fa.md) می‌دانستی.

</details>

<details>
<summary>این کامپایل می‌شود؟ <code>let text = String::from("hi"); let r = &mut text;</code></summary>

نه. `E0596`. `text` با `let` ساده بایند شده، پس مالک هم اجازه‌ی نوشتن ندارد و چیزی برای قرض دادن نیست. `let mut text` لازم است.

</details>

<details>
<summary>در <code>fn bump(counter: &amp;mut i32)</code>، چرا <code>counter += 1</code> کار نمی‌کند ولی <code>*counter += 1</code> کار می‌کند؟</summary>

`counter` خودش یک فلش است. `counter += 1` یعنی «به فلش یک واحد اضافه کن»، که نوعش نمی‌خورد — `E0308`. `*counter` یعنی عددی که سرِ فلش است.

</details>

<details>
<summary>دو <code>&amp;mut i32</code> هم‌زمان زنده — همیشه خطاست؟</summary>

نه. اگر به دو مقدارِ متفاوت اشاره کنند کاملاً مجاز است. قاعده درباره‌ی یک مقدار است، نه درباره‌ی شمردن.

</details>

<details>
<summary><code>for value in values</code> وقتی <code>values</code> یک <code>&amp;mut Vec&lt;i32&gt;</code> است، هر دور چه چیزی به تو می‌دهد؟</summary>

یک `&mut i32`. برای نوشتن روی آن `*value` لازم داری. روی یک `&Vec<i32>` همان حلقه یک `&i32` می‌دهد که فقط خواندنی است.

</details>

### تعمیر

هر سه مثالِ خراب را درست کن:

۱. `examples/04-owner-not-mut.rs` — یک کلمه اضافه می‌کند. بعد از درست کردنش، `&mut` را از فراخوانی بردار و ببین حالا چه می‌گوید.
۲. `examples/05-forgot-the-star.rs` — یک کاراکتر اضافه می‌کند. بعد `counter += 1` را هم امتحان کن و پیامِ خطا را با قبلی مقایسه کن.
۳. `examples/06-taking-through-a-shared-ref.rs` — **دو** جور درستش کن: یک بار طوری که یک `String` مستقل بگیری، یک بار طوری که اصلاً `String` جدیدی ساخته نشود (امضای تابع را عوض کن تا فقط طول را برگرداند).

روی نسخه‌ی دوم `cargo clippy` بگیر. درباره‌ی `&String` چیزی می‌گوید. حق با اوست و جوابِ کامل در [۱.۴.۱](../../04-text-and-strings/01-string-vs-str/README.fa.md) است؛ فعلاً فقط بخوانش و رد شو.

### پیاده‌سازی

پنج تابع در `src/lib.rs`:

```sh
cargo test -p p1-03-01-shared-and-mutable-refs
```

دو تایشان فقط نگاه می‌کنند و سه تایشان می‌نویسند. هیچ‌کدام `.clone()` لازم ندارد و هیچ‌کدام چیزی را فقط برای پس دادن برنمی‌گرداند. در سه تایشان جایی `*` لازم می‌شود؛ پیدا کردنِ آن جا نصفِ تمرین است.

تست‌ها علاوه بر جوابِ درست، یک چیزِ دیگر را هم بررسی می‌کنند: اینکه آرگومانی که فقط قرض داده شده، بعد از فراخوانی هنوز دستِ صداکننده باشد.

### بساز

`examples/02-giving-it-back.rs` از [۱.۲.۴](../../02-ownership-and-memory/04-ownership-across-functions/README.fa.md) را بردار و هر تابعش را با قرض گرفتن بازنویسی کن.

بعد بشمار: چند مقدارِ برگشتی حذف شد؟ چند `let` در `main` ساده‌تر شد؟

بعد یک برنامه‌ی کوچکِ خودت بنویس: یک `Vec<String>` از سطرهای گزارش در `main`، و سه تابع رویش —

- یکی که تعدادِ سطرها را گزارش کند،
- یکی که یک سطرِ تازه اضافه کند،
- یکی که همه را خالی کند.

هر سه با ارجاع کار کنند. حالا نگاه کن: کدام‌شان `&` گرفت و کدام‌شان `&mut`؟ چند تایشان چیزی برگرداندند؟ اگر جوابِ آخر «یکی» است، درست نوشته‌ای.

### چالش (اختیاری)

**بخشِ یک.** این را اجرا کن و خطا را بخوان. اسم و کدِ خطا را یادداشت کن، بعد حدس بزن درسِ بعدی چه خواهد گفت:

```rust
let mut lines = vec![String::from("alpha")];
let reader = &lines;
lines.push(String::from("beta"));
println!("{}", reader.len());
```

**بخشِ دو.** این کامپایل نمی‌شود، و خطایش `E0382` است — همانی که از [۱.۲.۲](../../02-ownership-and-memory/02-move-semantics/README.fa.md) می‌شناسی. چرا اینجا؟ (سرنخ: `&mut T` نوعِ `Copy` نیست.)

```rust
let mut count = 10;
let first = &mut count;
let second = first;
*first += 1;
println!("{second}");
```

**بخشِ سه.** یک `fn` بنویس که یک `&Vec<String>` بگیرد و یک ارجاع به بلندترین رشته‌اش برگرداند. کامپایل می‌شود. حالا نسخه‌ای بنویس که **دو** `&Vec<String>` بگیرد و باز هم یک ارجاع برگرداند: `E0106` می‌گیری. متنِ راهنمایی‌اش را بخوان و آن کلمه‌ی تازه را یادداشت کن — [۱.۳.۳](../03-borrow-scopes-and-nll/README.fa.md) و بعد فاز ۲ سراغش می‌روند.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به کارت می‌آید |
|---|---|---|
| قرض‌گیری (borrowing) | استفاده از یک مقدار بدونِ گرفتنش | تقریباً هر امضای تابعی |
| ارجاعِ اشتراکی `&T` | فلشِ فقط‌خواندنی؛ هر تعداد هم‌زمان مجاز | حالتِ پیش‌فرض |
| ارجاعِ تغییرپذیر `&mut T` | فلشی که می‌شود از رویش نوشت | وقتی باید مقدارِ صداکننده را عوض کنی |
| `*` (dereference) | «مقداری که سرِ این فلش است» | همیشه سمتِ چپِ `=` |
| auto-deref | نقطه‌ی متد خودش `*` می‌گذارد | `look.len()` روی یک `&String` |
| `E0596` | مالک `mut` نیست | یک کلمه فاصله تا حل شدن |
| `E0507` پشتِ ارجاع | تلاش برای بیرون کشیدن از یک قرض | یعنی سؤال را اشتباه پرسیده‌ای |

### الان می‌دانی

- `&` یک فلش به مقدارِ صداکننده می‌سازد؛ نه چیزی کپی می‌شود، نه مالکیت جابه‌جا می‌شود.
- یک ارجاع مالکِ چیزی نیست، پس پایانش هیچ‌چیز را آزاد نمی‌کند.
- `&T` فقط‌خواندنی است و `Copy`؛ `&mut T` می‌تواند بنویسد و `Copy` نیست.
- `&mut x` فقط وقتی ممکن است که `x` خودش `mut` باشد.
- `*` را برای انتساب لازم داری؛ برای صدا زدنِ متد کامپایلر خودش می‌گذاردش.
- روی یک `&mut Vec<i32>` حلقه زدن هر دور یک `&mut i32` می‌دهد.
- الگوی «بگیر و پس بده» با یک `&` جایگزین می‌شود.

### بعداً کامل‌تر می‌بینی

- **قاعده‌ی کاملِ هم‌نامی و خطاهای `E0499` و `E0502`** — [۱.۳.۲ — قواعدِ بررسی‌کننده‌ی قرض](../02-borrow-checker-rules/README.fa.md)
- **اینکه یک قرض دقیقاً کِی تمام می‌شود** — [۱.۳.۳ — دامنه‌ی قرض و NLL](../03-borrow-scopes-and-nll/README.fa.md)
- **قرض گرفتنِ بخشی از یک مجموعه** — [۱.۳.۴ — برش‌ها](../04-slices/README.fa.md)
- **اینکه چرا `&str` تقریباً همیشه از `&String` بهتر است** — [۱.۴.۱ — `String` در برابر `str`](../../04-text-and-strings/01-string-vs-str/README.fa.md)
- **برگرداندنِ یک ارجاع از یک تابع، و کلمه‌ی «lifetime»** — [فاز ۲ — طولِ عمر و حذفش](../../../phase2-intermediate/04-error-handling-and-lifetimes/03-lifetime-basics-and-elision/README.fa.md)
- **تغییر دادن از پشتِ یک `&T`، وقتی واقعاً لازم است** — [فاز ۲ — `RefCell` و تغییرپذیریِ درونی](../../../phase2-intermediate/05-smart-pointers/03-refcell-and-interior-mutability/README.fa.md)

### می‌توانی توضیح بدهی؟

- «قرض گرفتن» را در یک جمله، بدونِ استفاده از کلمه‌ی «اشاره‌گر».
- چرا `&lines` برای یک `Vec` با هزار عنصر همان‌قدر ارزان است که برای یکی با سه عنصر.
- چرا `&mut x` لازم دارد که `x` خودش `mut` باشد.
- کِی `*` باید بنویسی و کِی لازم نیست.
- چرا `let mine = *text;` وقتی `text` یک `&String` است رد می‌شود.
- الگوی `-> (usize, Vec<String>)` از درسِ قبل حالا چه شکلی می‌شود.

---

## بیشتر

- [کتابِ Rust — ارجاع‌ها و قرض‌گیری](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html) — همین زمین، رسمی.
- [Rust by Example — قرض‌گیری](https://doc.rust-lang.org/rust-by-example/scope/borrow.html) — مثال‌های کوتاهِ بیشتر.
- [`rustc --explain E0596`](https://doc.rust-lang.org/error_codes/E0596.html) — عادت کن هر کدِ خطایی را که می‌بینی همین‌طور باز کنی.

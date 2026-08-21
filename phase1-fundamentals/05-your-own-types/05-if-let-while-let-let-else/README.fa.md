# ۱.۵.۵ — `if let`، `while let`، `let else`

## در یک نگاه

بعد از این درس می‌توانی:

- یک `match` را که فقط یک بازوی جدی دارد به `if let` تبدیل کنی — و بگویی با این تبدیل چه چیزی را از دست دادی.
- یک مجموعه را با `while let Some(x) = ...` تا ته خالی کنی و بگویی حلقه دقیقاً کجا می‌ایستد.
- با `let ... else` یک مقدار را بگیری یا از تابع بیرون بزنی، و توضیح بدهی چرا بلوکِ `else` حتماً باید واگرا شود.
- روبه‌روی یک کدِ واقعی تصمیم بگیری که کدام‌یک از این چهار شکل خواناتر است — و کِی هیچ‌کدام، و `match` بهتر است.

**زمان:** حدود ۴۵ دقیقه · **پیش‌نیاز:** [۱.۵.۴ — `match` به‌طورِ عمیق](../04-match-in-depth/README.fa.md) و [۱.۱.۵ — جریان کنترل](../../01-foundations/05-control-flow/README.fa.md)

---

## چرا اهمیت دارد

درسِ قبل `match` را به تو داد و همراهش یک تضمین: کامپایلر حالت‌ها را می‌شمارد و اگر یکی جا مانده باشد `E0004` می‌گیرد. آن تضمین گران‌بهاست و بی‌دلیل از دستش نمی‌دهی.

ولی بیشترِ کدهای واقعی فقط **یک** حالتِ جالب دارند. بقیه «کاری نکن» هستند، و نوشتنِ `None => {}` یک خطِ کامل است که هیچ نمی‌گوید — بدتر: شبیهِ تصمیمی به نظر می‌رسد که گرفته‌ای، در حالی که فقط داری کامپایلر را راضی می‌کنی.

و یک چیزِ دوم. در [۱.۱.۵](../../01-foundations/05-control-flow/README.fa.md) نگهبان یا guard clause را یاد گرفتی: حالتِ استثنایی را همان اولِ تابع تمام کن و برو، تا بقیه‌ی تابع لازم نباشد به آن فکر کند. ولی آنجا نگهبان روی یک **شرط** بود — `if parts == 0`. چیزی که در عمل می‌خواهی روی آن نگهبان بگذاری معمولاً یک **الگو** است: «اگر این `Some` است بازش کن، وگرنه برو». تا امروز ابزارش را نداشتی و مجبور بودی کلِ تابع را داخلِ یک `if let` فرو ببری.

این درس هر سه شکاف را پر می‌کند، و آخرش یک قاعده‌ی انتخاب به تو می‌دهد.

---

## مفهوم

### `if let` — یک `match` که فقط یک بازویش برایت مهم است

```rust
let rating: Option<u8> = Some(9);

match rating {
    Some(score) => println!("match:      rated {score}/10"),
    None => {}
}
```

```text
match:      rated 9/10
```

آن بازوی دوم را بلند بخوان: «و اگر امتیازی نبود، هیچ کاری نکن.» یک خطِ کامل خرج شد تا هیچ گفته شود. Rust برای همین یک شکلِ کوتاه‌تر دارد:

```rust
if let Some(score) = rating {
    println!("if let:     rated {score}/10");
}
```

```text
if let:     rated 9/10
```

`if let` یعنی: اگر مقدارِ سمتِ راست با الگوی سمتِ چپ جور درآمد، بلوک را اجرا کن و هر اسمی که الگو می‌بندد را ببند؛ اگر نه، از رویش رد شو. **دقیقاً همان `match` است، با بازوی `_ => {}` که ننوشته‌ای.**

دو نکته که همین اول باید بدانی:

- سمتِ چپِ `=` یک **الگو** است، همان الگویی که در [۱.۱.۳](../../01-foundations/03-compound-types-and-destructuring/README.fa.md) دیدی. هر چیزی که در یک بازوی `match` مجاز بود، اینجا هم مجاز است.
- اسمی که الگو می‌بندد — اینجا `score` — فقط **داخلِ بلوک** زنده است. بیرونِ آکولاد وجود ندارد، و یکی از سه خطای این درس دقیقاً همین است.

### `if let ... else` — بازویی که پاک کرده بودی، برمی‌گردد

وقتی حالتِ دیگر هم کار دارد، `else` همان بازو را برمی‌گرداند:

```rust
let missing: Option<u8> = None;
if let Some(score) = missing {
    println!("with else:  rated {score}/10");
} else {
    println!("with else:  not rated yet");
}
```

```text
with else:  not rated yet
```

و چون این یک `if` است، مثلِ هر `if` دیگری در Rust یک **عبارت** است و می‌ارزد:

```rust
let shown = if let Some(score) = rating { score } else { 0 };
println!("as a value: {shown}");
```

```text
as a value: 9
```

همان سه قاعده‌ی [۱.۱.۵](../../01-foundations/05-control-flow/README.fa.md) هنوز برقرارند: هر دو شاخه باید یک نوع بدهند، و وقتی به‌عنوانِ مقدار استفاده‌اش می‌کنی `else` اجباری است.

### چیزی که از دست دادی: جامعیت

اینجا بخشی است که معمولاً کسی نمی‌گویدش. `match` از نظرِ **جامعیت (exhaustiveness)** بررسی می‌شود؛ `if let` نمی‌شود. تفاوت روزی خودش را نشان می‌دهد که یک واریانتِ تازه اضافه کنی.

```rust
fn describe(status: &Status) -> String {
    match status {
        Status::Watching { episode } => format!("on episode {episode}"),
        Status::Completed { rating } => format!("finished, {rating}/10"),
        Status::Dropped => "dropped".to_string(),
    }
}
```

```rust
fn medal(status: &Status) -> String {
    if let Status::Completed { rating } = status {
        format!("{rating}/10")
    } else {
        "—".to_string()
    }
}
```

```text
Watching { episode: 7 }   on episode 7     —
Completed { rating: 9 }   finished, 9/10   9/10
Dropped                   dropped          —
```

حالا یک واریانتِ چهارم اضافه کن:

```rust
enum Status {
    Watching { episode: u32 },
    Completed { rating: u8 },
    Dropped,
    OnHold,
}
```

```text
error[E0004]: non-exhaustive patterns: `&Status::OnHold` not covered
  --> examples\04-the-silent-hole.rs:19:11
   |
19 |     match status {
   |           ^^^^^^ pattern `&Status::OnHold` not covered
   |
note: `Status` defined here
  --> examples\04-the-silent-hole.rs:7:6
   |
 7 | enum Status {
   |      ^^^^^^
...
11 |     OnHold,
   |     ------ not covered
```

`describe` کامپایل نمی‌شود و انگشتش را دقیقاً روی جای خالی گذاشته. `medal` **بی‌صدا کامپایل می‌شود**، اجرا می‌شود، و برای `OnHold` جوابِ `—` می‌دهد. شاید درست باشد، شاید نه — ولی کسی از تو نپرسید.

> **این معامله را آگاهانه بکن:** هر بار که یک `match` را به `if let` تبدیل می‌کنی، یک شبکه‌ی ایمنی را برمی‌داری. اگر «بقیه‌ی حالت‌ها» واقعاً یک دسته‌ی یکدست‌اند («هر چیزِ دیگری یعنی نه»)، معامله‌ی خوبی است. اگر فقط داری بازوهای خسته‌کننده را پنهان می‌کنی، معامله‌ی بدی است.

### `while let` — تا وقتی الگو می‌خورَد، ادامه بده

`Vec::pop` آخرین عنصر را برمی‌دارد و `Some(...)` می‌دهد، و وقتی وکتور خالی شد `None`. این دقیقاً همان شکلی است که `while let` برایش ساخته شده:

```rust
let mut stack = vec![1, 2, 3];
while let Some(top) = stack.pop() {
    println!("popped:     {top}   (left: {})", stack.len());
}
```

```text
popped:     3   (left: 2)
popped:     2   (left: 1)
popped:     1   (left: 0)
```

بدونِ `while let` مجبور بودی این را بنویسی:

```rust
loop {
    match stack.pop() {
        Some(top) => println!("popped:     {top}   (left: {})", stack.len()),
        None => break,
    }
}
```

```text
popped:     3   (left: 2)
popped:     2   (left: 1)
popped:     1   (left: 0)
```

همان خروجی، با یک `loop`، یک `match` و یک `break` که هیچ‌کدام حرفِ تازه‌ای نمی‌زنند. `while let` این پنج خط را یک خط می‌کند.

```senpai-visual
{"kind":"concept","labels":["pop()","Some(x)?","body","None","done"]}
```

نکته‌ای که همه اولش اشتباه می‌فهمند: حلقه **اولین باری که الگو نخورَد تمام می‌شود**. چیزی رد نمی‌شود و دوباره امتحان نمی‌شود؛ `while let` هیچ `continue` پنهانی در خودش ندارد:

```rust
let mut countdown: u32 = 3;
while let Some(next) = countdown.checked_sub(1) {
    println!("countdown:  {next}");
    countdown = next;
}
```

```text
countdown:  2
countdown:  1
countdown:  0
```

روی صفر، `checked_sub(1)` جوابِ `None` می‌دهد و همان‌جا کار تمام است. و چون این یک حلقه‌ی معمولی است، `break` و `continue` هم سرِ جایشان کار می‌کنند.

### `let ... else` — بگیرش، یا برو

حالا به آن نگهبان برگردیم. این تابع باید یک عدد بدهد، ولی فقط برای یک واریانت جوابِ معناداری دارد:

```rust
fn gap_nested(entry: &Entry, latest: u32) -> u32 {
    if let Entry::Watching { episode } = entry {
        latest.saturating_sub(*episode)
    } else {
        0
    }
}
```

کار می‌کند، ولی کارِ اصلیِ تابع یک پله فرو رفته — و برای هر نگهبانِ بعدی یک پله‌ی دیگر فرو می‌رود. `let ... else` همان قاعده را صاف می‌نویسد:

```rust
fn gap_flat(entry: &Entry, latest: u32) -> u32 {
    let Entry::Watching { episode } = entry else {
        return 0;
    };
    latest.saturating_sub(*episode)
}
```

```text
latest episode out: 12
Watching { episode: 7 }      nested 5   flat 5
Completed                    nested 0   flat 0
PlanToWatch                  nested 0   flat 0
```

تفاوتِ اصلی در چشم نیست، در **دامنه** است: اسمی که `let ... else` می‌بندد داخلِ بلوک نمی‌ماند؛ در همان دامنه‌ای می‌نشیند که یک `let` معمولی می‌نشست. یعنی از خطِ بعد تا آخرِ تابع در دسترس است. `if let` این کار را نمی‌کند.

و حالا خویشاوندی‌اش را ببین:

```rust
// ۱.۱.۵ — نگهبان روی یک شرط
if parts == 0 {
    return 0;
}

// ۱.۵.۵ — نگهبان روی یک الگو
let Entry::Watching { episode } = entry else {
    return 0;
};
```

همان الگوی فکری است: **حالتی که جواب ندارد را همان اول تمام کن و برو.** فقط این بار نگهبان می‌تواند به شکلِ داده هم نگاه کند، نه فقط به یک `bool` — و در ضمن چیزی را که لازم داری برایت باز می‌کند.

### چرا `else` باید واگرا شود

اگر بلوکِ `else` را بنویسی و اجرا از تهش بیرون بیفتد، کامپایلر جلویت را می‌گیرد. متنِ کاملِ `E0308` در بخشِ خطاهاست؛ جمله‌ی مهمش این است: `expected !, found ()`.

دلیلش یک خطِ منطق است. `let Entry::Watching { episode } = entry else { ... };` قول داده که بعد از این خط، `episode` وجود دارد. اگر الگو نخورَد و بلوکِ `else` هم فقط چیزی چاپ کند و تمام شود، اجرا به خطِ بعد می‌رسد و `episode` هیچ مقداری ندارد. پس تنها راهِ درست این است که بلوکِ `else` **اصلاً به خطِ بعد نرسد**.

اسمِ این خاصیت **واگرا شدن (diverging)** است، و نوعش را در [۱.۱.۴](../../01-foundations/04-functions-and-expressions/README.fa.md) دیدی: `!`، نوعِ هرگز.

| `else` چه می‌گوید | چه چیزی را ترک می‌کند | نوعش |
|---|---|---|
| `return` یا `return x` | تابع | `!` |
| `break` | حلقه | `!` |
| `continue` | این دورِ حلقه | `!` |
| `panic!("...")` یا `todo!()` | برنامه | `!` |

آنجا `!` یک لطف بود: چون هرگز مقداری تولید نمی‌کند، می‌توانست جای هر نوعی بنشیند و `todo!()` هر امضایی را راضی کند. اینجا همان `!` **شرط** است؛ کامپایلر صریحاً می‌گوید انتظارش `!` بوده. همان نوع، همان معنا، این بار در نقشِ نگهبان.

### پس کدام، کِی؟

| می‌خواهی | بنویس |
|---|---|
| همه‌ی حالت‌ها کار دارند | `match` |
| فقط یک حالت کار دارد، بقیه واقعاً «هیچ» | `if let` |
| یک حالت کار دارد و بقیه یک کارِ مشترک | `if let ... else` |
| مقدار را بگیر وگرنه از اینجا برو | `let ... else` |
| تا وقتی الگو می‌خورَد ادامه بده | `while let` |
| سه شکل یا بیشتر، همه هم‌تراز | `match`، همیشه |

و شکلی که باید بشناسی و ننویسی:

```rust
if let Status::Watching { episode } = status {
    format!("on episode {episode}")
} else if let Status::Completed { rating } = status {
    format!("finished, {rating}/10")
} else {
    "dropped".to_string()
}
```

این یک `match` است که لباسِ مبدل پوشیده — و شبکه‌ی ایمنی‌اش را هم جا گذاشته. سه شکلِ هم‌تراز داری، پس یک `match` بنویس و بگذار کامپایلر بشمارد.

> `let ... else` و عملگرِ `?` مسئله‌های هم‌پوشان حل می‌کنند: هر دو «اگر نبود، از اینجا برو» را می‌نویسند. `?` مخصوصِ `Option` و `Result` است و خودش `return` می‌کند؛ `let ... else` با هر الگویی کار می‌کند و تو می‌گویی چطور برود. `?` کاملش در [۱.۶.۳](../../06-absence-and-failure/03-result-and-question-mark/README.fa.md) است.

---

## دست‌به‌کد

```sh
cargo run -p p1-05-05-if-let-while-let-let-else --example 01-one-arm-match
cargo run -p p1-05-05-if-let-while-let-let-else --example 02-while-let-drain
cargo run -p p1-05-05-if-let-while-let-let-else --example 03-let-else-guard
cargo run -p p1-05-05-if-let-while-let-let-else --example 04-the-silent-hole
```

بعد سه تای خراب:

```sh
cargo run -p p1-05-05-if-let-while-let-let-else --example 05-refutable-let --features broken
cargo run -p p1-05-05-if-let-while-let-let-else --example 06-else-must-diverge --features broken
cargo run -p p1-05-05-if-let-while-let-let-else --example 07-binding-escaped --features broken
```

بعد این‌ها را امتحان کن:

۱. در `04-the-silent-hole`، واریانتِ `OnHold` را اضافه کن و دوباره بساز. کدام تابع شکایت کرد و کدام ساکت ماند؟
۲. در `02-while-let-drain`، آن `break` را با `continue` عوض کن. چه اتفاقی می‌افتد و چرا؟
۳. در `03-let-else-guard`، خطِ `return 0;` را با یک `println!` عوض کن. خطا را بخوان — دقیقاً همان `E0308`ای است که در بخشِ بعد می‌آید.

---

## خطاهایی که خواهی دید

### `E0005` — الگویی که ممکن است نخورَد، در یک `let` ساده

```text
error[E0005]: refutable pattern in local binding
 --> examples\05-refutable-let.rs:9:9
  |
9 |     let Some(score) = rating;
  |         ^^^^^^^^^^^ pattern `None` not covered
  |
  = note: `let` bindings require an "irrefutable pattern", like a `struct` or an `enum` with only one variant
  = note: for more information, visit https://doc.rust-lang.org/book/ch19-02-refutability.html
  = note: the matched value is of type `Option<u8>`
help: you might want to use `let...else` to handle the variant that isn't matched
  |
9 |     let Some(score) = rating else { todo!() };
  |                              ++++++++++++++++
```

**کامپایلر به چه اعتراض دارد:** یک `let` ساده قول می‌دهد که بعد از این خط، اسم‌های الگو وجود دارند. برای اینکه این قول همیشه راست باشد، الگو باید **همیشه** بخورَد. `Some(score)` همیشه نمی‌خورَد؛ ممکن است `None` بیاید.

اینجا دو واژه‌ی تازه‌ی Rust را می‌بینی: الگویی که ممکن است نخورَد **ردشدنی (refutable)** است، و الگویی که همیشه می‌خورَد **ردناشدنی (irrefutable)**. یک تاپل، یک ساختار، یک اسمِ ساده — همه ردناشدنی‌اند و در `let` مجازند. یک واریانتِ خاص از یک شمارشی ردشدنی است و نیست.

**راه‌حل:** یکی از این سه، بسته به اینکه واقعاً چه می‌خواهی:

```rust
if let Some(score) = rating {
    println!("score: {score}");
}
```

**چرا این راه‌حل است:** خودِ خطا راهِ دیگر را هم پیشنهاد می‌دهد و درست می‌گوید — به `else { todo!() }` در پیامش نگاه کن. یعنی «جای خالی را نشانت می‌دهم، تصمیمش با توست». اگر می‌خواهی وقتی چیزی نبود از تابع بیرون بزنی، `let ... else` جوابِ دقیق است؛ اگر می‌خواهی فقط رد شوی، `if let`؛ و اگر هر دو حالت کار دارند، `match`.

### `E0308` — بلوکِ `else` واگرا نمی‌شود

```text
error[E0308]: `else` clause of `let...else` does not diverge
  --> examples\06-else-must-diverge.rs:10:35
   |
10 |       let Some(score) = rating else {
   |  ___________________________________^
11 | |         println!("no rating yet");
12 | |     };
   | |_____^ expected `!`, found `()`
   |
   = note:   expected type `!`
           found unit type `()`
   = help: try adding a diverging expression, such as `return` or `panic!(..)`
   = help: ...or use `match` instead of `let...else`
```

**کامپایلر به چه اعتراض دارد:** آن بلوکِ `else` تمام می‌شود و اجرا از تهش می‌ریزد بیرون. خطِ بعد `score` را می‌خواهد و `score` بسته نشده. پس Rust می‌گوید نوعِ این بلوک باید `!` باشد و آنچه دید `()` بود.

**راه‌حل:**

```rust
let Some(score) = rating else {
    println!("no rating yet");
    return;
};
```

**چرا این راه‌حل است:** `return` نوعش `!` است، پس بلوک دیگر «تمام نمی‌شود» و قولِ `let` سرِ جایش می‌ماند. دقت کن که چاپ کردن ممنوع نیست — بلوکِ `else` می‌تواند هر کاری بکند، فقط باید **آخرش برود**.

و این جمله را دو بار بخوان: `expected !, found ()`. همان نوعِ هرگزِ [۱.۱.۴](../../01-foundations/04-functions-and-expressions/README.fa.md) است. آنجا `!` اجازه‌ای بود که هر امضایی را راضی می‌کرد؛ اینجا یک الزام است. راهنماییِ دومِ کامپایلر هم خواندنی است: «یا از `match` استفاده کن» — چون در `match` هر بازو مقدارِ خودش را می‌دهد و اصلاً چنین قولی در کار نیست.

### `E0425` — اسمی که داخلِ بلوک ماند

```text
error[E0425]: cannot find value `score` in this scope
  --> examples\07-binding-escaped.rs:15:25
   |
15 |     println!("outside: {score}");
   |                         ^^^^^ not found in this scope
```

**کامپایلر به چه اعتراض دارد:** `score` را یک `if let` بست، و آن اسم فقط داخلِ بلوکِ همان `if let` زنده است. بیرونِ آکولاد چنین اسمی وجود ندارد — نه اینکه خالی باشد؛ اصلاً نیست.

**راه‌حل:** اگر واقعاً بعد از بلوک هم لازمش داری، یعنی `if let` ابزارِ اشتباهی بود:

```rust
let Some(score) = rating else { return };
println!("outside: {score}");
```

**چرا این راه‌حل است:** این دقیقاً همان تفاوتی است که این دو شکل را از هم جدا می‌کند. `if let` یک بلوک است و اسمش داخلِ بلوک می‌ماند؛ `let ... else` یک `let` است و اسمش در دامنه‌ی جاری می‌نشیند. وقتی به این خطا خوردی، سؤالِ درست «چطور اسم را بیرون بکشم؟» نیست، «کدام‌یک از این دو را می‌خواستم؟» است.

---

## تمرین

### گرم‌کردن

این چه چاپ می‌کند؟

```rust
let mut stack = vec![1, 2, 3];
while let Some(top) = stack.pop() {
    if top == 2 {
        continue;
    }
    println!("{top}");
}
```

<details>
<summary>جواب</summary>

```text
3
1
```

`continue` فقط بقیه‌ی همین دور را رد می‌کند؛ حلقه ادامه می‌دهد چون `stack.pop()` هنوز `Some` می‌دهد. تنها چیزی که حلقه را تمام می‌کند، نخوردنِ الگوست.

</details>

<details>
<summary><code>if let</code> در برابرِ <code>match</code>: کدام تضمین را از دست می‌دهی؟</summary>

جامعیت. `match` نبودِ یک حالت را با `E0004` می‌گیرد؛ `if let` واریانتِ تازه را بی‌صدا داخلِ `else` می‌اندازد.

</details>

<details>
<summary>چرا <code>let Some(x) = v;</code> کامپایل نمی‌شود ولی <code>let (a, b) = pair;</code> می‌شود؟</summary>

چون `(a, b)` ردناشدنی است — یک تاپلِ دوتایی همیشه همین شکل را دارد. `Some(x)` ردشدنی است، چون `None` هم ممکن است. یک `let` ساده فقط الگوی ردناشدنی می‌پذیرد.

</details>

<details>
<summary>بلوکِ <code>else</code> در <code>let ... else</code> چه نوعی باید داشته باشد و چرا؟</summary>

`!`، نوعِ هرگز. چون اگر اجرا از تهِ آن بلوک بیرون بیاید، خطِ بعد اسمی را می‌خواهد که بسته نشده. پس بلوک باید برود: `return`، `break`، `continue` یا یک پنیک.

</details>

<details>
<summary><code>while let Some(x) = v.pop()</code> دقیقاً کِی می‌ایستد؟</summary>

اولین باری که `pop()` چیزی بدهد که با `Some(x)` جور در نیاید — یعنی `None`. نه یک دور زودتر، نه یک دور دیرتر، و هیچ عنصری هم رد نمی‌شود.

</details>

<details>
<summary>اسمی که <code>if let</code> می‌بندد تا کجا زنده است؟ و اسمی که <code>let ... else</code> می‌بندد؟</summary>

اولی تا انتهای بلوکِ `if let`. دومی تا انتهای دامنه‌ای که در آن نوشته شده — مثلِ هر `let` دیگری. همین تفاوت است که `let ... else` را برای نگهبان مناسب می‌کند.

</details>

### تعمیر

هر سه‌ی خراب‌ها را درست کن:

۱. `examples/05-refutable-let.rs` را **دو** جور: یک بار با `if let`، یک بار با `let ... else`. بعد بگو کدام‌شان با نیتِ آن برنامه جور است.
۲. `examples/06-else-must-diverge.rs` را طوری که پیامِ «no rating yet» همچنان چاپ شود.
۳. `examples/07-binding-escaped.rs` را **دو** جور: یک بار با بردنِ `println!` به داخلِ بلوک، یک بار با عوض کردنِ `if let` به `let ... else`. کدام‌شان چیزی جز قالب‌بندی را عوض می‌کند؟

### پیاده‌سازی

پنج تابع در `src/lib.rs`:

```sh
cargo test -p p1-05-05-if-let-while-let-let-else
```

هر کدام یکی از شکل‌های امروز را می‌خواهد. بعد از اینکه تست‌ها سبز شدند، یکی‌شان را با `match` بازنویسی کن و دو نسخه را کنارِ هم بگذار: کدام خواناتر است، و آیا نظرت برای هر پنج تا یکی است؟

### بساز

یک `pub fn shelf_report(entries: Vec<Entry>) -> String` بنویس که یک خط خلاصه بدهد: چند تا در حالِ تماشا، چند تا تمام‌شده، و بالاترین امتیازی که دیده — با قالبی که خودت انتخاب می‌کنی و در مستندِ تابع می‌نویسی.

بعد همان را دو بار بنویس: یک بار با `match` داخلِ حلقه، یک بار با چند `if let`. کدام کوتاه‌تر شد؟ و کدام‌یک وقتی یک واریانتِ تازه به `Entry` اضافه کنی، خودش به تو خبر می‌دهد؟

### چالش (اختیاری)

**بخشِ یک.** این چه چاپ می‌کند؟ حدس بزن، بعد اجرا کن:

```rust
let mut queue = vec![Some(1), None, Some(3)];
while let Some(item) = queue.pop() {
    println!("{item:?}");
}
```

آن `Some` بیرونی مالِ `pop()` است، نه مالِ عنصر. حالا همان را طوری بنویس که فقط عددها را چاپ کند و از خالی‌ها رد شود — و حلقه تا ته برود.

**بخشِ دو.** این را اجرا کن و هشدارش را کامل بخوان:

```rust
let n = 5;
if let x = n {
    println!("irrefutable: {x}");
}
```

کامپایلر می‌گوید این `if let` بی‌فایده است. با واژه‌های `E0005` توضیح بده چرا — و بگو اگر همین را `while let` بنویسی چه اتفاقی می‌افتد.

**بخشِ سه.** (به جلو نگاه می‌کند.) در ویرایشِ ۲۰۲۴ می‌شود چند شرط و چند الگو را با `&&` در یک `if let` زنجیر کرد؛ به این می‌گویند let-chains. این مخزن روی ویرایشِ ۲۰۲۱ است، پس اینجا کامپایل نمی‌شود — ولی یادداشتِ انتشارش را پیدا کن و بخوان، و بگو کدام‌یک از `if let`های تودرتوی امروزت را حذف می‌کرد.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به کارت می‌آید |
|---|---|---|
| `if let` | یک `match` با یک بازو | وقتی فقط یک حالت کار دارد |
| `if let ... else` | همان، با بازوی دوم | وقتی حالتِ دیگر هم کار دارد |
| `while let` | تا وقتی الگو می‌خورَد | خالی کردنِ یک مجموعه |
| `let ... else` | بگیرش، یا برو | نگهبان روی یک الگو |
| ردشدنی (refutable) | الگویی که ممکن است نخورَد | `match`، `if let`، `let ... else` |
| ردناشدنی (irrefutable) | الگویی که همیشه می‌خورَد | تنها چیزی که در `let` ساده مجاز است |
| واگرا شدن (diverging) | اجرا از اینجا رد نمی‌شود | بلوکِ `else` در `let ... else` |
| `E0005` | الگوی ردشدنی در `let` ساده | خودِ خطا `let...else` را پیشنهاد می‌دهد |
| `E0308` در `let...else` | انتظارِ `!` بود و `()` آمد | بلوکِ `else` باید برود |

### الان می‌دانی

- `if let` دقیقاً یک `match` است با بازوی «کاری نکن» که ننوشته‌ای.
- `if let` هم مثلِ `if` یک عبارت است و می‌تواند مقدار بدهد.
- تبدیلِ `match` به `if let` جامعیت را از تو می‌گیرد، و این معامله باید آگاهانه باشد.
- `while let` اولین باری که الگو نخورَد تمام می‌شود، نه یک دور دیرتر.
- `let ... else` اسم را در دامنه‌ی جاری می‌بندد؛ `if let` داخلِ بلوکش.
- بلوکِ `else` در `let ... else` باید واگرا شود، و نوعش `!` است — همان نوعِ هرگز.
- الگوی ردناشدنی همیشه می‌خورَد، و فقط همان در یک `let` ساده مجاز است.

### بعداً کامل‌تر می‌بینی

- **`Option` و همه‌ی کارهایی که با آن می‌شود کرد** — [۱.۶ — نبود و شکست](../../06-absence-and-failure/README.fa.md)
- **متدهایی که جای خیلی از این `if let`ها را می‌گیرند** — [۱.۶.۲ — ترکیب‌گرهای `Option`](../../06-absence-and-failure/02-option-combinators/README.fa.md)
- **عملگرِ `?`، خویشاوندِ نزدیکِ `let ... else`** — [۱.۶.۳ — `Result` و `?`](../../06-absence-and-failure/03-result-and-question-mark/README.fa.md)
- **پنیک، به‌عنوانِ یکی از راه‌های واگرا شدن** — [۱.۶.۴ — پنیک در برابرِ `Result`](../../06-absence-and-failure/04-panic-vs-result/README.fa.md)
- **الگوهای پیشرفته و جاهای دیگری که الگو ظاهر می‌شود** — [فاز ۲ — تطبیقِ الگو از نزدیک](../../../phase2-intermediate/08-rust-toolbox/01-pattern-matching-depth/README.fa.md)

### می‌توانی توضیح بدهی؟

- یک `if let` را به `match` ترجمه کن، بازو به بازو.
- وقتی یک `match` را `if let` می‌کنی چه چیزی را از دست می‌دهی؟ یک مثال بزن که واقعاً به آن بربخوری.
- `while let Some(x) = v.pop()` کِی می‌ایستد و چرا دقیقاً آنجا؟
- چرا بلوکِ `else` در `let ... else` باید واگرا شود؟ جوابت را با نوعِ هرگز بگو.
- تفاوتِ دامنه‌ی اسمی که `if let` می‌بندد با اسمی که `let ... else` می‌بندد چیست؟
- «ردشدنی» و «ردناشدنی» یعنی چه، و کدام‌یک در یک `let` ساده مجاز است؟
- یک موقعیت بگو که `match` از هر سه‌ی این‌ها بهتر است، و بگو چرا.

---

## بیشتر

- [کتابِ Rust — جریانِ کنترلِ مختصر با `if let`](https://doc.rust-lang.org/book/ch06-03-if-let.html) — همین زمین، رسمی.
- [مرجعِ Rust — عبارت‌های `if let`](https://doc.rust-lang.org/reference/expressions/if-expr.html) — قاعده‌ی دقیق، از جمله چیزهایی که اینجا نگفتیم.
- [یادداشتِ انتشارِ Rust 1.65](https://blog.rust-lang.org/2022/11/03/Rust-1.65.0.html) — جایی که `let ... else` آمد، همراه با دلیلش. کوتاه است و ارزشِ خواندن دارد.
- [`std::vec::Vec::pop`](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.pop) — همان متدی که کلِ `while let` امروز رویش سوار بود.

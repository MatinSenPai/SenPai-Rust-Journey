# ۲.۹.۳ — استریم‌ها

## در یک نگاه

بعد از این درس می‌توانی:

- توضیح بدهی `Stream` دقیقاً چه نسبتی با `Iterator` ([۲.۲.۴](../../02-iterators-and-closures/04-implementing-iterator/README.fa.md)) و `Future` ([۲.۸.۵](../../08-concurrency/05-futures-and-runtimes/README.fa.md)) دارد، امضایِ `poll_next` را از حفظ بنویسی، و بگویی از کدام کریت واقعاً می‌آید.
- یک استریم را با `while let Some(item) = stream.next().await` بپیمایی، و آداپتورهایِ `StreamExt` — مثلِ `.filter()`، `.map()`، `.take()` — را رویش زنجیر کنی، همان عادتِ آداپتورهایِ `Iterator` که در [۲.۲](../../02-iterators-and-closures/README.fa.md) دستت آمد.
- یک استریمِ واقعی بسازی که مقدارهایش را یکی‌یکی و در طولِ زمانِ واقعی تحویل می‌دهد، مصرفش کنی، و `.next().await`اش را با `select!` ([۲.۹.۲](../02-select-and-cancellation-safety/README.fa.md)) در برابرِ یک مهلتِ زمانی مسابقه بدهی.

**زمان:** حدود ۵۵ دقیقه · **پیش‌نیاز:**
[۲.۹.۲ — `select!` و ایمنیِ لغو](../02-select-and-cancellation-safety/README.fa.md)،
[۲.۲.۴ — پیاده‌سازیِ `Iterator` و `IntoIterator`](../../02-iterators-and-closures/04-implementing-iterator/README.fa.md)،
[۲.۸.۵ — Futureها و محیط‌های اجرا](../../08-concurrency/05-futures-and-runtimes/README.fa.md)

## چرا اهمیت دارد

[۲.۹.۱](../01-spawn-joinset-structured-concurrency/README.fa.md) و [۲.۹.۲](../02-select-and-cancellation-safety/README.fa.md) هر دو دورِ همان چیزی چرخیدند که [۲.۸.۵](../../08-concurrency/05-futures-and-runtimes/README.fa.md) تعریفش کرده بود: یک `Future`، یک مقدارِ *تنها* که یک‌بار، دیر یا زود، آماده می‌شود. `JoinSet` چند تا از این مقدارهایِ تنها را هم‌زمان می‌راند؛ `select!` رویِ چندتایشان مسابقه می‌دهد. ولی خیلی از دادهٔ async واقعی اصلاً یک مقدارِ تنها نیست — یک **دنباله** است: ردیف‌هایی که از یک پرس‌وجویِ پایگاه‌داده یکی‌یکی برمی‌گردند، پیام‌هایی که رویِ یک وب‌سوکت می‌رسند، خط‌هایی که از خروجیِ یک زیرفرآیند می‌آیند — هرکدام، در طولِ زمان.

[۲.۲](../../02-iterators-and-closures/README.fa.md) دقیقاً همین شکل را برایِ دنیایِ همگام حل کرد: `Iterator`. ولی `Iterator::next()` همگام است — اگر مقدارِ بعدی هنوز نیامده، صدا زدنش همان‌جا ریسمان را قفل می‌کند. برایِ یک دنبالهٔ async — دنباله‌ای که هر عنصرش شاید هنوز روی شبکه در راه باشد — این دقیقاً همان مشکلی است که `Future` برایِ یک مقدارِ تنها حلش کرد: به‌جایِ قفل‌کردن، `Pending` برگردان و کنترل را به محیط اجرا پس بده. `Stream` دقیقاً همین دو ایده را باهم ترکیب می‌کند: شکلِ `Iterator` را دارد، ولی مثلِ `Future` پول می‌شود.

## مفهوم

### `Stream` از کجا می‌آید

`Stream` صفتِ خودِ `tokio` نیست. از کریتِ `futures_core` می‌آید — کریتی کوچک و پایه‌ای که چند صفتِ بنیادیِ async را نگه می‌دارد، بدونِ اینکه هیچ اجراکننده یا هیچ آداپتورِ اضافه‌ای با خودش بیاورد. `tokio_stream` — کریتی که این درس استفاده می‌کند — همان صفت را دوباره صادر می‌کند و رویش آداپتورها را اضافه می‌کند؛ این دوره نیازی به کریتِ کاملِ `futures` ندارد.

```rust
pub trait Stream {
    type Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>>;
}
```

این را کنارِ دو صفتی که از قبل می‌شناسی بگذار:

- `Iterator::next(&mut self) -> Option<Self::Item>` — همگام، فوری: یا مقدار داری یا `None`.
- `Future::poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>` — همان مکانیزمِ پول‌شدنِ [۲.۸.۵](../../08-concurrency/05-futures-and-runtimes/README.fa.md)، برایِ *یک* مقدار.
- `Stream::poll_next` هر دو را باهم دارد: امضایِ پول‌شدنِ `Future`، ولی خروجی‌اش `Poll<Option<T>>` است نه `Poll<T>` — چون یک استریم، برخلافِ یک `Future`، ممکن است بارها `Ready` بدهد، هر بار یک عنصر، تا وقتی سرانجام `Ready(None)` بدهد و تمام شود.

```senpai-visual
{"kind":"async","labels":["poll_next اول: Ready(Some(item ۱))","poll_next دوم: Ready(Some(item ۲))","poll_next سوم: Ready(Some(item ۳))","poll_next چهارم: Ready(None) — تمام شد"]}
```

### `StreamExt` و ایدیومِ اصلی

خودِ `Stream` هیچ متدِ راحتی‌ای ندارد — دقیقاً مثلِ `Iterator`، همه‌چیز رویِ همان یک متدِ اجباری سوار می‌شود. `StreamExt` (از `tokio_stream`) همان نقشی را دارد که آداپتورهایِ `Iterator` در [۲.۲](../../02-iterators-and-closures/README.fa.md) داشتند — با یک فرقِ مهم: `StreamExt` تویِ prelude نیست، باید خودت `use`ش کنی.

ساده‌ترین راهِ ساختنِ یک استریم، تبدیلِ یک پیمایشگرِ معمولی است:

```rust
use tokio_stream::StreamExt;

let mut numbers = tokio_stream::iter(vec![10, 20, 30]);

while let Some(n) = numbers.next().await {
    println!("got {n}");
}
```

```text
got 10
got 20
got 30
```

`while let Some(item) = stream.next().await { ... }` دقیقاً همان ایدیومی است که از این‌جا تا آخرِ درس به‌کار می‌بریم. اگر با پایتون کار کرده باشی، معادلِ ذهنی‌اش `async for item in stream:` است — ولی Rust این نحو را ندارد؛ `StreamExt::next()` دقیقاً همان جای خالی را با یک متدِ معمولی پر می‌کند، نه با یک کلیدواژه‌ی تازه.

### آداپتورها زنجیر می‌شوند — دقیقاً مثلِ Iterator

عادتِ زنجیرکردنِ [۲.۲.۲](../../02-iterators-and-closures/02-iterator-adapters/README.fa.md) تقریباً بی‌کم‌وکاست منتقل می‌شود:

```rust
let mut doubled_evens = tokio_stream::iter(1..=10)
    .filter(|n| n % 2 == 0)
    .map(|n| n * 2);

while let Some(n) = doubled_evens.next().await {
    println!("{n}");
}
```

```text
4
8
12
16
20
```

مثلِ آداپتورهایِ `Iterator`، این‌ها هم **تنبل**‌اند: خودِ `.filter()`/`.map()` هیچ کاری نمی‌کنند، فقط یک استریمِ تازه توصیف می‌کنند؛ فقط `.next().await` است که واقعاً یک قدم جلو می‌برد.

### یک استریمِ واقعی: مقدارها در طولِ زمان

تا این‌جا `tokio_stream::iter` هر مقداری که داشت، همان لحظه آماده می‌داد. یک استریمِ واقعی معمولاً از یک گیرنده‌ی کانال یا داده‌ای که رویِ شبکه می‌رسد ساخته می‌شود؛ برایِ اینکه بدونِ شبکه هم رفتارِ واقعی‌اش را ببینی، `.then()` را — که برایِ هر عنصر یک `Future` اجرا می‌کند — با یک `sleep` واقعی ترکیب می‌کنیم:

```rust
let start = Instant::now();
let ticks = tokio_stream::iter([40u64, 40, 40]).then(|delay_ms| async move {
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    delay_ms
});
tokio::pin!(ticks);
```

ساختنِ `ticks` هنوز هیچ `sleep`ی را اجرا نکرده — یک استریم هم، درست مثلِ یک `Future`، تنبل است. `tokio::pin!` هم لازم است: `Future`ای که `.then()` از کلوژرِ ما می‌سازد، دقیقاً مثلِ ماشین‌وضعیتِ هر `async fn`ِ واقعی‌ای که [۲.۸.۵](../../08-concurrency/05-futures-and-runtimes/README.fa.md) نشانت داد، `Unpin` نیست — و `StreamExt::next()` به یک استریمِ `Unpin` نیاز دارد.

حالا مصرفش کن:

```rust
while let Some(delay_ms) = ticks.next().await {
    println!("tick (waited {delay_ms}ms) — elapsed so far: {:?}", start.elapsed());
}
```

```text
tick (waited 40ms) — elapsed so far: 43.4923ms
tick (waited 40ms) — elapsed so far: 89.5897ms
tick (waited 40ms) — elapsed so far: 137.5148ms
```

(عددهایِ دقیقِ «elapsed» رویِ ماشینِ خودت کمی فرق خواهند داشت — ولی همیشه تقریباً مضربی از ۴۰ میلی‌ثانیه می‌مانند، هیچ‌وقت نزدیکِ صفر.) هر `sleep` واقعاً اتفاق افتاد، یکی پس از دیگری — `.then()` عنصرِ بعدی را از استریمِ زیرین بیرون نمی‌کشد تا `Future`ِ عنصرِ فعلی تمام شود.

### `select!` رویِ یک استریم — دقیقاً مثلِ هر `Future` دیگری

[۲.۹.۲](../02-select-and-cancellation-safety/README.fa.md) به‌ات `select!` را داد: چند `Future` را هم‌زمان می‌گذارد رویِ خط، و هرکدام زودتر آماده شد همان برنده می‌شود. `stream.next()` هم خودش یک `Future` برمی‌گرداند — پس هیچ مکانیزمِ تازه‌ای لازم نیست، فقط یک شاخه‌ی دیگر تویِ همان `select!`:

```rust
loop {
    tokio::select! {
        item = ticks.next() => match item {
            Some(delay_ms) => println!("tick (waited {delay_ms}ms)"),
            None => {
                println!("stream ended");
                break;
            }
        },
        _ = tokio::time::sleep(Duration::from_millis(200)) => {
            println!("timed out waiting for the next tick");
            break;
        }
    }
}
```

```text
tick (waited 30ms)
tick (waited 30ms)
tick (waited 30ms)
stream ended
```

(این‌جا سه تیکِ ۳۰میلی‌ثانیه‌ای در برابرِ مهلتِ ۲۰۰میلی‌ثانیه‌ای مسابقه می‌دهند — با این فاصله، همیشه استریم برنده می‌شود؛ اگر مهلت را کوتاه‌تر از فاصله‌یِ بینِ تیک‌ها می‌گذاشتی، همیشه شاخه‌ی `sleep` می‌برد. ترتیبِ برنده‌شدنِ `select!` بینِ دو شاخه‌ای که واقعاً هم‌زمان آماده‌اند تضمین‌شده نیست؛ این‌جا فاصله عمداً آن‌قدر بزرگ گذاشته شده که نتیجه هر بار همین بماند.)

```senpai-visual
{"kind":"concurrency","labels":["ticks.next() await می‌شود","sleep(200ms) هم موازی await می‌شود","تیکِ استریم می‌رسد: زودتر از مهلت","چاپ کن، دوباره از اولِ select! بزن","استریم تمام شد: از حلقه بیرون بیا"]}
```

## دست‌به‌کد

```sh
cargo run -p p2-09-03-streams --example 01-iter-and-next
cargo run -p p2-09-03-streams --example 02-adapters-chained
cargo run -p p2-09-03-streams --example 03-time-spaced-stream
cargo run -p p2-09-03-streams --example 04-select-vs-timeout
```

بعد دوتای خراب:

```sh
cargo run -p p2-09-03-streams --example 05-forgot-streamext --features broken
cargo run -p p2-09-03-streams --example 06-for-loop-on-stream --features broken
```

بعد این‌ها را امتحان کن:

۱. در `02-adapters-chained`، ترتیبِ `.filter()` و `.map()` را عوض کن — اول `.map(|n| n * 2)`، بعد `.filter(|n| n % 2 == 0)`. خروجی همان است؟ چرا آره یا چرا نه؟
۲. در `03-time-spaced-stream`، سه تأخیرِ `[40, 40, 40]` را با `[10, 100, 10]` عوض کن. حدس بزن هر «elapsed so far» چیزی نزدیکِ کدام عدد می‌شود — بعد اجرا کن و ببین.
۳. در `04-select-vs-timeout`، عددِ `200` را به `20` عوض کن. حالا کدام شاخه هر بار می‌برد؟ چرا؟

## خطاهایی که خواهی دید

### `E0599` — متدی به نامِ `next` پیدا نشد

```text
error[E0599]: no method named `next` found for struct `tokio_stream::Iter<I>` in the current scope
   --> phase2-intermediate\09-async-in-practice\03-streams\examples\05-forgot-streamext.rs:11:33
    |
 11 |     while let Some(n) = numbers.next().await {
    |                                 ^^^^
    |
   ::: C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\tokio-stream-0.1.18\src\stream_ext.rs:144:8
    |
144 |     fn next(&mut self) -> Next<'_, Self>
    |        ---- the method is available for `tokio_stream::Iter<std::vec::IntoIter<{integer}>>` here
    |
    = help: items from traits can only be used if the trait is in scope
help: trait `StreamExt` which provides `next` is implemented but not in scope; perhaps you want to import it
    |
  8 + use tokio_stream::StreamExt;
    |
help: there is a method `try_next` with a similar name
    |
 11 |     while let Some(n) = numbers.try_next().await {
    |                                 ++++

For more information about this error, try `rustc --explain E0599`.
```

**کامپایلر به چه اعتراض دارد:** برخلافِ `Iterator`، که `.next()`اش رایگان و تویِ prelude است، `StreamExt` یک صفتِ معمولی است که باید صریحاً `use`اش کنی. کامپایلر حتی می‌بیند `tokio_stream::Iter<...>` واقعاً یک متدِ `next` دارد — فقط از صفتی می‌آید که تویِ اسکوپ نیست.

**راه‌حل:** خطِ پیشنهادیِ خودِ کامپایلر:

```rust
use tokio_stream::StreamExt;
```

**چرا این راه‌حل است:** با این `use`، متدهایِ `StreamExt` — `.next()`، `.map()`، `.filter()`، هرچه دیدی — همه رویِ هر `Stream`ای در دسترس می‌شوند؛ دقیقاً همان چیزی که پیامِ کمکیِ کامپایلر هم گفت.

### `E0277` — یک `Stream`، `Iterator` نیست

```text
error[E0277]: `tokio_stream::Iter<std::vec::IntoIter<{integer}>>` is not an iterator
  --> phase2-intermediate\09-async-in-practice\03-streams\examples\06-for-loop-on-stream.rs:11:14
   |
11 |     for n in numbers {
   |              ^^^^^^^ `tokio_stream::Iter<std::vec::IntoIter<{integer}>>` is not an iterator
   |
   = help: the trait `Iterator` is not implemented for `tokio_stream::Iter<std::vec::IntoIter<{integer}>>`
   = note: required for `tokio_stream::Iter<std::vec::IntoIter<{integer}>>` to implement `IntoIterator`

For more information about this error, try `rustc --explain E0277`.
```

**کامپایلر به چه اعتراض دارد:** `for x in value` به `IntoIterator` نیاز دارد، و `Stream` آن صفت را پیاده نمی‌کند — عمداً؛ چون پیمایشِ همگامِ `for` هیچ‌جایی برایِ `Pending` برگرداندن و کنترل را پس‌دادن ندارد. این دقیقاً همان دامی است که مستنداتِ خودِ `tokio_stream` هم صریحاً هشدار می‌دهند تازه‌واردها بهش می‌افتند.

**راه‌حل:** `while let` + `.next().await`:

```rust
while let Some(n) = numbers.next().await {
    println!("{n}");
}
```

**چرا این راه‌حل است:** `while let` هر بار فقط یک `.await` می‌کند — درست همان مکانیزمی که به یک استریم اجازه می‌دهد بینِ دو عنصر واقعاً معلق شود، کاری که `for` نمی‌تواند انجامش دهد.

## تمرین

### گرم‌کردن

<details>
<summary>این کد چه چیزی چاپ می‌کند؟ (فرض کن <code>StreamExt</code> از قبل <code>use</code> شده)</summary>

```rust
let mut s = tokio_stream::iter(vec!["a", "b"]);
println!("{:?}", s.next().await);
println!("{:?}", s.next().await);
println!("{:?}", s.next().await);
```

</details>

<details>
<summary>پاسخ</summary>

```text
Some("a")
Some("b")
None
```

دو عنصر تویِ `Vec` بودند؛ سومین `.next().await` استریم را تمام‌شده می‌بیند و `None` می‌دهد — دقیقاً مثلِ آخرین `.next()` یک `Iterator`.

</details>

<details>
<summary>این کامپایل می‌شود؟</summary>

```rust
use tokio_stream::StreamExt;

let numbers = tokio_stream::iter(vec![1, 2, 3]);
for n in numbers {
    println!("{n}");
}
```

</details>

<details>
<summary>پاسخ</summary>

نه — `E0277`، «is not an iterator». حتی با `StreamExt` در اسکوپ، `for` به `IntoIterator` نیاز دارد، و `Stream` آن را پیاده نمی‌کند. باید `while let Some(n) = numbers.next().await` بنویسی.

</details>

<details>
<summary>درست یا غلط: ساختنِ <code>tokio_stream::iter(v).then(|x| async move { ... })</code>، به‌تنهایی، همان لحظه هر <code>sleep</code>ی را که داخلش هست شروع می‌کند.</summary>

</details>

<details>
<summary>پاسخ</summary>

غلط. یک استریم، درست مثلِ یک `Future`، تنبل است — تا وقتی چیزی `.next().await`اش نکند، هیچ `poll_next`ی صدا زده نمی‌شود و هیچ `sleep`ی شروع نمی‌شود.

</details>

### تعمیر

هر دو مثالِ خراب را درست کن:

۱. `examples/05-forgot-streamext.rs` را با اضافه‌کردنِ `use tokio_stream::StreamExt;` درست کن.
۲. `examples/06-for-loop-on-stream.rs` را با تبدیلِ `for n in numbers { ... }` به یک حلقه‌ی `while let Some(n) = numbers.next().await { ... }` درست کن.

### پیاده‌سازی

دو تابع در `src/lib.rs`:

- `sum_stream(values: Vec<i32>) -> i32` — `values` را با `tokio_stream::iter` به یک استریم تبدیل می‌کند و همه‌ی عنصرهایش را با یک حلقه‌ی `while let Some(...) = ... .next().await` جمع می‌زند (نه `values.iter().sum()`). یک `values` خالی جمعش `0` می‌شود.
- `first_n_even(values: Vec<i32>, limit: usize) -> Vec<i32>` — از `values` یک استریم می‌سازد، فقط عددهایِ زوج را نگه می‌دارد، فقط `limit`تایِ اولِ همان‌ها را (به همان ترتیبِ اصلی) نگه می‌دارد، و به‌شکلِ `Vec<i32>` برمی‌گرداند. اگر کمتر از `limit` عددِ زوج وجود داشته باشد، همه‌شان را برمی‌گرداند.

```sh
cargo test -p p2-09-03-streams
```

### بساز

یک `pub async fn ticks_after(delays_ms: Vec<u64>) -> Vec<u64>` بنویس: دقیقاً مثلِ الگویِ `03-time-spaced-stream` — هر عنصرِ `delays_ms` را با یک `tokio::time::sleep` واقعی به همان مقدار صبر می‌کند، یکی پس از دیگری — و در پایان همه‌ی مقدارها را، به همان ترتیب، در یک `Vec` برمی‌گرداند. بعد یک تستِ کوچک برایِ خودت بنویس که با `Instant` ثابت کند زمانِ کلِ اجرا واقعاً نزدیکِ مجموعِ `delays_ms` است — نه چیزی نزدیکِ صفر.

### چالش (اختیاری)

بدونِ هیچ آداپتوری و بدونِ `tokio_stream::iter`، یک نوعِ `CountdownStream` بساز که یک `remaining: u32` نگه می‌دارد و صفتِ `Stream` را برایش دستی پیاده می‌کند — همان روحیه‌ی `Countdown::poll` تویِ [۲.۸.۵](../../08-concurrency/05-futures-and-runtimes/README.fa.md)، این‌بار برایِ `Stream`: هر `poll_next`، اگر `remaining` صفر باشد `Poll::Ready(None)` می‌دهد؛ در غیرِ این صورت یک واحد از `remaining` کم می‌کند و مقدارِ تازه را در `Poll::Ready(Some(...))` برمی‌گرداند. (چون `CountdownStream` هیچ ارجاعِ داخلی‌ای ندارد، خودش به‌طور خودکار `Unpin` است — نیازی به `Box::pin` یا `tokio::pin!` نداری.)

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| `Stream` | نسخه‌ی async صفتِ `Iterator`؛ `poll_next` مثلِ `Future::poll` پول می‌شود، ولی `Poll<Option<T>>` می‌دهد | هر دنباله‌ی مقدارهایی که به‌مرورِ زمان می‌رسند |
| `StreamExt` | آداپتورهایِ `Stream` — `.next()`، `.map()`، `.filter()`، `.take()`، `.then()` — از `tokio_stream`، نه از prelude | `use tokio_stream::StreamExt;` بالایِ هر فایلی که با استریم کار می‌کند |
| `tokio_stream::iter` | ساده‌ترین راهِ ساختنِ یک `Stream` از رویِ یک پیمایشگرِ معمولی | مثال‌ها، تست‌ها، و هر جایی که داده‌ات از قبل تویِ حافظه است |
| `while let Some(item) = stream.next().await` | ایدیومِ اصلیِ راندنِ یک استریم — چون Rust `async for` ندارد | هر جا یک استریم را باید مصرف کنی |
| `.then()` | یک `Future` برایِ هر عنصر اجرا می‌کند؛ استریمِ نتیجه اغلب `Unpin` نیست | ساختنِ استریمی که هر مقدارش واقعاً زمان می‌برد |

### الان می‌دانی

- `Stream` دقیقاً چه نسبتی با `Iterator` و `Future` دارد، و `poll_next` را از حفظ می‌نویسی.
- چرا باید خودت `use tokio_stream::StreamExt;` کنی، و چرا `for x in stream` هیچ‌وقت کامپایل نمی‌شود.
- ایدیومِ اصلیِ مصرفِ استریم — `while let Some(item) = stream.next().await` — و اینکه آداپتورهایِ `.map()`/`.filter()`/`.take()` دقیقاً مثلِ `Iterator` تنبل و زنجیرپذیرند.
- چطور با `.then()` و یک `sleep` واقعی، استریمی بسازی که مقدارهایش واقعاً در طولِ زمان می‌رسند، و چرا قبل از `.next()`اش به `tokio::pin!` نیاز داری.
- `select!` چطور، بدونِ هیچ مکانیزمِ تازه‌ای، `.next().await` یک استریم را در برابرِ یک `Future`ِ دیگر مسابقه می‌دهد.

### بعداً کامل‌تر می‌بینی

- **صفت‌های async، و بردنِ کارِ سنگین/مسدودکننده بیرون از محیط اجرا با `spawn_blocking`** — [۲.۹.۴ — صفت‌های async و `spawn_blocking`](../04-async-traits-and-blocking/README.fa.md)، آخرین درسِ این ماژول.
- **استریم‌هایِ واقعی: ردیف‌هایِ یک پرس‌وجویِ `sqlx`، پیام‌هایِ یک وب‌سوکتِ `axum`** — [فاز ۳](../../../phase3-backend-foundations/README.fa.md)، جایی که این `while let Some(...) = ... .next().await` را رویِ داده‌ی واقعی می‌بینی، نه یک `Vec` تویِ حافظه.

### می‌توانی توضیح بدهی؟

- چرا امضایِ `Stream::poll_next` هم شکلِ `Future::poll` را دارد هم شکلِ `Iterator::next` را؟ با کلماتِ خودت بگو کجایِ امضا از کدام‌یک می‌آید.
- چرا باید خودت `StreamExt` را `use` کنی، درحالی‌که هیچ‌وقت مجبور نبودی `Iterator` را `use` کنی؟
- چرا `for x in stream` کامپایل نمی‌شود؟
- چرا `tokio::pin!` قبل از `.next()` رویِ استریمِ ساخته‌شده با `.then()` لازم است؟
- `select!` چطور می‌تواند، بدونِ هیچ مکانیزمِ تازه‌ای، `.next().await` یک استریم را در برابرِ یک `Future`ِ دیگر مسابقه بدهد؟

## بیشتر

- [مستنداتِ `futures_core::Stream`](https://docs.rs/futures-core/latest/futures_core/stream/trait.Stream.html) — همان صفتی که امروز دیدی، بدونِ هیچ آداپتوری رویش.
- [مستنداتِ `tokio_stream::StreamExt`](https://docs.rs/tokio-stream/latest/tokio_stream/trait.StreamExt.html) — کاملِ فهرستِ آداپتورها: `.merge()`، `.chain()`، `.timeout()`، `.throttle()` و بقیه.
- [فصلِ «Streams» از کتابِ رسمیِ tokio](https://tokio.rs/tokio/tutorial/streams) — همین ایدیوم، با مثالِ یک کانالِ واقعی.
- [کریتِ `async-stream`](https://docs.rs/async-stream/latest/async_stream/) — یک ماکرو که به‌ات یک نحوِ شبیهِ `yield` می‌دهد تا استریم بسازی؛ خودِ `tokio_stream` هم در مستنداتش پیشنهادش می‌کند، برایِ وقتی الگویِ `.then()`ِ امروز کافی نبود.

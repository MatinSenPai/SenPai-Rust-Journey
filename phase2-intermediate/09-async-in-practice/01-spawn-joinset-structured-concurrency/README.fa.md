# ۲.۹.۱ — `spawn`، `JoinSet`، هم‌روندیِ ساخت‌یافته

## در یک نگاه

بعد از این درس می‌توانی:

- توضیح بدهی `tokio::join!` چرا هیچ تسکِ جداگانه‌ای نمی‌سازد، و دقیقاً بگویی این با spawn‌کردنِ هرکدام روی تسکِ خودش ([۲.۸.۶](../../08-concurrency/06-tokio-basics/README.fa.md)) چه فرقی دارد.
- یک دسته کار با تعدادی که تا زمانِ اجرا معلوم نمی‌شود را با `JoinSet` جمع کنی، و نتیجه‌ها را به همان ترتیبی بخوانی که واقعاً تمام می‌شوند — نه ترتیبی که spawn‌شان کردی.
- بگویی چرا حذف‌کردنِ یک `JoinSet` هر تسکِ هنوز در حالِ اجرا را خودش لغو می‌کند، و این رفتار را با کلماتِ خودت «هم‌روندیِ ساخت‌یافته» صدا بزنی.

**زمان:** حدود ۷۰ دقیقه · **پیش‌نیاز:**
[۲.۸.۶ — مبانیِ `tokio`](../../08-concurrency/06-tokio-basics/README.fa.md)

## چرا اهمیت دارد

[۲.۸.۶](../../08-concurrency/06-tokio-basics/README.fa.md) دقیقاً دو تسک را spawn کرد — با دست، هرکدام تویِ متغیرِ خودش، و بعد هردو را به ترتیب `.await` کرد. این برایِ دو تا کار خوب جواب می‌دهد. برایِ پنج‌تا هم، شاید. ولی یک سرویسِ واقعی معمولاً نمی‌داند از قبل چندتا کار قرار است هم‌زمان انجام بدهد — یک درخواست می‌آید با فهرستی از ده آیتم، درخواستِ بعدی با سه‌تا. نمی‌شود برایِ هرکدام یک متغیرِ جدا نوشت.

این درس دو ابزارِ تازه می‌دهد، برایِ دو شکلِ متفاوت از همین مسئله. اول `tokio::join!`، برایِ وقتی که تعدادِ کار ثابت و از قبل معلوم است ولی نمی‌خواهی هزینه‌ی spawn‌کردن را بپردازی. بعد `JoinSet<T>`، برایِ دقیقاً همان حالتی که بالا توصیف شد — تعدادی که فقط سرِ اجرا معلوم می‌شود. و در همین مسیر، یک ایده‌ی اسم‌دار به‌ات می‌رسد: **هم‌روندیِ ساخت‌یافته (structured concurrency)** — اینکه عمرِ یک تسک را به یک دامنه در کدت گره بزنی، به‌جایِ اینکه رهایش کنی برود دنبالِ زندگیِ خودش.

## مفهوم

### `tokio::join!`: چند Futureِ مشخص، روی همان یک تسک

این را ببین:

```rust
async fn fetch(id: u32, delay_ms: u64) -> String {
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    format!("item-{id} (waited {delay_ms}ms)")
}

let start = Instant::now();
let (a, b) = tokio::join!(fetch(1, 150), fetch(2, 50));
println!("{a}");
println!("{b}");
println!("total: {:?}", start.elapsed());
```

اجرایِ کاملِ فایل (`examples/01-join-two-futures.rs`) این را می‌دهد:

```text
item-1 (waited 150ms)
item-2 (waited 50ms)
total: 162.1631ms
```

(عددِ آخر رویِ ماشینِ خودت کمی فرق می‌کند — همیشه کمی بالایِ ۱۵۰ میلی‌ثانیه می‌ماند، هیچ‌وقت نزدیکِ ۲۰۰.) دو `sleep` — یکی ۱۵۰ میلی‌ثانیه، یکی ۵۰ — ولی کلِ برنامه فقط حدودِ ۱۵۰ میلی‌ثانیه طول کشید، نه ۲۰۰. پس `fetch(1, ...)` و `fetch(2, ...)` واقعاً هم‌زمان پیش رفتند. ولی هیچ `tokio::spawn`ی این‌جا نیست، و هیچ دستگیره‌ای هم گرفته نشد.

`tokio::join!` دقیقاً همین را می‌کند: هر Futureِ نوشته‌شده تویِ آرگومان‌هایش را می‌گیرد و بینِشان تناوب می‌کند — یکی را تا اولین نقطه‌ی توقفش جلو می‌برد، بعدی را، و همین‌طور، تا همه تمام شوند — و فقط وقتی *همه* آماده باشند، مقدارهاشان را در یک تاپل، به همان ترتیبی که نوشته‌ای، برمی‌گرداند. مقایسه‌اش با `tokio::spawn` را که [۲.۸.۶](../../08-concurrency/06-tokio-basics/README.fa.md) بهت داد کنارِ هم بگذار:

- **هیچ تسکِ جداگانه‌ای در کار نیست.** هر دو `fetch` همچنان تویِ همان یک تسکی می‌مانند که `join!` را صدا زده. برایِ همین هیچ‌وقت رویِ دو ریسمانِ سیستم‌عاملِ جدا اجرا نمی‌شوند — `join!` تو را از تسکِ فراخواننده بیرون نمی‌برد، فقط بینِ چند Future، *داخلِ* همان تسک، تناوب می‌کند.
- **هیچ دستگیره‌ای گرفته نمی‌شود.** چون تسکِ جداگانه‌ای نساخته، چیزی هم نیست که یک `JoinHandle` برایش برگرداند یا لغوش کنی — نتیجه مستقیماً همان‌جا، سرِ خطِ `join!`، به‌دستت می‌رسد.
- **هر Future باید از قبل، با دست، نوشته شده باشد.** `fetch(1, 150)` و `fetch(2, 50)` دو آرگومانِ جداگانه‌اند، تویِ کدی که خودت نوشته‌ای. هیچ راهی نیست که یک `Vec` از Futureهایی که سرِ اجرا ساخته شده‌اند را به `join!` بدهی و بگویی «هرچقدر که تویِ این هست را منتظر بمان» — برایِ آن حالت، ابزارِ دیگری لازم است.

```senpai-visual
{"kind":"concurrency","labels":["poll fetch 1","poll fetch 2","interleave, one task","both ready together"]}
```

### `Vec<JoinHandle<T>>` تعداد را حل می‌کند، ترتیب را نه

خودِ محدودیتِ «تعداد از قبل معلوم نیست» را [۲.۸.۶](../../08-concurrency/06-tokio-basics/README.fa.md) هم داشت — تمرینِ پیاده‌سازی‌اش دقیقاً همین را می‌خواست: یک `Vec<u32>` با هر طولی بگیر، برایِ هرکدام یک تسک spawn کن. راهِ حلش هم همین بود: spawn‌کردن تویِ یک `.map(...).collect()`، و جمع‌کردنِ دستگیره‌ها تویِ یک `Vec<JoinHandle<T>>`:

```rust
let requests = vec![(1, 150), (2, 10), (3, 80)];
let handles: Vec<_> = requests
    .into_iter()
    .map(|(id, delay)| tokio::spawn(fetch(id, delay)))
    .collect();

for handle in handles {
    println!("{}", handle.await.unwrap());
}
```

اجرایِ کاملِ فایل (`examples/02-vec-joinhandle-spawn-order.rs`):

```text
item-1 (waited 150ms)
item-2 (waited 10ms)
item-3 (waited 80ms)
```

سه‌تا `sleep` با سه تأخیرِ کاملاً متفاوت — ۱۵۰، ۱۰، ۸۰ میلی‌ثانیه — ولی نتیجه‌ها دقیقاً به همان ترتیبی چاپ شدند که spawn شدند: ۱، ۲، ۳. تسکِ ۲ (فقط ۱۰ میلی‌ثانیه) خیلی زودتر از تسکِ ۱ تمام شده بود — ولی چون حلقه اول رویِ دستگیره‌ی تسکِ ۱ `.await` می‌کند، همان‌جا می‌ماند، حتی اگر تسکِ ۲ از قبل نتیجه‌اش را آماده گذاشته باشد. با یک `Vec<JoinHandle<T>>`، همیشه همین می‌شود: نتیجه‌ها به ترتیبِ *spawn* برمی‌گردند، نه به ترتیبِ *تمام‌شدن*. اگر بخواهی بدانی کدام‌یک واقعاً زودتر تمام شد — بدونِ اینکه صبر کنی نوبتش تویِ حلقه برسد — این الگو خودش راهش را نمی‌دهد.

### `JoinSet<T>`: مجموعه‌ای رشدپذیر از تسک‌ها، به ترتیبِ تمام‌شدن

`tokio::task::JoinSet<T>` دقیقاً همین کمبود را پر می‌کند. به‌جایِ اینکه هر دستگیره را خودت تویِ یک `Vec` نگه داری، به یک `JoinSet` می‌سپاریشان — و او هرکدام را، دقیقاً همان لحظه که تمام شد، به‌ات پس می‌دهد:

```rust
let requests = vec![(1, 150), (2, 10), (3, 80)];
let mut set = JoinSet::new();
for (id, delay) in requests {
    set.spawn(fetch(id, delay));
}

println!("{} tasks outstanding", set.len());
while let Some(result) = set.join_next().await {
    println!("finished: {}", result.unwrap());
}
println!("{} tasks outstanding", set.len());
```

اجرایِ کاملِ فایل (`examples/03-joinset-completion-order.rs`):

```text
3 tasks outstanding
finished: item-2 (waited 10ms)
finished: item-3 (waited 80ms)
finished: item-1 (waited 150ms)
0 tasks outstanding
```

همین سه درخواستِ قبلی — ۱۵۰، ۱۰، ۸۰ میلی‌ثانیه — ولی این‌بار نتیجه‌ها دقیقاً به ترتیبِ **تمام‌شدن** رسیدند: ۲ (۱۰ میلی‌ثانیه) اول، ۳ (۸۰ میلی‌ثانیه) دوم، ۱ (۱۵۰ میلی‌ثانیه) آخر. (این ترتیب این‌جا قابلِ‌اعتماد است چون سه تأخیر کاملاً از هم فاصله دارند؛ هیچ چیزی تویِ API خودِ `JoinSet` قول نمی‌دهد دو تسک با تأخیرِ *برابر* به یک ترتیبِ مشخص برسند.)

سه متد کارِ اصلی را انجام می‌دهند:

- **`.spawn(future)`** — دقیقاً مثلِ `tokio::spawn`، یک تسکِ تازه می‌سازد (همان قاعده‌ی `Send + 'static`ِ [۲.۸.۴](../../08-concurrency/04-send-and-sync/README.fa.md) هنوز برقرار است) — با این فرق که این تسک را خودِ `set` هم می‌شناسد و هم مالکش می‌شود. `.spawn()` واقعاً یک مقدار پس می‌دهد — یک `AbortHandle` برایِ لغوِ همان یک تسک به‌تنهایی — ولی لازم نیست نگهش داری: خودِ `set` مالکِ تسک است، و نتیجه بعداً از راهِ `.join_next()` برمی‌گردد، نه از یک دستگیره‌ای که تو نگه داشته باشی.
- **`.join_next().await`** — منتظرِ *هرکدام* از تسک‌های داخلِ `set` می‌ماند که زودتر از بقیه تمام شود، و `Option<Result<T, JoinError>>` برمی‌گرداند: `Some(نتیجه)` تا وقتی تسکی مانده، `None` وقتی `set` خالی شده. تویِ یک حلقه‌ی `while let Some(...) = ...` صدا زدنش دقیقاً یعنی «تا خالی‌شدنِ کامل، هرچی تمام شد را بگیر».
- **`.len()` / `.is_empty()`** — چندتا تسک هنوز داخلِ `set` مانده‌اند، بدونِ اینکه منتظرِ هیچ‌کدام بمانی.

```senpai-visual
{"kind":"queue","labels":["spawn 3 tasks","fastest finishes first","join_next returns it","slowest finishes last"]}
```

### یک تسک چطور شکست می‌خورد: `Result<T, JoinError>`

هر مقداری که `.join_next().await` از داخلِ آن `Option` بیرون می‌کشد، خودش یک `Result<T, JoinError>` است — نه مستقیماً `T`. `Ok(value)` یعنی تسک عادی تمام شد. `Err(join_error)` یعنی چیزی درست پیش نرفت — یا تسک پنیک کرد، یا (کوتاه اشاره می‌کنم، مفصلش را [۲.۹.۲](../02-select-and-cancellation-safety/README.fa.md) می‌دهد) از بیرون لغو/abort شد. `JoinError::is_panic()` این دو حالت را از هم جدا می‌کند.

```rust
let mut set = JoinSet::new();
set.spawn(async { 1 + 1 });
set.spawn(async {
    panic!("simulated failure inside a spawned task");
});
set.spawn(async { 2 + 2 });

while let Some(result) = set.join_next().await {
    match result {
        Ok(value) => println!("finished: ok({value})"),
        Err(err) => println!("finished: panicked = {}", err.is_panic()),
    }
}
```

اجرایِ کاملِ فایل (`examples/04-joinset-task-panics.rs`) — یک اجرا:

```text
finished: ok(2)

thread 'tokio-rt-worker' (20840) panicked at phase2-intermediate\09-async-in-practice\01-spawn-joinset-structured-concurrency\examples\04-joinset-task-panics.rs:19:9:
simulated failure inside a spawned task
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
finished: ok(4)
finished: panicked = true
```

پیامِ پنیک از داخلِ تسک، رویِ stderr، خودش را وسطِ خروجیِ سه `finished: ...` جا می‌کند — چون تسکِ پنیک‌کرده هیچ‌وقت واقعاً از بین نمی‌رود بدونِ اینکه اول این پیام چاپ شود؛ `tokio` فقط پنیک را می‌گیرد و به‌جایِ خرابکردنِ کلِ برنامه، تبدیلش می‌کند به همین `Err(JoinError)`ای که می‌بینی. (شناسه‌ی ریسمان — این‌جا `(20840)` — هر بار عوض می‌شود، درست مثلِ نمونه‌هایِ مشابه در [۲.۸.۶](../../08-concurrency/06-tokio-basics/README.fa.md).) ترتیبِ دقیقِ سه‌تا `finished: ...` را هم قول نمی‌دهم — هیچ‌کدام از این سه تسک هیچ‌وقت `.await` نمی‌کنند، پس هر سه رویِ اولین `poll`شان تمام می‌شوند، و اینکه کدام‌یک زودتر به `join_next` می‌رسد چیزی است که تنها این API تعیینش نمی‌کند؛ تنها چیزی که می‌توانی رویش حساب کنی این است که هر سه، دیر یا زود، ظاهر می‌شوند — یکی‌شان `panicked = true`.

### هم‌روندیِ ساخت‌یافته: عمرِ یک تسک، بسته به دامنه‌اش

حالا به سؤالِ اصلی می‌رسیم: وقتی خودِ `set` از بین می‌رود، برایِ تسک‌هایی که هنوز داخلش مانده‌اند چه اتفاقی می‌افتد؟

```rust
let ran = Arc::new(AtomicBool::new(false));
{
    let flag = Arc::clone(&ran);
    let mut set = JoinSet::new();
    set.spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        flag.store(true, Ordering::SeqCst);
        println!("structured task: finished sleeping"); // never prints
    });
    // `set` drops right here — well before the 100ms sleep is over.
}
tokio::time::sleep(Duration::from_millis(200)).await;
println!("structured task ran: {}", ran.load(Ordering::SeqCst));
```

اجرایِ کاملِ فایل (`examples/05-structured-drop-aborts.rs`):

```text
structured task ran: false
```

تسک ۱۰۰ میلی‌ثانیه می‌خوابد؛ برنامه بعدش ۲۰۰ میلی‌ثانیه صبر می‌کند — یعنی اگر تسک واقعاً تمام شده بود، بیشتر از وقتِ کافی برایِ چاپِ خطش داشت. ولی هیچ‌وقت چاپ نشد، و `flag` هم هیچ‌وقت `true` نشد. **حذف‌شدنِ `set`، در همان لحظه، هر تسکی را که هنوز داخلش زنده بود abort کرد** — دقیقاً همان‌جا که آن بلوکِ `{ ... }` بسته شد، نه یک لحظه دیرتر.

این را با یک `tokio::spawn` ساده مقایسه کن — بدونِ هیچ `JoinSet`ی:

```rust
let ran = Arc::new(AtomicBool::new(false));
{
    let flag = Arc::clone(&ran);
    let handle = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        flag.store(true, Ordering::SeqCst);
        println!("unstructured task: finished sleeping");
    });
    drop(handle); // dropping the HANDLE does not cancel the TASK
}
tokio::time::sleep(Duration::from_millis(200)).await;
println!("unstructured task ran: {}", ran.load(Ordering::SeqCst));
```

اجرایِ کاملِ فایل (`examples/06-unstructured-spawn-detaches.rs`):

```text
unstructured task: finished sleeping
unstructured task ran: true
```

این‌بار تسک واقعاً تمام شد — با اینکه دستگیره‌اش هم، درست مثلِ قبل، همان‌جا `drop` شد. فرق دقیقاً همین است: یک `JoinHandle` که `drop`اش می‌کنی فقط یعنی «دیگر منتظرِ این نمی‌مانم»؛ خودِ تسک را دست نمی‌زند، همچنان کاملاً مستقل و **جداشده (detached)** از هرکسی که spawnش کرده، تا هروقت خودش تمام شود یا محیط اجرا خاموش شود، به کارش ادامه می‌دهد.

این دقیقاً همان فرقی است که اسمش **هم‌روندیِ ساخت‌یافته (structured concurrency)** است: وقتی یک `JoinSet` (یا گروهی از دستگیره‌هایی که خودت با دست نگه داشته‌ای و همه‌شان را `.await` می‌کنی) از دامنه‌اش بیرون می‌رود، هر تسکی که هنوز مالِ اوست، عمرش دقیقاً به همان دامنه گره خورده — نه بیشتر. یک `tokio::spawn`ِ تنها و بی‌دستگیره، **ساخت‌نیافته (unstructured)** است: هیچ دامنه‌ای صاحبش نیست، هیچ‌کس مسئولِ خاموش‌کردنش نیست، و تا وقتی خودش تمام شود یا محیطِ اجرا خاموش شود، به کارش ادامه می‌دهد — به هیچ‌کس هم پاسخ نمی‌دهد.

```senpai-visual
{"kind":"ownership","labels":["JoinSet spawns task","scope ends","JoinSet drops","task aborted"]}
```

### کدام را انتخاب کنی؟

سه ابزار، سه شکلِ متفاوت از یک مسئله:

- **`tokio::join!`** — یک دسته‌یِ کوچک و ثابت از Futureها، معلوم از همان لحظه‌ی کامپایل، که لازم نیست رویِ چند ریسمان پخش شوند. بدونِ تسکِ جداگانه، بدونِ دستگیره.
- **`tokio::spawn` + یک `JoinHandle` دستی ([۲.۸.۶](../../08-concurrency/06-tokio-basics/README.fa.md))** — وقتی از قبل دقیقاً می‌دانی چندتا کار داری و هرکدام را می‌خواهی رویِ تسکِ خودش، احتمالاً رویِ ریسمانِ خودش.
- **`JoinSet<T>`** — وقتی تعداد سرِ اجرا معلوم می‌شود، یا وقتی مهم است هرچه زودتر بفهمی کدام‌یک اول تمام شد — نه اینکه به ترتیبِ spawn منتظر بمانی.

## دست‌به‌کد

```sh
cargo run -p p2-09-01-spawn-joinset-structured-concurrency --example 01-join-two-futures
cargo run -p p2-09-01-spawn-joinset-structured-concurrency --example 02-vec-joinhandle-spawn-order
cargo run -p p2-09-01-spawn-joinset-structured-concurrency --example 03-joinset-completion-order
cargo run -p p2-09-01-spawn-joinset-structured-concurrency --example 04-joinset-task-panics
cargo run -p p2-09-01-spawn-joinset-structured-concurrency --example 05-structured-drop-aborts
cargo run -p p2-09-01-spawn-joinset-structured-concurrency --example 06-unstructured-spawn-detaches
```

بعد دوتایِ خراب:

```sh
cargo run -p p2-09-01-spawn-joinset-structured-concurrency --example 07-forgot-await-on-join-next --features broken
cargo run -p p2-09-01-spawn-joinset-structured-concurrency --example 08-unwrap-panicked-join --features broken
```

بعد این‌ها را امتحان کن:

۱. تویِ `03-joinset-completion-order`، تأخیرِ id=2 را از ۱۰ به ۲۰۰ میلی‌ثانیه عوض کن (بزرگ‌تر از بقیه). ترتیبِ خروجی چطور عوض می‌شود؟
۲. تویِ `05-structured-drop-aborts`، `sleep(200)`ِ آخرِ `main` را بردار. برنامه دیگر تقریباً بلافاصله تمام می‌شود؟ آیا این تغییر می‌دهد که تسکِ داخلِ `JoinSet` abort شده یا نه؟ (سرنخ: به همان `println!`ی که هیچ‌وقت چاپ نمی‌شود نگاه کن — نه به اینکه `main` چقدر زنده می‌ماند.)
۳. تویِ `04-joinset-task-panics`، `set.spawn(async { 1 + 1 });` را چند بار تکرار کن. ترتیبِ خروجی‌ها را چند بار اجرا کن و ببین — همیشه یکی است؟

## خطاهایی که خواهی دید

### `E0308` — `join_next` فراموش‌شده‌ی `.await`

```text
error[E0308]: mismatched types
  --> phase2-intermediate\09-async-in-practice\01-spawn-joinset-structured-concurrency\examples\07-forgot-await-on-join-next.rs:16:15
   |
16 |     while let Some(result) = set.join_next() {
   |               ^^^^^^^^^^^^   --------------- this expression has type `impl Future<Output = Option<Result<u32, JoinError>>>`
   |               |
   |               expected future, found `Option<_>`
   |
   = note: expected opaque type `impl Future<Output = Option<Result<u32, JoinError>>>`
                     found enum `Option<_>`
help: consider `await`ing on the `Future`
   |
16 |     while let Some(result) = set.join_next().await {
   |                                             ++++++

For more information about this error, try `rustc --explain E0308`.
```

**کامپایلر به چه اعتراض دارد:** `.join_next()` خودش یک متدِ عادی نیست که مستقیماً `Option` برگرداند — یک `async fn` است، پس چیزی که برمی‌گرداند یک `Future` است. الگویِ `Some(result)` انتظار دارد رویِ یک `Option` تطبیق بدهد؛ رویِ یک `Future` که هنوز `.await` نشده، اصلاً معنی ندارد.

**راه‌حل:** همان چیزی که خودِ کامپایلر پیشنهاد داد — یک `.await` اضافه کن:

```rust
while let Some(result) = set.join_next().await {
    // ...
}
```

**چرا این راه‌حل است:** `.await` دقیقاً همان نقطه‌ای است که این Future واقعاً منتظر می‌ماند تا یکی از تسک‌ها تمام شود، و بعد `Option<Result<T, JoinError>>`ِ واقعی را بیرون می‌کشد — همان چیزی که الگویِ `Some(result)` از اول انتظارش را داشت.

### پنیک — `unwrap()` رویِ یک `JoinError`

```text
thread 'tokio-rt-worker' (10992) panicked at phase2-intermediate\09-async-in-practice\01-spawn-joinset-structured-concurrency\examples\08-unwrap-panicked-join.rs:12:9:
simulated failure inside a spawned task
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread 'main' (17536) panicked at phase2-intermediate\09-async-in-practice\01-spawn-joinset-structured-concurrency\examples\08-unwrap-panicked-join.rs:16:27:
called `Result::unwrap()` on an `Err` value: JoinError::Panic(Id(17), "simulated failure inside a spawned task", ...)
```

**کامپایلر به چه اعتراض دارد:** این هم خطایِ کامپایلر نیست — دو تا پنیکِ واقعی، سرِ اجرا. اولی از داخلِ خودِ تسک است (پیامِ ما). دومی این‌جاست: `set.join_next().await.unwrap()` یک `Result<T, JoinError>` را بدونِ نگاه‌کردن به آن `unwrap()` می‌کند؛ چون آن Result یک `Err` بود (تسک پنیک کرده بود)، همین `unwrap()` هم خودش پنیک می‌گیرد — این‌بار تویِ ریسمانِ `main`. (شناسه‌هایِ ریسمان — `(10992)` و `(17536)` — رویِ اجرایِ خودت عددهایِ دیگری خواهند بود.)

**راه‌حل:** رویِ `Result` مچ کن، به‌جایِ `unwrap()`:

```rust
match set.join_next().await.unwrap() {
    Ok(value) => println!("{value}"),
    Err(err) => println!("task failed: panicked = {}", err.is_panic()),
}
```

**چرا این راه‌حل است:** یک تسکِ spawnشده می‌تواند شکست بخورد — دقیقاً همان چیزی که «مفهوم» نشانت داد. `unwrap()`کردن یعنی فرض می‌کنی هیچ‌وقت این اتفاق نمی‌افتد؛ `match`کردن یعنی هردو حالت را واقعاً مدیریت می‌کنی.

## تمرین

### گرم‌کردن

<details>
<summary>این کد چه چیزی چاپ می‌کند، و کلِ برنامه تقریباً چقدر طول می‌کشد؟</summary>

```rust
let (a, b) = tokio::join!(
    async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        "a"
    },
    async {
        tokio::time::sleep(Duration::from_millis(30)).await;
        "b"
    }
);
println!("{a} {b}");
```

</details>

<details>
<summary>پاسخ</summary>

```text
a b
```

و کلِ برنامه چیزی نزدیکِ ۱۰۰ میلی‌ثانیه طول می‌کشد (نه ۱۳۰) — هردو Future هم‌زمان، رویِ همان یک تسک، پیش می‌روند؛ `join!` فقط وقتی که *هردو* آماده باشند برمی‌گرداند، و همیشه به همان ترتیبی که نوشته‌ای — `a` قبل از `b` — حتی با اینکه `b` زودتر تمام شده بود.

</details>

<details>
<summary>این کامپایل می‌شود؟</summary>

```rust
let mut set: JoinSet<u32> = JoinSet::new();
set.spawn(async { 42 });
let first: u32 = set.join_next().await.unwrap().unwrap();
println!("{first}");
```

</details>

<details>
<summary>پاسخ</summary>

بله — کامپایل می‌شود و بدونِ پنیک اجرا هم می‌شود، چون آن یک تسک واقعاً موفق می‌شود (`Ok(42)`)، پس هر دو `unwrap()` («از `Option` بیرون بکش»، «از `Result` بیرون بکش») رویِ یک مقدارِ سالم فرود می‌آیند.

</details>

<details>
<summary>درست یا غلط: <code>JoinSet::spawn</code> دستگیره‌ای برمی‌گرداند که خودت باید نگهش داری.</summary>

</details>

<details>
<summary>پاسخ</summary>

غلط — لازم نیست چیزی نگه داری: خودِ `set` مالکِ تسک می‌شود، و نتیجه را بعداً از راهِ `.join_next().await` می‌گیری. `.spawn()` واقعاً یک مقدار برمی‌گرداند — یک `AbortHandle` که می‌شود با آن همان یک تسک را به‌تنهایی لغو کرد — ولی چیزی مجبورت نمی‌کند نگهش داری، و مثال‌هایِ این درس هم هیچ‌وقت این کار را نمی‌کنند.

</details>

### تعمیر

هر دو مثالِ خراب را درست کن:

۱. `examples/07-forgot-await-on-join-next.rs` را طوری درست کن که کامپایل شود، بدونِ اینکه رفتارش عوض شود.
۲. `examples/08-unwrap-panicked-join.rs` را طوری درست کن که دیگر پنیک نگیرد — به‌جایِ `unwrap()` رویِ نتیجه، پیامِ `"task failed"` را چاپ کن وقتی تسک پنیک کرده، و مقدار را چاپ کن وقتی موفق شده.

### پیاده‌سازی

دو تابع در `src/lib.rs`:

- `fetch_simulated(id: u32, delay_ms: u64) -> String` — برایِ `delay_ms` میلی‌ثانیه واقعاً `.await` می‌کند، بعد دقیقاً رشته‌ی `"item-{id}"` را برمی‌گرداند — مثلاً برایِ `id = 7`، رشته‌ی `"item-7"`.
- `fetch_all_via_joinset(requests: Vec<(u32, u64)>) -> Vec<String>` — به‌ازایِ هر جفتِ `(id, delay_ms)` تویِ `requests` یک تسک رویِ یک `JoinSet` spawn می‌کند (هرکدام `fetch_simulated` را صدا می‌زند)، و همه‌ی نتیجه‌ها را به همان ترتیبی که تسک‌ها واقعاً **تمام** می‌شوند برمی‌گرداند — نه ترتیبِ `requests`.

```sh
cargo test -p p2-09-01-spawn-joinset-structured-concurrency
```

### بساز

یک `pub async fn count_task_outcomes(ids: Vec<u32>, panics_at: Vec<u32>) -> (usize, usize)` بنویس: به‌ازایِ هر id تویِ `ids` یک تسک رویِ یک `JoinSet` spawn می‌کند؛ تسکی که idاش تویِ `panics_at` هست باید پنیک کند، بقیه باید بی‌دردسر تمام شوند. با یک حلقه‌ی `join_next` بشمار چندتا `Ok` و چندتا `Err` برگشت، و `(تعدادِ موفق, تعدادِ پنیک‌کرده)` را برگردان.

### چالش (اختیاری)

یک `pub async fn first_ok_of(ids: Vec<u32>, delay_ms: u64, fail_ids: Vec<u32>) -> Option<String>` بنویس: به‌ازایِ هر id یک تسک spawn می‌کند — idهایِ تویِ `fail_ids` بلافاصله پنیک می‌کنند، بقیه `fetch_simulated(id, delay_ms)` را صدا می‌زنند. اولین نتیجه‌یِ موفق را که رسید برگردان (`Some`)، و همان لحظه، با `.abort_all()`، هر تسکِ دیگری که هنوز داخلِ `JoinSet` مانده را لغو کن — دیگر لازم نیست منتظرشان بمانی. اگر همه پنیک کردند، `None` برگردان.

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| `tokio::join!` | چند Futureِ ثابت را، هم‌زمان، رویِ همان یک تسک، منتظر می‌ماند و به همان ترتیبِ نوشته‌شده برمی‌گرداند | دسته‌یِ کوچک و از-قبل-معلومِ کارِ هم‌زمان |
| `JoinSet<T>` | مجموعه‌ای رشدپذیر از تسک‌هایِ spawnشده؛ نتیجه‌ها را به ترتیبِ تمام‌شدن پس می‌دهد | تعدادی که فقط سرِ اجرا معلوم می‌شود |
| `.join_next().await` | منتظرِ *هرکدام* از تسک‌هایِ داخلِ `JoinSet` که زودتر تمام شود می‌ماند؛ `None` وقتی خالی شده | حلقه‌یِ اصلیِ جمع‌کردنِ نتیجه‌ها |
| `Result<T, JoinError>` | نتیجه‌ی هر تسک — `Err` یا از پنیک، یا از لغوشدن | تشخیصِ شکست از موفقیت |
| هم‌روندیِ ساخت‌یافته (structured concurrency) | عمرِ یک تسک بسته به دامنه‌ای که spawnش کرده؛ حذفِ آن دامنه تسک را هم لغو می‌کند | `JoinSet` در برابرِ یک `tokio::spawn` تنها و بی‌دستگیره |

### الان می‌دانی

- چرا `tokio::join!` هیچ تسکِ جداگانه‌ای نمی‌سازد، و این با `tokio::spawn` چه فرقی دارد.
- چرا یک `Vec<JoinHandle<T>>` نتیجه‌ها را به ترتیبِ spawn پس می‌دهد، نه ترتیبِ تمام‌شدن — و `JoinSet` این را چطور حل می‌کند.
- `.spawn()`، `.join_next().await`، `.len()`، `.is_empty()` رویِ `JoinSet<T>`.
- یک تسک دقیقاً چطور به‌عنوانِ `Err(JoinError)` سرِ `.join_next()` ظاهر می‌شود، و `is_panic()` چه فرقی می‌گذارد.
- چرا حذف‌شدنِ یک `JoinSet` هر تسکِ هنوز داخلش را abort می‌کند، و یک `tokio::spawn`ِ تنها چرا این کار را نمی‌کند — و چرا اولی را «ساخت‌یافته» و دومی را «ساخت‌نیافته» صدا می‌زنیم.

### بعداً کامل‌تر می‌بینی

- **لغوِ عمدی و امنِ یک تسک، با `select!`** — [۲.۹.۲ — `select!` و ایمنیِ لغو](../02-select-and-cancellation-safety/README.fa.md). این‌جا فقط گفتیم یک `JoinError` می‌تواند از یک لغو هم بیاید، نه فقط پنیک؛ آن‌جا یاد می‌گیری چطور خودت، عمداً، یک تسک را لغو کنی.

### می‌توانی توضیح بدهی؟

- چرا `tokio::join!` هیچ‌وقت تو را رویِ بیش از یک ریسمانِ سیستم‌عامل نمی‌برد، حتی زیرِ محیط اجرایِ `multi_thread`؟
- چرا یک `Vec<JoinHandle<T>>` همیشه نتیجه‌ها را به ترتیبِ spawn برمی‌گرداند، نه ترتیبِ تمام‌شدن؟
- `.join_next().await` دقیقاً چه چیزی برمی‌گرداند، و چرا دو لایه دارد (`Option` و `Result`)؟
- اگر یک `JoinSet` را drop کنی درحالی‌که سه تسک هنوز داخلش در حالِ اجرا هستند، برایِ هرسه‌شان چه اتفاقی می‌افتد؟
- چرا یک `tokio::spawn`ِ تنها را «ساخت‌نیافته» صدا می‌زنیم؟

## بیشتر

- [مستنداتِ `JoinSet`](https://docs.rs/tokio/latest/tokio/task/struct.JoinSet.html) — امضایِ کاملِ `.spawn()`، `.join_next()`، `.abort_all()`، و چند متدِ دیگر که این درس بهشان نپرداخت.
- [مستنداتِ `tokio::join!`](https://docs.rs/tokio/latest/tokio/macro.join.html) — همان ماکرویی که [۲.۸.۶](../../08-concurrency/06-tokio-basics/README.fa.md) در «چالش»اش پیش‌نمایشش را داد.
- [مستنداتِ `JoinError`](https://docs.rs/tokio/latest/tokio/task/struct.JoinError.html) — `is_panic()`، `is_cancelled()`، و `into_panic()` برایِ گرفتنِ پیامِ اصلیِ پنیک.
- [فصلِ «Spawning» از کتابِ رسمیِ tokio](https://tokio.rs/tokio/tutorial/spawning) — همان تمایزِ تسک/ریسمانی که [۲.۸.۶](../../08-concurrency/06-tokio-basics/README.fa.md) رویش تکیه داشت، با مثال‌هایِ بیشتر.

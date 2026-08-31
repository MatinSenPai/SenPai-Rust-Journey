# ۲.۹.۲ — `select!` و ایمنیِ لغو

## در یک نگاه

بعد از این درس می‌توانی:

- توضیح بدهی `select!` دقیقاً چه‌کار می‌کند — چند `Future` را روی یک تسک با هم رقابت می‌دهد، فقط اولی که تمام شود می‌برد، و Futureِ هر شاخه‌ی دیگر همان لحظه drop می‌شود — و این را با `join!`/`JoinSet`ِ [۲.۹.۱](../01-spawn-joinset-structured-concurrency/README.fa.md) مقایسه کنی که برعکس، منتظرِ تکمیلِ همه می‌مانند.
- یک عملیاتِ واقعی را با `select!` در برابرِ یک مهلتِ زمانی بسازی — یا با دستِ خودت (یک شاخه‌ی `sleep`)، یا با میان‌برِ آماده‌ی `tokio::time::timeout` — و بگویی این دو دقیقاً یک کار می‌کنند.
- یک Futureِ ناامن‌-از-نظرِ-لغو را، از رویِ یک باگِ واقعی و تکرارپذیر، تشخیص بدهی، و با بیرون بردنِ اثرِ جانبیِ خطرناک از شاخه‌ی رقابت‌کننده درستش کنی؛ و با `CancellationToken` از یک بخشِ برنامه از تسکِ در حالِ اجرا بخواهی مشارکتی متوقف شود.

**زمان:** حدود ۷۵ دقیقه · **پیش‌نیاز:**
[۲.۹.۱ — `spawn`، `JoinSet`، هم‌روندیِ ساخت‌یافته](../01-spawn-joinset-structured-concurrency/README.fa.md)،
[۲.۸.۱ — ریسمان‌ها، `Mutex` و `Arc`](../../08-concurrency/01-threads-mutex-arc/README.fa.md)،
[۲.۸.۶ — مبانیِ `tokio`](../../08-concurrency/06-tokio-basics/README.fa.md)

## چرا اهمیت دارد

`JoinSet`ی که [۲.۹.۱](../01-spawn-joinset-structured-concurrency/README.fa.md) بهت داد یک فرض ثابت دارد: هر تسکی که بهش سپردی، در نهایت لازمش داری — پس تا همه تمام نشوند رهایشان نمی‌کند.

ولی خیلی از برنامه‌های واقعی دقیقاً برعکسِ این را می‌خواهند. یک درخواستِ HTTP که باید حداکثر ۲ ثانیه صبر کنی، نه بیشتر. یک کارگرِ پس‌زمینه که باید فوراً — نه «هر وقت کارش تمام شد» — متوقف شود، وقتی برنامه دارد خاموش می‌شود. یک کلاینت که به دو سرورِ آینه‌ای یک درخواست می‌فرستد و فقط جوابِ اولی که برسد را می‌خواهد، نه هر دو را. در هر سه‌تا، تو *نمی‌خواهی* منتظرِ همه بمانی؛ می‌خواهی هرچه زودتر تمام شد ببری، و بقیه را دور بریزی. `select!` دقیقاً همین ابزار است — و همین «دور ریختن» چیزی است که کلِ این درس دورش می‌چرخد.

## مفهوم

### `select!`: هرکدام زودتر تمام شود می‌برد، بقیه drop می‌شوند

این را اجرا کن:

```rust
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    tokio::select! {
        _ = sleep(Duration::from_millis(200)) => {
            println!("the 200ms sleep won");
        }
        _ = sleep(Duration::from_millis(50)) => {
            println!("the 50ms sleep won");
        }
    }
    println!("select! returned - the 200ms sleep's future is already gone, not just paused");
}
```

```text
the 50ms sleep won
select! returned - the 200ms sleep's future is already gone, not just paused
```

هر شاخه‌ی `select!` یک الگو، یک علامتِ `=`، یک عبارتِ async (اینجا `sleep(...)`)، یک `=>` و بعد بدنه‌ای دارد که وقتی *همان* شاخه برنده شد اجرا می‌شود. `select!` هر دو Futureِ `sleep` را می‌سازد، بعد هر دو را دوباره و دوباره poll می‌کند تا یکی‌شان `Ready` بدهد. اینجا، Futureِ ۵۰میلی‌ثانیه‌ای زودتر `Ready` می‌شود؛ بدنه‌ی همان شاخه اجرا می‌شود، و **کلِ `select!` همان لحظه برمی‌گردد** — نه اینکه صبر کند تا Futureِ ۲۰۰میلی‌ثانیه‌ای هم تمام شود. آن Futureِ ۲۰۰میلی‌ثانیه‌ای، همین‌جا، **drop** می‌شود: نه معلق، نه مکث‌شده، از بین رفته — دقیقاً همان‌طور که هر مقدارِ دیگری در Rust وقتی از دامنه بیرون می‌رود drop می‌شود.

این دقیقاً نقطه‌ی مقابلِ چیزی است که [۲.۹.۱](../01-spawn-joinset-structured-concurrency/README.fa.md) بهت داد. `join!` و `JoinSet` هر Futureای را که بهشان سپردی، **همه‌شان** را نگه می‌دارند تا تمام شوند — هیچ‌کدام را دور نمی‌ریزند. `select!` دقیقاً برعکس: فقط برنده را نگه می‌دارد، و همان لحظه‌ی برنده‌شدن، بقیه را قطعی و بی‌بازگشت drop می‌کند.

```senpai-visual
{"kind":"concurrency","labels":["دو Future ساخته می‌شوند: 200ms و 50ms","هر دو با هم poll می‌شوند","Futureِ 50ms زودتر Ready می‌شود","بدنه‌ی همان شاخه اجرا می‌شود","Futureِ 200ms همین‌جا drop می‌شود - نه معلق، از بین رفته"]}
```

### کاربردِ کلاسیک: رقابت با یک مهلتِ زمانی

رایج‌ترین دلیلِ استفاده از `select!` دقیقاً همین است: یک عملیاتِ واقعی را در برابرِ یک تایمر بگذاری تا یک مهلت به آن بدهی. فرض کن این تابع را داری — یک fetchِ شبیه‌سازی‌شده که گاهی زیادی طول می‌کشد:

```rust
async fn fetch(delay_ms: u64) -> String {
    sleep(Duration::from_millis(delay_ms)).await;
    format!("data after {delay_ms}ms")
}
```

با `select!`، یک شاخه fetch را می‌گیرد، شاخه‌ی دیگر یک `sleep` است که نقشِ مهلت را بازی می‌کند:

```rust
async fn fetch_with_budget(delay_ms: u64, budget_ms: u64) -> Option<String> {
    tokio::select! {
        data = fetch(delay_ms) => Some(data),
        _ = sleep(Duration::from_millis(budget_ms)) => None,
    }
}
```

```rust
match fetch_with_budget(20, 100).await {
    Some(data) => println!("fast fetch: got {data:?}"),
    None => println!("fast fetch: timed out"),
}
match fetch_with_budget(300, 100).await {
    Some(data) => println!("slow fetch: got {data:?}"),
    None => println!("slow fetch: timed out"),
}
```

```text
fast fetch: got "data after 20ms"
slow fetch: timed out
```

با مهلتِ ثابتِ ۱۰۰میلی‌ثانیه: fetchی که فقط ۲۰میلی‌ثانیه طول می‌کشد زودتر می‌رسد و می‌برد؛ fetchی که ۳۰۰میلی‌ثانیه طول می‌کشد هیچ‌وقت فرصت نمی‌کند — Futureاش، همان لحظه‌ای که `sleep`ِ ۱۰۰میلی‌ثانیه‌ای برنده می‌شود، drop می‌شود، وسطِ کار، بدونِ اینکه هیچ‌وقت به آن `sleep(delay_ms)` داخلِ خودش برسد.

این الگو آن‌قدر رایج است که خودِ `tokio` یک میان‌بر برایش دارد — `tokio::time::timeout`، که همین دقیقاً همین شکلِ `select!` را زیرِ پوستش دارد، فقط با یک `Result` به‌جایِ `Option`:

```rust
match tokio::time::timeout(Duration::from_millis(100), fetch(20)).await {
    Ok(data) => println!("fast fetch: got {data:?}"),
    Err(_) => println!("fast fetch: timed out"),
}
match tokio::time::timeout(Duration::from_millis(100), fetch(300)).await {
    Ok(data) => println!("slow fetch: got {data:?}"),
    Err(_) => println!("slow fetch: timed out"),
}
```

```text
fast fetch: got "data after 20ms"
slow fetch: timed out
```

همان دو نتیجه، همان دو ورودی — این‌بار فقط `Ok`/`Err` به‌جایِ `Some`/`None`. هر جا این شکلِ دقیق — «یک عملیات در برابرِ یک مهلتِ ثابت» — را داری، `tokio::time::timeout` را انتخاب کن؛ خودِ `select!` را وقتی لازم داری که بدانی این‌جا بگذار که مهلت **چیزِ دیگری** باشد، نه فقط یک `sleep` — دقیقاً همان چیزی که چند بخشِ پایین‌تر با `CancellationToken` می‌بینی.

### ایمنیِ لغو، دقیق و ملموس

حالا برگرد به همان جمله‌ی بالا: «آن Futureِ بازنده drop می‌شود.» این جمله، وقتی Futureِ بازنده فقط داشت `sleep` می‌کرد، بی‌خطر بود — یک `sleep`ِ نیمه‌کاره، وقتی از بین برود، هیچ‌چیزِ بیرونی را خراب نمی‌کند. ولی خیلی از Futureها، وسطِ راهشان، کاری بیرون از خودشان انجام می‌دهند: یک شمارنده‌ی مشترک را بالا می‌برند، یک بافر را نیمه می‌نویسند، یک قفل را می‌گیرند. اگر `select!` دقیقاً همان لحظه، وسطِ همان کار، آن Future را drop کند، آن اثرِ جانبی — چیزی که از قبل انجام شده بود — همان‌جا می‌ماند، ولی چیزی که قرار بود بعدش جبرانش کند، هرگز اجرا نمی‌شود. نه پنیک، نه خطا، هیچ‌چیزی برایِ گرفتن نیست — فقط یک وضعیتِ بیرونی که برای همیشه اشتباه ماند.

یک Future را **ایمن از نظرِ لغو (cancellation-safe)** می‌نامیم اگر drop‌شدنش وسطِ poll، هیچ‌وقت هیچ وضعیتِ بیرونی — یک شمارنده‌ی مشترک، یک بافرِ نیمه‌نوشته، یک قفل — را در حالتِ غلط رها نکند. بیا این را با یک باگِ واقعی، نه فرضی، ببینیم. این تابع یک شمارنده‌ی «تعدادِ درخواست‌هایِ در حالِ اجرا» را نگه می‌دارد — یک الگویِ کاملاً واقعی، شبیهِ چیزی که یک متریک یا یک gauge برایِ نظارت می‌سازد:

```rust
async fn tracked_fetch(in_flight: Arc<Mutex<u32>>, delay_ms: u64) -> String {
    *in_flight.lock().unwrap() += 1; // "یک fetch همین الان شروع شد"
    sleep(Duration::from_millis(delay_ms)).await; // کارِ واقعی
    *in_flight.lock().unwrap() -= 1; // "تمام شد" - اگر زودتر drop شود هرگز اجرا نمی‌شود
    "done".to_string()
}
```

حالا این را پنج بار، با یک مهلتِ همیشه-برنده، در یک حلقه صدا بزن:

```rust
let in_flight = Arc::new(Mutex::new(0u32));
let attempts = 5;
for _ in 0..attempts {
    tokio::select! {
        _ = tracked_fetch(in_flight.clone(), 200) => {}
        _ = sleep(Duration::from_millis(20)) => {}
    }
}
println!("attempts made: {attempts}");
println!("in_flight counter: {}", *in_flight.lock().unwrap());
```

```text
attempts made: 5
in_flight counter: 5
```

مهلت (۲۰ میلی‌ثانیه) همیشه از fetch (۲۰۰ میلی‌ثانیه) جلو می‌زند، پس هر بار `tracked_fetch` وسطِ همان `sleep(delay_ms)`ِ داخلی‌اش drop می‌شود — بعد از خطِ `+= 1`، ولی همیشه پیش از رسیدن به خطِ `-= 1`. پنج بار این اتفاق افتاد؛ پنج بار شمارنده بالا رفت؛ هیچ‌وقت پایین نیامد. هیچ پنیکی نگرفتیم، هیچ خطایی هم نبود — فقط یک عددِ همیشگی اشتباه، که هیچ‌کس بدونِ خواندنِ دقیقِ این کد متوجهش نمی‌شود.

```senpai-visual
{"kind":"concept","labels":["+= 1 اجرا می‌شود: یک درخواست در حال اجراست","sleep شروع می‌شود","مهلت زودتر می‌رسد","select! برمی‌گردد؛ tracked_fetch drop می‌شود","-= 1 هرگز اجرا نمی‌شود؛ شمارنده برای همیشه بالاست"]}
```

### راه‌حل: اثرِ جانبیِ خطرناک را از شاخه‌ی رقابت‌کننده بیرون ببر

راه‌حل نیازی به هیچ ابزارِ تازه‌ای ندارد — فقط باید همان `+= 1`/`-= 1` را از داخلِ Futureای که ممکن است ببازد بیرون بیاوری، و بگذاری داخلِ چیزی که خودش هرگز رقابت نمی‌کند:

```rust
async fn run_one(in_flight: Arc<Mutex<u32>>, delay_ms: u64, budget_ms: u64) -> Option<String> {
    *in_flight.lock().unwrap() += 1; // بیرونِ Futureِ رقابت‌کننده: همیشه اجرا می‌شود
    let result = tokio::select! {
        data = fetch(delay_ms) => Some(data),
        _ = sleep(Duration::from_millis(budget_ms)) => None,
    };
    *in_flight.lock().unwrap() -= 1; // اینجا select! از قبل برگشته
    result
}
```

همان حلقه‌ی پنج‌باره‌ی بالا، این‌بار با `run_one` به‌جایِ `tracked_fetch`:

```rust
let in_flight = Arc::new(Mutex::new(0u32));
let attempts = 5;
for _ in 0..attempts {
    run_one(in_flight.clone(), 200, 20).await;
}
println!("attempts made: {attempts}");
println!("in_flight counter: {}", *in_flight.lock().unwrap());
```

```text
attempts made: 5
in_flight counter: 0
```

نکته‌ی ساختاری این است: خودِ `run_one` هرگز با چیزِ دیگری به رقابت گذاشته نمی‌شود — فراخواننده فقط `.await`ش می‌کند، عادی، تا آخر. پس `+= 1` و `-= 1` همیشه جفت‌جفت اجرا می‌شوند، هر بار. `select!`ِ داخلش هنوز دقیقاً همان کارِ قبلی را می‌کند — هنوز Futureِ بازنده را drop می‌کند — ولی حالا هیچ حسابداریِ خطرناکی داخلِ آن Futureِ drop‌شدنی نمانده؛ فقط خودِ `fetch` آنجاست، و از بین رفتنِ یک `fetch`ِ نیمه‌کاره کاملاً بی‌خطر است. قاعده‌ی کلی همین است: **کاری که با drop‌شدن نباید نیمه‌کاره بماند را بیرونِ شاخه‌ی `select!` بگذار، یا کاری کن که اثرش فقط بعد از مشخص‌شدنِ برنده اتفاق بیفتد.**

### `CancellationToken`: از یک تسک بخواه مشارکتی متوقف شود

مثالِ بالا یک مهلتِ *زمانی* بود. گاهی مهلتی که می‌خواهی زمان نیست — یک سیگنالِ بیرونی است: کاربر روی «لغو» زد، برنامه دارد خاموش می‌شود، یک درخواستِ دیگر دیگر لازمش نیست. `CancellationToken`، از کریتِ **جداگانه‌یِ** `tokio-util` (نه خودِ `tokio` — باید جدا importش کنی)، دقیقاً همین را می‌دهد: یک دستگیره‌ی قابلِ‌کلون و ارزان‌برایِ‌اشتراک. هر کلونی از یک `CancellationToken` به همان یک لغوِ زیرین اشاره می‌کند؛ `.cancel()` را روی هرکدام صدا بزنی، همه خبردار می‌شوند. خودِ `.cancelled()` یک Future است — Futureای که تا وقتی کسی `.cancel()` را صدا نزده `Pending` می‌ماند، و همان لحظه‌ی `.cancel()` شدن `Ready` می‌شود — پس دقیقاً مثلِ هر Futureِ دیگری می‌شود گذاشتش تویِ یک شاخه‌ی `select!`.

```rust
async fn worker(token: CancellationToken) -> u32 {
    let mut ticks = 0;
    loop {
        tokio::select! {
            _ = token.cancelled() => break,
            _ = sleep(Duration::from_millis(40)) => {
                ticks += 1;
                println!("worker: tick {ticks}");
            }
        }
    }
    ticks
}
```

یک کنترل‌کننده که بعدِ مدتی می‌خواهد کارگر را متوقف کند:

```rust
let token = CancellationToken::new();
let handle = tokio::spawn(worker(token.clone()));
sleep(Duration::from_millis(140)).await;
println!("controller: asking the worker to stop");
token.cancel();
println!("worker completed {} ticks before stopping", handle.await.unwrap());
```

```text
worker: tick 1
worker: tick 2
worker: tick 3
controller: asking the worker to stop
worker: cancelled, stopping
worker completed 3 ticks before stopping
```

(ترتیبِ دقیقِ چاپِ خطِ «controller» نسبت به «tick 3» بینِ اجراهای مختلف فرق می‌کند — چون کارگر و کنترل‌کننده دو تسکِ کاملاً مستقل‌اند؛ ولی تعدادِ نهاییِ tickها، با این تایمینگ، همیشه ۳ می‌ماند.) کارگر هر ۴۰میلی‌ثانیه یک tick می‌زند؛ کنترل‌کننده بعدِ ۱۴۰میلی‌ثانیه `cancel()` را صدا می‌زند — دقیقاً بینِ سومین tick (در ۱۲۰میلی‌ثانیه) و چهارمین (در ۱۶۰میلی‌ثانیه). کارگر همین که `.cancelled()` را poll می‌کند و می‌بیند حالا `Ready` است، از حلقه بیرون می‌زند — نه فوری در همان لحظه‌ی صدا زدنِ `.cancel()` از بیرون، بلکه دقیقاً وقتی نوبتِ همان شاخه از `select!` می‌رسد که poll شود. این دقیقاً همان معنیِ «مشارکتی» است که [۲.۸.۵](../../08-concurrency/05-futures-and-runtimes/README.fa.md) درباره‌ی زمان‌بندیِ تسک‌ها بهت داد: هیچ‌کس کارگر را وسطِ کارش قطع نمی‌کند؛ خودِ کارگر است که، در نقطه‌ای که خودش مشخص کرده، می‌پرسد «باید متوقف شوم؟»

```senpai-visual
{"kind":"concurrency","labels":["controller: token.cancel() صدا زده می‌شود","token.cancelled() در تسک worker Ready می‌شود","select! داخل worker شاخه‌ی cancelled را برنده می‌کند","worker از حلقه بیرون می‌زند"]}
```

### `biased;`: خاموش‌کردنِ تصادفی‌بودن

وقتی چند شاخه‌ی `select!` هم‌زمان آماده باشند — مثلاً هر دو همین حالا `Ready`اند، نه اینکه یکی زودتر برسد — `tokio` به‌طورِ پیش‌فرض یکی را **به‌طورِ تصادفی** از بینِ آماده‌ها انتخاب می‌کند، نه لزوماً همیشه اولی که نوشتی. کلمه‌ی کلیدیِ `biased;`، به‌عنوانِ اولین خط داخلِ بلوکِ `select!`، همین تصادفی‌بودن را خاموش می‌کند: شاخه‌ها دقیقاً به همان ترتیبی که نوشتی‌شان poll می‌شوند، و اولینِ آماده — نه یکیِ تصادفی از بینِ آماده‌ها — برنده است.

```rust
tokio::select! {
    biased;
    _ = std::future::ready(()) => println!("biased: branch A won"),
    _ = std::future::ready(()) => println!("biased: branch B won"),
}
```

```text
biased: branch A won
```

این خروجی، با `biased;`، هر بار که اجرا کنی همین می‌ماند — چون شاخه‌ی اول همیشه اول poll می‌شود. بدونِ `biased;`، همین دو خط را چند بار اجرا کنی، گاهی A می‌برد و گاهی B (خودِ کد در `examples/07-biased-select.rs` هر دو حالت را کنارِ هم نشانت می‌دهد). بیشترِ وقت‌ها به این نیازی نداری — تصادفی‌بودنِ پیش‌فرض دقیقاً برای این است که یک شاخه، وقتی همیشه اول نوشته شده، بقیه را گرسنه نگه ندارد؛ فقط وقتی یک شاخه واقعاً باید اولویت داشته باشد (مثلاً یک شاخه‌ی لغو، که همیشه باید اول چک شود) سراغِ `biased;` برو.

## دست‌به‌کد

```sh
cargo run -p p2-09-02-select-and-cancellation-safety --example 01-race-two-sleeps
cargo run -p p2-09-02-select-and-cancellation-safety --example 02-fetch-with-timeout
cargo run -p p2-09-02-select-and-cancellation-safety --example 03-timeout-shorthand
cargo run -p p2-09-02-select-and-cancellation-safety --example 04-not-cancellation-safe
cargo run -p p2-09-02-select-and-cancellation-safety --example 05-cancellation-safe
cargo run -p p2-09-02-select-and-cancellation-safety --example 06-cancellation-token
cargo run -p p2-09-02-select-and-cancellation-safety --example 07-biased-select
```

بعد سه‌تای خراب:

```sh
cargo run -p p2-09-02-select-and-cancellation-safety --example 08-moved-value-broken --features broken
cargo run -p p2-09-02-select-and-cancellation-safety --example 09-double-borrow-broken --features broken
cargo run -p p2-09-02-select-and-cancellation-safety --example 10-mismatched-types-broken --features broken
```

بعد این‌ها را امتحان کن:

۱. در `04-not-cancellation-safe`، عددِ `attempts` را از ۵ به ۵۰ عوض کن. شمارنده‌ی نهایی به چه عددی می‌رسد؟ چرا دقیقاً همین رابطه؟
۲. در `06-cancellation-token`، مدتِ `sleep` قبل از `token.cancel()` را از ۱۴۰ به ۱۰ میلی‌ثانیه کم کن (کمتر از یک tick). چند tick چاپ می‌شود؟
۳. `07-biased-select` را ده بار پشتِ سرِ هم اجرا کن (یا با یک حلقه‌ی shell). شاخه‌ی «unbiased» چند بار A را برد و چند بار B را؟ شاخه‌ی «biased» چطور؟

## خطاهایی که خواهی دید

### `E0382` — استفاده از مقداری که قبلاً منتقل شده

```text
error[E0382]: use of moved value: `payload`
  --> phase2-intermediate\09-async-in-practice\02-select-and-cancellation-safety\examples\08-moved-value-broken.rs:18:25
   |
15 |     let payload = String::from("hello");
   |         ------- move occurs because `payload` has type `String`, which does not implement the `Copy` trait
16 |     tokio::select! {
17 |         _ = send(payload) => {}
   |                  ------- value moved here
18 |         _ = log_dropped(payload) => {}
   |                         ^^^^^^^ value used here after move
   |
note: consider changing this parameter type in function `send` to borrow instead if owning the value isn't necessary
  --> phase2-intermediate\09-async-in-practice\02-select-and-cancellation-safety\examples\08-moved-value-broken.rs:5:24
   |
 5 | async fn send(payload: String) {
   |          ----          ^^^^^^ this parameter takes ownership of the value
   |          |
   |          in this function
help: consider cloning the value if the performance cost is acceptable
   |
17 |         _ = send(payload.clone()) => {}
   |                         ++++++++

For more information about this error, try `rustc --explain E0382`.
```

**کامپایلر به چه اعتراض دارد:** `select!` هر شاخه را فقط وقتی برنده شد اجرا می‌کند، ولی عبارتِ *هر* شاخه‌ای — یعنی خودِ `send(payload)` و خودِ `log_dropped(payload)` — پیش از هرگونه poll‌شدنی، همان لحظه‌ی رسیدن به `select!`، ساخته می‌شود. یعنی `payload` باید هم‌زمان به هر دو تابع منتقل شود، ولی `String` یک مقدارِ `Copy` نیست — انتقالش به `send` یعنی دیگر چیزی برایِ منتقل‌کردن به `log_dropped` نمانده، حتی اگرچه در عمل فقط یکی از این دو Future واقعاً اجرا خواهد شد.

**راه‌حل:** یا هر شاخه یک کلونِ خودش را بگیرد:

```rust
tokio::select! {
    _ = send(payload.clone()) => {}
    _ = log_dropped(payload) => {}
}
```

یا — اگر تابع‌ها اصلاً نیازی به مالکیت ندارند — امضایشان را به `&str` تغییر بده.

**چرا این راه‌حل است:** کامپایلر نمی‌داند کدام شاخه برنده می‌شود — این چیزی است که فقط سرِ اجرا مشخص می‌شود — پس باید طوری کد را قبول کند که *هرکدام* برنده شود درست از آب دربیاید. کلون‌کردن دقیقاً همین را تضمین می‌کند: هر شاخه نسخه‌ی خودش را دارد، مهم نیست کدام واقعاً اجرا شود.

### `E0499` — نمی‌شود یک متغیر را بیش از یک‌بار هم‌زمان به‌طورِ تغییرپذیر قرض گرفت

```text
error[E0499]: cannot borrow `total` as mutable more than once at a time
  --> phase2-intermediate\09-async-in-practice\02-select-and-cancellation-safety\examples\09-double-borrow-broken.rs:23:21
   |
21 | /     tokio::select! {
22 | |         _ = add_one(&mut total) => {}
   | |                     ---------- first mutable borrow occurs here
23 | |         _ = add_two(&mut total) => {}
   | |                     ^^^^^^^^^^ second mutable borrow occurs here
24 | |     }
   | |_____- first borrow later used here

For more information about this error, try `rustc --explain E0499`.
```

**کامپایلر به چه اعتراض دارد:** همان دلیلِ خطایِ قبلی، این‌بار برایِ قرض‌ها به‌جایِ مالکیت. `select!` باید هر دو Futureِ `add_one(&mut total)` و `add_two(&mut total)` را از قبل، هر دو با هم، بسازد تا بتواند هر دو را poll کند — پس هر دو ارجاعِ `&mut total` باید هم‌زمان زنده باشند، درست همان چیزی که قاعده‌ی هم‌نامی ([۱.۳.۱](../../../phase1-fundamentals/03-borrowing-and-references/01-shared-and-mutable-refs/README.fa.md)) اجازه نمی‌دهد.

**راه‌حل:** `total` را پشتِ یک `Mutex` بگذار (دقیقاً همان چیزی که در «مفهوم» برایِ `in_flight` دیدی)، یا — اگر معنایش اجازه می‌دهد — هر شاخه شمارنده‌ی خودش را داشته باشد و بعد از `select!` جمعشان بزن.

**چرا این راه‌حل است:** خودِ رقابت — یعنی دو Futureِ زنده‌یِ هم‌زمان تا وقتی یکی برنده شود — دقیقاً همان چیزی است که باعث می‌شود دو قرض هم‌زمان لازم باشد؛ این ساختاری است، نه اتفاقی. یک `Mutex` قاعده‌ی هم‌نامی را از کامپایل به اجرا می‌برد ([۲.۸.۱](../../08-concurrency/01-threads-mutex-arc/README.fa.md))، دقیقاً جایی که این ساختار نیازش دارد.

### `E0308` — شاخه‌های `match` نوع‌های ناسازگار دارند

```text
error[E0308]: `match` arms have incompatible types
  --> phase2-intermediate\09-async-in-practice\02-select-and-cancellation-safety\examples\10-mismatched-types-broken.rs:17:49
   |
15 |       let result = tokio::select! {
   |  __________________-
16 | |         data = fetch() => data,
   | |                           ---- this is found to be of type `&str`
17 | |         _ = sleep(Duration::from_millis(50)) => 404,
   | |                                                 ^^^ expected `&str`, found integer
18 | |     };
   | |_____- `match` arms have incompatible types

For more information about this error, try `rustc --explain E0308`.
```

**کامپایلر به چه اعتراض دارد:** پیامِ خطا خودش لو می‌دهد — `select!` زیرِ پوستش به یک `match` باز می‌شود، و درست مثلِ هر `match`ِ دیگری ([۱.۵.۴](../../../phase1-fundamentals/05-your-own-types/04-match-in-depth/README.fa.md))، وقتی نتیجه‌اش را در یک متغیر می‌ریزی، همه‌ی شاخه‌ها باید دقیقاً یک نوع برگردانند. اینجا یک شاخه `&str` می‌دهد، دیگری یک عددِ صحیح — کامپایلر نمی‌داند `result` باید کدام نوع باشد.

**راه‌حل:** هر دو شاخه را به یک نوعِ مشترک برسان — مثلاً هر دو `String`، یا هر دو بپیچشان در یک `Option`:

```rust
let result = tokio::select! {
    data = fetch() => Some(data.to_string()),
    _ = sleep(Duration::from_millis(50)) => None,
};
```

**چرا این راه‌حل است:** دقیقاً همان الگویِ `fetch_with_budget` بالاتر — بستنِ هر نتیجه در یک `Option` (یا هر نوعِ مشترکِ دیگر) دقیقاً همان چیزی است که به دو شاخه با معنایِ متفاوت («یافتم» / «مهلت تمام شد») اجازه می‌دهد یک نوعِ واحد داشته باشند.

## تمرین

### گرم‌کردن

<details>
<summary>این کد چه چیزی چاپ می‌کند؟</summary>

```rust
#[tokio::main]
async fn main() {
    tokio::select! {
        _ = tokio::time::sleep(Duration::from_millis(10)) => {
            println!("A");
        }
        _ = tokio::time::sleep(Duration::from_millis(300)) => {
            println!("B");
        }
    }
    println!("done");
}
```

</details>

<details>
<summary>پاسخ</summary>

```text
A
done
```

شاخه‌ی ۱۰میلی‌ثانیه‌ای زودتر `Ready` می‌شود، بدنه‌اش چاپ می‌کند، `select!` همان لحظه برمی‌گردد؛ شاخه‌ی ۳۰۰میلی‌ثانیه‌ای هیچ‌وقت `println!("B")`اش را اجرا نمی‌کند چون Futureاش قبل از رسیدن به آن مقدار drop می‌شود.

</details>

<details>
<summary>این کامپایل می‌شود؟</summary>

```rust
async fn a(s: String) {}
async fn b(s: String) {}

async fn run() {
    let s = String::from("x");
    tokio::select! {
        _ = a(s) => {}
        _ = b(s) => {}
    }
}
```

</details>

<details>
<summary>پاسخ</summary>

نه — `E0382`. عبارتِ هر دو شاخه، `a(s)` و `b(s)`، پیش از هر poll‌شدنی ساخته می‌شود، پس `s` باید هم‌زمان به هر دو منتقل شود؛ `String` هم `Copy` نیست. دقیقاً همان چیزی که در «خطاهایی که خواهی دید» دیدی.

</details>

<details>
<summary>درست یا غلط: <code>select!</code>، مثلِ <code>join!</code>، منتظر می‌ماند تا همه‌ی شاخه‌هایش تمام شوند.</summary>

</details>

<details>
<summary>پاسخ</summary>

غلط. `join!` منتظرِ همه می‌ماند؛ `select!` دقیقاً برعکس — فقط منتظرِ *اولی* می‌ماند و بقیه را همان لحظه drop می‌کند. این دقیقاً همان تفاوتی است که کلِ این درس دورش می‌چرخد.

</details>

### تعمیر

هر سه مثالِ خراب را درست کن:

۱. `examples/08-moved-value-broken.rs` را با کلون‌کردنِ مقدار برایِ یکی از دو شاخه درست کن.
۲. `examples/09-double-borrow-broken.rs` را با گذاشتنِ `total` پشتِ یک `Arc<Mutex<u32>>` درست کن — هر شاخه با `.clone()` خودِ `Arc` را بگیرد.
۳. `examples/10-mismatched-types-broken.rs` را با بستنِ هر دو شاخه در یک `Option<String>` مشترک درست کن.

### پیاده‌سازی

دو تابع در `src/lib.rs`:

- `fetch_with_deadline(delay_ms: u64, budget_ms: u64) -> Option<String>` — یک fetchِ شبیه‌سازی‌شده که `delay_ms` میلی‌ثانیه صبر می‌کند و بعد دقیقاً رشته‌ی `format!("data after {delay_ms}ms")` را می‌دهد، در برابرِ یک مهلتِ `budget_ms` میلی‌ثانیه‌ای. اگر fetch زودتر تمام شود، `Some` همان رشته را برگردان؛ اگر مهلت زودتر برسد، `None`.
- `run_until_cancelled(token: CancellationToken, tick_ms: u64) -> u32` — حلقه‌ای که هر `tick_ms` میلی‌ثانیه یک «tick» می‌زند، تا وقتی `token` لغو شود. هر دور، یک tick (یک `sleep`ِ `tick_ms`میلی‌ثانیه‌ای) را در برابرِ `token.cancelled()` رقابت می‌دهد. یک tick فقط وقتی شمرده می‌شود که `sleep`اش پیش از دیده‌شدنِ لغو تمام شده باشد؛ tickی که هنوز در حالِ `sleep` بود وقتی توکن لغو شد، شمرده نمی‌شود. وقتی حلقه متوقف شد، تعدادِ کلِ tickهایِ کامل‌شده را برگردان.

```sh
cargo test -p p2-09-02-select-and-cancellation-safety
```

### بساز

یک `pub async fn fetch_with_retries(token: CancellationToken, delay_ms: u64, budget_ms: u64, max_attempts: u32) -> Option<String>` بنویس: تا `max_attempts` بار، `fetch_with_deadline` را با همان `delay_ms`/`budget_ms` تکرار می‌کند؛ همین که یک تلاش موفق شد (Some برگرداند)، همان را برگردان؛ اگر در هر لحظه — بینِ تلاش‌ها یا وسطِ یکی‌شان — `token` لغو شد، فوراً `None` برگردان؛ اگر همه‌ی `max_attempts` تلاش با مهلت تمام شدند، `None` برگردان. بعد چند تستِ کوچک برایِ خودت بنویس: یک‌بار برایِ موفقیتِ تلاشِ اول، یک‌بار برایِ باختنِ همه‌ی تلاش‌ها، یک‌بار برایِ یک توکنِ از قبل لغوشده.

### چالش (اختیاری)

با `JoinSet`ِ [۲.۹.۱](../01-spawn-joinset-structured-concurrency/README.fa.md)، یک `pub async fn fetch_first_of(delays: Vec<u64>, budget_ms: u64) -> Option<String>` بنویس: هر مقدار در `delays` را با `tokio::spawn` جداگانه spawn کن (یک `fetch` به‌ازایِ هرکدام)، بعد با `select!`، «اولین تسکی که تمام شد» را در برابرِ یک `sleep(budget_ms)`ِ مشترک به رقابت بگذار. اگر هر تسکی زودتر از مهلت تمام شود، نتیجه‌اش را برگردان؛ اگر مهلت از همه‌شان جلو بزند، `None`. (سرنخ: `JoinSet::join_next()` خودش یک متدِ async است — می‌شود مستقیم به‌عنوانِ یک عبارتِ شاخه در `select!` صدایش زد.)

## جمع‌بندی

`select!` و `JoinSet`/`spawn`ِ [۲.۹.۱](../01-spawn-joinset-structured-concurrency/README.fa.md) دو ابزارِ کاملاً متضادند، نه دو نسخه از یک ایده: یکی هرچه بهش سپردی نگه می‌دارد تا همه تمام شوند؛ آن یکی، همین که یکی‌شان برنده شد، بقیه را عمداً دور می‌ریزد. این «دور ریختن» رایگان نیست — قیمتش دقیقاً همان چیزی است که این درس «ایمنیِ لغو» نامیدش: هر Futureای که کارِ نیمه‌تمامش را بیرون از خودش جا می‌گذارد، اگر شانسی داشته باشد که ببازد، باید طوری نوشته شود که آن کار هرگز نیمه‌کاره نماند. این یکی از تیزترین لبه‌هایِ Rustِ ناهمگام است — نه چون پیچیده است، بلکه چون سکوت می‌کند: نه پنیک می‌گیری، نه خطا می‌بینی، فقط یک عددِ اشتباه که یک روز، در یک لاگ، پیدایش می‌کنی.

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| `select!` | چند Future را روی یک تسک رقابت می‌دهد؛ برنده اجرا می‌شود، بقیه همان لحظه drop می‌شوند | راندازی با مهلت، لغو، «هرکدام زودتر» |
| `tokio::time::timeout` | میان‌برِ آماده برایِ دقیقاً یک شکل از `select!`: یک عملیات در برابرِ یک `sleep` | هر جا مهلت فقط زمان است، نه سیگنالِ دیگری |
| ایمنیِ لغو (cancellation safety) | drop‌شدنِ یک Futureِ بازنده وسطِ کار، وضعیتِ بیرونی‌اش را در حالتِ غلط رها نکند | هر Futureای که پیش از تکمیل شمارنده/بافر/قفلِ بیرونی را لمس می‌کند |
| `CancellationToken` | دستگیره‌ی قابلِ‌کلونِ `tokio-util` برایِ خاموشیِ مشارکتی؛ `.cancelled()` یک Future است | یک بخشِ برنامه از تسکِ دیگری می‌خواهد متوقف شود |
| `biased;` | شاخه‌ها را به ترتیبِ نوشته‌شده poll می‌کند، نه تصادفی از بینِ آماده‌ها | وقتی یک شاخه (مثلِ لغو) واقعاً باید همیشه اول چک شود |

### الان می‌دانی

- `select!` چند Future را روی یک تسک رقابت می‌دهد و فقط برنده را نگه می‌دارد — بقیه همان لحظه drop می‌شوند؛ این دقیقاً برعکسِ `join!`/`JoinSet` است.
- چطور یک عملیاتِ واقعی را در برابرِ یک مهلت بسازی، دستی با یک شاخه‌ی `sleep` یا با میان‌برِ `tokio::time::timeout`.
- ایمنیِ لغو یعنی چه، و چرا drop‌شدنِ یک Futureِ بازنده می‌تواند یک وضعیتِ بیرونی را — بدونِ هیچ پنیک یا خطایی — برای همیشه خراب کند.
- راه‌حل: اثرِ جانبیِ خطرناک را بیرونِ شاخه‌ی رقابت‌کننده بگذار، تا هرگز نیمه‌کاره drop نشود.
- `CancellationToken` چیست، از کدام کریت می‌آید، و چطور `.cancelled()` را به‌عنوانِ یک شاخه‌ی `select!` استفاده کنی.
- `biased;` چه‌کار می‌کند و کِی واقعاً لازمش داری.

### بعداً کامل‌تر می‌بینی

- **دنباله‌های async — مقادیری که یکی‌یکی، در طولِ زمان، می‌رسند** — [۲.۹.۳ — استریم‌ها](../03-streams/README.fa.md)
- **صفت‌های async، و بردنِ کارِ سنگین/مسدودکننده بیرون از محیط اجرا با `spawn_blocking`** — [۲.۹.۴](../04-async-traits-and-blocking/README.fa.md)

### می‌توانی توضیح بدهی؟

- `select!` با `join!`/`JoinSet` دقیقاً چه فرقی دارد؟ یک جمله بگو.
- «ایمنیِ لغو» را با کلماتِ خودت تعریف کن — بدونِ استفاده از کلماتِ خودِ این درس.
- در مثالِ باگ‌دار، دقیقاً چرا شمارنده همیشه بالا می‌رود ولی هیچ‌وقت پایین نمی‌آید؟
- راه‌حل چرا واقعاً کار می‌کند؟ کدام خط، دقیقاً، دیگر هرگز drop نمی‌شود؟
- `CancellationToken.cancelled()` را چطور با یک `sleep` مقایسه می‌کنی؟ هر دو Futureاند — فرقشان کجاست؟

## بیشتر

- [مستنداتِ `tokio::select!`](https://docs.rs/tokio/latest/tokio/macro.select.html) — همه‌ی جزئیات، شاملِ `biased;`، شاخه‌هایِ `if` و رفتارِ دقیقِ الگوهایِ ردشدنی.
- [مستنداتِ `tokio::time::timeout`](https://docs.rs/tokio/latest/tokio/time/fn.timeout.html) — امضایِ دقیق و مثال‌هایِ بیشتر.
- [مستنداتِ `tokio_util::sync::CancellationToken`](https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html) — شاملِ `child_token()` برایِ درختِ لغوهایِ تودرتو، که این درس بهش نپرداخت.
- [فصلِ «Select» از کتابِ رسمیِ tokio](https://tokio.rs/tokio/tutorial/select) — همین ایمنیِ لغو، با مثال‌هایِ بیشتر.

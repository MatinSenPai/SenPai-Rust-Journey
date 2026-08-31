# ۲.۹.۴ — صفت‌های async و `spawn_blocking`

## در یک نگاه

بعد از این درس می‌توانی:

- توضیح بدهی چرا می‌توانی `async fn` را مستقیم داخلِ یک صفت بنویسی — بدونِ هیچ ماکرویی — و آن را از پشتِ یک کراندِ جنریک صدا بزنی.
- خطایِ dyn-سازگار نبودنِ یک صفتِ async را بخوانی، با `#[async_trait]` رفعش کنی، و بگویی این رفع دقیقاً چه هزینه‌ای رویِ دوشت می‌گذارد.
- یک کارِ واقعاً سنگینِ پردازنده یا یک تماسِ مسدودکننده را با `spawn_blocking` از محیطِ اجرا بیرون ببری — و بگویی چرا این کار را فقط برایِ همین دو مورد انجام می‌دهی، نه برایِ هر تابعِ همگامِ سریع.

**زمان:** حدود ۷۰ دقیقه · **پیش‌نیاز:**
[۲.۹.۳ — استریم‌ها](../03-streams/README.fa.md)،
[۲.۳.۷ — ارسالِ ایستا در برابرِ پویا، و ایمنیِ شیء](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.fa.md)

---

## چرا اهمیت دارد

تا اینجایِ همین ماژول، هر سه درسِ قبل دورِ یک محورِ مشترک چرخیدند: شکل‌دادن به کاری که از قبل async است. [۲.۹.۱](../01-spawn-joinset-structured-concurrency/README.fa.md) نشانت داد چطور چند تسک را هم‌زمان spawn کنی و منتظرشان بمانی؛ [۲.۹.۲](../02-select-and-cancellation-safety/README.fa.md) نشانت داد چطور رویِ چند `Future` هم‌زمان صبر کنی و یکی را به‌موقع لغو کنی؛ [۲.۹.۳](../03-streams/README.fa.md) نشانت داد چطور یک دنباله‌ی مقدارهایی را که یکی‌یکی، در طولِ زمان می‌رسند، پردازش کنی. این درس، آخرینِ ماژول، سراغِ چیزِ دیگری می‌رود: نه شکلِ کار، بلکه دو جایی که async با واقعیت‌هایِ دیگرِ Rust برخورد می‌کند و لبه‌ی تیزش را نشان می‌دهد.

لبه‌ی اول قراردادهاست. یک سرویسِ بک‌اندِ واقعی — دقیقاً فاز ۳، جایی که این ماژول رویش فرود می‌آید — پر است از صفت‌هایی مثلِ «هرچیزی که بتواند یک کاربر را با شناسه پیدا کند» یا «هرچیزی که بتواند یک پیام را جایی تحویل بدهد». حالا که متدهایِ این صفت‌ها async‌اند (طبیعی‌ترین حالتِ ممکن، چون همه‌شان دارند با شبکه یا پایگاه‌داده کار می‌کنند)، یک سؤالِ ساده جلویت می‌ایستد: می‌شود چند پیاده‌سازیِ متفاوت از این صفت را، پشتِ یک نوعِ واحد، تویِ یک `Vec` نگه داشت — همان `dyn Trait`ی که [۲.۳.۷](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.fa.md) بهت داد؟ جوابش، برایِ یک `async fn`ِ خام، نه است. بخشِ اولِ این درس همین نه را با کدِ واقعی نشانت می‌دهد و راهِ حلش را هم می‌دهد.

لبه‌ی دوم واقعیتِ فیزیکی است. نه هر کاری async است. هش‌کردنِ یک پسورد، فشرده‌سازیِ یک عکس، صدا زدنِ یک کتابخانه‌ی C که خودش هیچ ایده‌ای از `tokio` ندارد — همه‌شان یا واقعاً پردازنده را مشغول می‌کنند یا واقعاً مسدودکننده‌اند، و هیچ‌کدام دورِ `.await` نمی‌چرخند. بخشِ دومِ این درس نشانت می‌دهد این‌جور کار، اگر مستقیم داخلِ یک `async fn` بیفتد، دقیقاً چه بلایی سرِ همه‌یِ تسک‌هایِ دیگرِ همان ریسمانِ کارگر می‌آورد — و `spawn_blocking` چطور همان کار را بدونِ آن بلا انجام می‌دهد.

---

## مفهوم

### نوشتنِ `async fn` مستقیم داخلِ یک صفت

از چند سالِ پیش (Rust 1.75 به بعد؛ toolchainِ همین دوره خیلی جلوترش است) دیگر لازم نیست برایِ نوشتنِ یک صفت با متدهایِ async هیچ ماکرویی وارد کنی. یک صفتِ کوچک بساز، دقیقاً مثلِ هر صفتِ دیگری، فقط با یک `async fn` داخلش:

```rust
trait Fetcher {
    async fn fetch(&self) -> String;
}

struct Server;

impl Fetcher for Server {
    async fn fetch(&self) -> String {
        "data from Server".to_string()
    }
}

async fn print_fetch<F: Fetcher>(f: &F) {
    println!("{}", f.fetch().await);
}
```

همین. نه ماکرو، نه هیچ چیزِ اضافه — درست مثلِ هر `impl Trait for Type`ِ دیگری که [۲.۳.۱](../../03-traits-and-generics/01-defining-and-implementing-traits/README.fa.md) بهت داد، فقط این‌بار متدش `async` است. `print_fetch` هم یک تابعِ جنریکِ کراندارِ معمولی است: ارسالِ ایستا، یک نسخه‌ی کامپایل‌شده به‌ازایِ هر نوعِ ملموسی که واقعاً `Fetcher` را پیاده کند — دقیقاً همان مکانیزمی که [۲.۳.۷](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.fa.md) به اسمِ ارسالِ ایستا معرفی کرد. صدایش بزن:

```rust
#[tokio::main]
async fn main() {
    print_fetch(&Server).await;
}
```

```text
data from Server
```

### چرا `Box<dyn Fetcher>` کامپایل نمی‌شود: ایمنیِ شیء، این‌بار برایِ متدهایِ async

[۲.۳.۷](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.fa.md) دو دلیلِ شایعِ شکستنِ ایمنیِ شیء را نشانت داد — متدی که `Self` را با مقدار برمی‌گرداند، و متدی با پارامترِ جنریکِ خودش — و صریح گفت این‌ها «شایع‌ترین دلیلِ واقعی‌اند، نه فهرستِ کاملِ قانون‌ها»، و وعده داد همین‌جا برمی‌گردد و دیوارِ سومی را که سال‌ها `async fn` تویِ یک صفت با آن دست‌وپنجه نرم می‌کرد نشانت می‌دهد. حالا برگرد به `Fetcher` بالا و همان کاری را بکن که با `Spinoff` و `Rated` کردی — یک `dyn Fetcher` بساز:

```rust
#[tokio::main]
async fn main() {
    let f: Box<dyn Fetcher> = Box::new(Server);
    println!("{}", f.fetch().await);
}
```

کامپایل نمی‌شود — خطایِ دقیقش را در «خطاهایی که خواهی دید» می‌بینی، همان `E0038`ی که [۲.۳.۷](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.fa.md) از قبل می‌شناسی، فقط این‌بار پیامش می‌گوید «because method `fetch` is `async`». دلیلش هم دقیقاً همان vtable است: پشتِ `dyn Fetcher`، نوعِ واقعی پاک شده، و کامپایلر برایِ هر متد یک slotِ ثابت تویِ آن جدول رزرو می‌کند. یک `async fn`، وقتی صدا زده می‌شود، نوعِ بی‌نامِ خودش را برمی‌گرداند — یک ماشینِ وضعیت که کامپایلر برایِ همان یک `async fn` می‌سازد، اندازه‌اش بسته به بدنه‌اش فرق می‌کند، و هیچ دو `async fn`ی همین نوع را ندارند. یک vtable نمی‌تواند slotی بسازد که اندازه‌اش را نمی‌داند — دقیقاً همان مشکلِ `Self` با مقدار، از زاویه‌ای دیگر.

قبل از Rust 1.75، این دیوار حتی گسترده‌تر بود: خودِ نحوِ `async fn` تویِ صفت اصلاً وجود نداشت؛ برایِ نوشتنِ *هر* صفتی با متدِ async — چه قرار بود روزی `dyn` شود چه نه — یک ماکرو اجباری بود. همان ماکرو، `#[async_trait]`، هنوز هست؛ فقط حالا کارش محدود شده به دقیقاً همین یک مسئله.

```senpai-visual
{"kind":"async","labels":["async fn در یک صفت: ارسالِ ایستا، بدونِ تخصیص","dyn Trait به یک vtable با جای ثابت نیاز دارد","async fn هیچ اندازه‌ی ثابتی برایِ آن جا ندارد","async_trait: بازنویسی به فیوچرِ جعبه‌ای","حالا شیء صفتی ممکن است، با یک تخصیصِ هیپ در هر صدا"]}
```

### `#[async_trait]`: چطور صفت را dyn-سازگار می‌کند، و به چه قیمتی

راهِ حل همان صفت است، فقط با یک خط اضافه — رویِ خودِ تعریفِ صفت، **و** رویِ هر `impl` آن:

```rust
use async_trait::async_trait;

#[async_trait]
trait Fetcher {
    async fn fetch(&self) -> String;
}

struct Server;

#[async_trait]
impl Fetcher for Server {
    async fn fetch(&self) -> String {
        "data from Server".to_string()
    }
}
```

```rust
#[tokio::main]
async fn main() {
    let f: Box<dyn Fetcher> = Box::new(Server);
    println!("{}", f.fetch().await);
}
```

```text
data from Server
```

این‌بار کامپایل می‌شود. `#[async_trait]` عملاً امضایِ `fetch` را زیرِ پوست به چیزی شبیهِ `fn fetch<'a>(&'a self) -> Pin<Box<dyn Future<Output = String> + Send + 'a>>` بازمی‌نویسد — همان امضایِ خطایِ `E0195`ای که در «خطاهایی که خواهی دید» می‌بینی، وقتی این بازنویسی رویِ صفت اتفاق بیفتد ولی رویِ `impl` نه. یک تابعِ معمولی که یک `Future`ِ جعبه‌شده برمی‌گرداند، اندازه‌اش همیشه یک اشاره‌گر است — دقیقاً همان چیزی که یک vtable می‌تواند برایش slot باز کند. مسئله حل شد، ولی رایگان نبود: هر صدا زدنِ `fetch` حالا یک `Box::pin` واقعی می‌سازد — یک تخصیصِ هیپ — که نسخه‌ی خامِ بخشِ قبل هرگز نمی‌پرداخت.

### قاعده‌ی تصمیم: کِی خام، کِی `#[async_trait]`

یک خط: **پیش‌فرض، `async fn` را مستقیم تویِ صفت بنویس؛ سراغِ `#[async_trait]` فقط وقتی برو که واقعاً به `dyn Trait` — یعنی `Box<dyn Trait>`، یا هر جایِ دیگری که چند پیاده‌سازیِ متفاوت باید پشتِ یک نوعِ واحد بنشینند — نیاز داری.** اگر همیشه با یک نوعِ ملموسِ ثابت کار می‌کنی (یا با یک کراندِ جنریک، مثلِ `print_fetch` بالا)، خامش رایگان‌تر است — هیچ تخصیصِ اضافه‌ای در کار نیست. اگر یک `Vec<Box<dyn Repository>>` لازم داری، یا یک تابع می‌خواهد «هرچیزی که این صفت را دارد» بگیرد بدونِ اینکه نوعش را جنریک کند، آن‌وقت هزینه‌ی یک تخصیصِ هیپ در هر صدا، قیمتِ واقعاً کمی است برایِ چیزی که خامش اصلاً نمی‌تواند بدهد.

### چرا یک کارِ سنگین یا مسدودکننده، محیطِ اجرا را متوقف می‌کند

[۲.۸.۵](../../08-concurrency/05-futures-and-runtimes/README.fa.md) این را «زمان‌بندیِ مشارکتی» نامید: یک تسک فقط دقیقاً همان‌جاهایی که خودش صریحاً مشخص کرده — نقاطِ `.await` — کنترل را به محیطِ اجرا پس می‌دهد، نه هر لحظه‌ای که سیستم‌عامل بخواهد. این یعنی محیطِ اجرا فقط زمانی می‌تواند تسکِ دیگری را رویِ همان ریسمانِ کارگر جلو ببرد که تسکِ فعلی به یک `.await` برسد و آن‌جا معلق شود. یک کارِ واقعاً سنگینِ پردازنده — یا یک تماسِ مسدودکننده به یک کتابخانه‌ی همگام — هیچ‌کدامِ این‌ها نیست: هیچ `.await`ی داخلش نیست، پس هرگز کنترل را پس نمی‌دهد، تا وقتی خودش تمام شود.

این را ببین — `slow_sum` یک محاسبه‌ی واقعیِ همگام است، هیچ async‌ای تویِ بدنه‌اش نیست، و `ticker` هر ۵۰ میلی‌ثانیه یک تیک باید بزند:

```rust
fn slow_sum(n: u64) -> u64 {
    let mut total: u64 = 0;
    for i in 0..n {
        total = total.wrapping_add(i);
    }
    total
}

async fn ticker() {
    for tick in 1..=3 {
        tokio::time::sleep(Duration::from_millis(50)).await;
        println!("tick {tick}");
    }
}
```

با یک محیطِ اجرایِ `current_thread` (یک ریسمانِ کارگرِ تنها — همانی که [۲.۸.۶](../../08-concurrency/06-tokio-basics/README.fa.md) معرفی کرد) `ticker` را spawn کن، بعد `slow_sum` را **مستقیم**، بدونِ هیچ `spawn_blocking`ی، صدا بزن:

```rust
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let start = Instant::now();
    let ticks = tokio::spawn(ticker());

    println!("heavy result: {}", slow_sum(20_000_000));

    ticks.await.unwrap();
    println!("elapsed: {:?}", start.elapsed());
}
```

```text
heavy result: 199999990000000
tick 1
tick 2
tick 3
elapsed: 304.935ms
```

`ticker` قبل از `slow_sum` spawn شد — قرار بود از همان اول هر ۵۰ میلی‌ثانیه یک تیک بزند. ولی هیچ «tick»ی، حتیٰ یکی، پیش از «heavy result» چاپ نمی‌شود — این هم شانس نیست، تضمینِ خودِ مدلِ زمان‌بندیِ مشارکتی است: با یک ریسمانِ کارگرِ تنها، تا وقتی `slow_sum` برنگردد، هیچ‌چیزِ دیگری اصلاً فرصتِ اجرا پیدا نمی‌کند. عددِ دقیقِ `elapsed` رویِ ماشینِ خودت فرق می‌کند، ولی این ترتیب — صفر تیک پیش از «heavy result» — همیشه همین است.

### `spawn_blocking`: بردنِ کار به استخرِ ریسمانِ جداگانه

`tokio::task::spawn_blocking` دقیقاً همین مشکل را حل می‌کند: یک کلوژرِ معمولی (نه `async`) می‌گیرد و آن را به استخرِ ریسمانِ مسدودکننده‌یِ جداگانه‌یِ `tokio` می‌سپارد — کاملاً جدا از ریسمان‌هایِ کارگرِ async — و یک `JoinHandle` برمی‌گرداند، دقیقاً همان نوعی که [۲.۹.۱](../01-spawn-joinset-structured-concurrency/README.fa.md) از `tokio::spawn` گرفتی، همان‌طور `.await`اش می‌کنی تا نتیجه را پس بگیری. همان `slow_sum` و همان `ticker`، فقط یک تغییر در `main`:

```rust
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let start = Instant::now();
    let ticks = tokio::spawn(ticker());

    let heavy = tokio::task::spawn_blocking(|| slow_sum(20_000_000))
        .await
        .unwrap();
    println!("heavy result: {heavy}");

    ticks.await.unwrap();
    println!("elapsed: {:?}", start.elapsed());
}
```

```text
tick 1
heavy result: 199999990000000
tick 2
tick 3
elapsed: 169.5612ms
```

این‌بار «tick 1» پیش از «heavy result» چاپ شد — ریسمانِ کارگرِ تنها، تا وقتی `slow_sum` رویِ استخرِ جداگانه در حالِ اجراست، آزاد بود کارِ `ticker` را جلو ببرد. **دقیقاً چند تیک پیش از «heavy result» بیاید، اجرا به اجرا فرق می‌کند** — این تایمینگِ واقعیِ بینِ دو ریسمانِ جداست، نه یک تضمین؛ همین اجرا را چند بار که گرفتم، یک‌بار فقط یک تیک پیش از «heavy result» آمد، یک‌بارِ دیگر دو تا. تنها چیزی که تضمین‌شده این است: **دستِ‌کم یک تیک** همیشه پیش از «heavy result» می‌آید — چیزی که در نسخه‌ی بالا، بدونِ `spawn_blocking`، هرگز اتفاق نمی‌افتد. عددِ `elapsed` هم رویِ این ماشین، در این اجرا، از ۳۰۴ به حدودِ ۱۷۰ میلی‌ثانیه افتاد: چون حالا کارِ سنگین و تیک‌زدن هم‌پوشانی دارند، به‌جایِ اینکه پشتِ‌سرِ هم صف بکشند.

```senpai-visual
{"kind":"concurrency","labels":["کارِ سنگین مستقیم داخلِ async fn","ریسمانِ کارگر تا پایانِ آن کار گیر می‌کند","بقیه‌ی تسک‌ها هیچ فرصتی برای اجرا ندارند","spawn_blocking: همان کار رویِ استخرِ ریسمانِ جداگانه","ریسمانِ کارگر آزاد می‌ماند، بقیه‌ی تسک‌ها ادامه می‌دهند"]}
```

### قاعده‌ی تصمیم: کِی `spawn_blocking`، کِی نه

یک خط: **`spawn_blocking` را برایِ کارِ واقعاً سنگینِ پردازنده یا یک تماسِ مسدودکننده‌یِ اجتناب‌ناپذیر بردار — نه برایِ هر تابعِ همگامی که فقط اتفاقاً سریع است.** خودِ `spawn_blocking` رایگان نیست: یک کارِ تازه رویِ یک ریسمانِ دیگر برنامه‌ریزی می‌کند، که برایِ محاسبه‌ای در حدِ میکروثانیه — پارس‌کردنِ یک رشته، جمع‌زدنِ چند عدد — سربارش از خودِ کار بیشتر می‌شود. مرزِ عملی: اگر داری به یک کتابخانه‌ی همگام زنگ می‌زنی که خودش هیچ نسخه‌ی `.await`پذیری ندارد، یا محاسبه‌ات واقعاً چند میلی‌ثانیه یا بیشتر طول می‌کشد، `spawn_blocking` بردار؛ اگر کارت سریع است، همان‌جا، مستقیم صدایش بزن — یک `.await` دورش نپیچ که کاری نمی‌کند جز اضافه‌کردنِ یک لایه‌ی بی‌فایده.

---

## دست‌به‌کد

(هر اجرا اول یک هشدارِ `unused variable: n` هم نشانت می‌دهد — از تابعِ هنوز-`todo!()`ِ `run_cpu_work_off_the_runtime` تویِ `src/lib.rs`ی که پایین‌تر، در «تمرین»، خودت کاملش می‌کنی. بی‌خیالش شو؛ خروجیِ زیرش است که مهم است.)

```sh
cargo run -p p2-09-04-async-traits-and-blocking --example 01-native-async-trait
cargo run -p p2-09-04-async-traits-and-blocking --example 02-async-trait-macro
cargo run -p p2-09-04-async-traits-and-blocking --example 03-blocking-call-stalls-the-runtime
cargo run -p p2-09-04-async-traits-and-blocking --example 04-spawn-blocking-fixes-it
```

بعد سه‌تای خراب:

```sh
cargo run -p p2-09-04-async-traits-and-blocking --example 05-dyn-native-async-trait-broken --features broken
cargo run -p p2-09-04-async-traits-and-blocking --example 06-forgot-async-trait-on-impl-broken --features broken
cargo run -p p2-09-04-async-traits-and-blocking --example 07-spawn-blocking-forgot-move-broken --features broken
```

بعد این‌ها را امتحان کن:

۱. در `03-blocking-call-stalls-the-runtime`، عددِ `20_000_000` را ده برابر کن. عددِ «elapsed» چطور تغییر می‌کند؟ ترتیبِ خطوط (همه‌ی «tick»ها بعدِ «heavy result») هم عوض می‌شود؟
۲. در `04-spawn-blocking-fixes-it`، چند بار پشتِ سرِ هم اجرایش کن. هر بار دقیقاً چند «tick» پیش از «heavy result» می‌آید؟ همیشه یکی است؟
۳. در `02-async-trait-macro`، خطِ `#[async_trait]` را از بالایِ `impl Fetcher for Server` بردار (نه از بالایِ خودِ `trait`). چه خطایی می‌گیری؟ (سرنخ: همان چیزی که در «خطاهایی که خواهی دید» می‌بینی.)

---

## خطاهایی که خواهی دید

### `E0038` — یک صفتِ async، «dyn compatible» نیست

```text
error[E0038]: the trait `Fetcher` is not dyn compatible
  --> phase2-intermediate\09-async-in-practice\04-async-traits-and-blocking\examples\05-dyn-native-async-trait-broken.rs:23:20
   |
23 |     let f: Box<dyn Fetcher> = Box::new(Server);
   |                    ^^^^^^^ `Fetcher` is not dyn compatible
   |
note: for a trait to be dyn compatible it needs to allow building a vtable
      for more information, visit <https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility>
  --> phase2-intermediate\09-async-in-practice\04-async-traits-and-blocking\examples\05-dyn-native-async-trait-broken.rs:10:14
   |
 9 | trait Fetcher {
   |       ------- this trait is not dyn compatible...
10 |     async fn fetch(&self) -> String;
   |              ^^^^^ ...because method `fetch` is `async`
   = help: consider moving `fetch` to another trait
   = help: only type `Server` implements `Fetcher`; consider using it directly instead.

For more information about this error, try `rustc --explain E0038`.
```

**کامپایلر به چه اعتراض دارد:** پیامش دقیق است — `fetch` به این دلیل که `async` است، ایمنیِ شیء را می‌شکند. یک `async fn`، پشتِ صحنه، نوعِ بی‌نامِ خودش را برمی‌گرداند — یک ماشینِ وضعیت که اندازه‌اش بسته به بدنه‌ی همان تابع فرق می‌کند. یک vtable باید پیش از هر صدایی بداند هر متد چه اندازه‌ای برمی‌گرداند؛ نوعی که اندازه‌اش از پیش معلوم نیست، جایی تویِ آن جدول ندارد.

**راه‌حل:** `#[async_trait]` را رویِ خودِ صفت بگذار، **و** رویِ هر `impl` آن — دقیقاً همان‌طور که در «مفهوم» دیدی:

```rust
#[async_trait]
trait Fetcher {
    async fn fetch(&self) -> String;
}
```

**چرا این راه‌حل است:** `#[async_trait]` امضایِ `fetch` را به چیزی بازمی‌نویسد که یک `Future`ِ جعبه‌شده برمی‌گرداند — `Pin<Box<dyn Future<Output = String> + Send>>` — و اندازه‌ی یک `Box` همیشه ثابت است، هرچه پشتش باشد. حالا vtable می‌تواند slotی برایش باز کند، به قیمتِ یک تخصیصِ هیپ در هر صدا.

### `E0195` — امضایِ `impl` با صفت جور درنمی‌آید

```text
error[E0195]: lifetime parameters or bounds on method `fetch` do not match the trait declaration
  --> phase2-intermediate\09-async-in-practice\04-async-traits-and-blocking\examples\06-forgot-async-trait-on-impl-broken.rs:20:19
   |
12 | #[async_trait]
   | -------------- this bound might be missing in the impl
13 | trait Fetcher {
14 |     async fn fetch(&self) -> String;
   |              ------------
   |              |     |
   |              |     this bound might be missing in the impl
   |              lifetimes in impl do not match this method in trait
...
20 |     async fn fetch(&self) -> String {
   |                   ^ lifetimes do not match method in trait

For more information about this error, try `rustc --explain E0195`.
```

**کامپایلر به چه اعتراض دارد:** `#[async_trait]` رویِ خودِ `trait Fetcher` نشسته، پس امضایِ `fetch` تویِ تعریفِ صفت از قبل به شکلِ بازنویسی‌شده (با یک طول‌عمرِ اضافه برایِ `Future`ِ جعبه‌شده) درآمده. `impl Fetcher for Server`، بدونِ آن ماکرو رویِ خودش، همچنان امضایِ اصلی و ساده را نوشته — دو امضا دیگر یکی نیستند، و Rust یک `impl` را فقط وقتی می‌پذیرد که دقیقاً همان امضایِ صفت را تکرار کند.

**راه‌حل:** همان `#[async_trait]` را رویِ `impl` هم بگذار:

```rust
#[async_trait]
impl Fetcher for Server {
    async fn fetch(&self) -> String {
        "data from Server".to_string()
    }
}
```

**چرا این راه‌حل است:** حالا هر دو طرف — صفت و پیاده‌سازی — از یک ماکرو رد شده‌اند و دقیقاً همان امضایِ بازنویسی‌شده را دارند. `#[async_trait]` یک تصمیمِ همه‌یا-هیچ است: یا رویِ صفت و همه‌ی `impl`هایش می‌نشیند، یا اصلاً رویِ هیچ‌کدام.

### `E0373` — کلوژرِ `spawn_blocking` بدونِ `move`

```text
error[E0373]: closure may outlive the current function, but it borrows `n`, which is owned by the current function
  --> phase2-intermediate\09-async-in-practice\04-async-traits-and-blocking\examples\07-spawn-blocking-forgot-move-broken.rs:21:46
   |
21 |     let handle = tokio::task::spawn_blocking(|| slow_sum(n));
   |                                              ^^          - `n` is borrowed here
   |                                              |
   |                                              may outlive borrowed value `n`
   |
note: function requires argument type to outlive `'static`
  --> phase2-intermediate\09-async-in-practice\04-async-traits-and-blocking\examples\07-spawn-blocking-forgot-move-broken.rs:21:18
   |
21 |     let handle = tokio::task::spawn_blocking(|| slow_sum(n));
   |                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
help: to force the closure to take ownership of `n` (and any other referenced variables), use the `move` keyword
   |
21 |     let handle = tokio::task::spawn_blocking(move || slow_sum(n));
   |                                              ++++

For more information about this error, try `rustc --explain E0373`.
```

**کامپایلر به چه اعتراض دارد:** کلوژرِ `spawn_blocking` رویِ یک ریسمانِ کاملاً دیگر اجرا می‌شود، شاید مدت‌ها بعد از اینکه تابعِ فراخواننده — همینجا `main` — برگشته باشد. برایِ همین `spawn_blocking` از کلوژرش می‌خواهد `'static` باشد: هرچه لازم دارد را خودش مالک باشد، نه اینکه فقط قرضش کرده باشد. کلوژرِ `|| slow_sum(n)` فقط `n` را قرض می‌کند، و `n` مالِ `main` است — قولی که کامپایلر نمی‌تواند نگه دارد.

**راه‌حل:** `move` را جلویِ کلوژر بگذار:

```rust
let handle = tokio::task::spawn_blocking(move || slow_sum(n));
```

**چرا این راه‌حل است:** `n` یک `u64` است — `Copy` — پس `move` هزینه‌ای ندارد، فقط مالکیتِ یک کپیِ کوچک را به خودِ کلوژر می‌دهد. حالا کلوژر دیگر به هیچ‌چیزِ بیرونِ خودش وابسته نیست؛ می‌تواند هر جا، هر وقت، رویِ استخرِ ریسمانِ مسدودکننده اجرا شود، بدونِ اینکه به عمرِ `n` تویِ `main` وابسته باشد.

---

## تمرین

### گرم‌کردن

<details>
<summary>این چه چیزی چاپ می‌کند؟</summary>

```rust
trait Namer {
    async fn name(&self) -> String;
}

struct Cat;

impl Namer for Cat {
    async fn name(&self) -> String {
        "Neko".to_string()
    }
}

async fn print_name<N: Namer>(n: &N) {
    println!("{}", n.name().await);
}
```

فرض کن این رو داخلِ `main` صدا می‌زنیم: `print_name(&Cat).await;`

</details>

<details>
<summary>پاسخ</summary>

```text
Neko
```

هیچ چیزِ غافلگیرکننده‌ای نیست — یک `async fn` تویِ صفت، صدا زده‌شده از پشتِ یک کراندِ جنریک، دقیقاً مثلِ هر متدِ صفتِ دیگری کار می‌کند.

</details>

<details>
<summary>کامپایل می‌شود؟</summary>

```rust
trait Namer {
    async fn name(&self) -> String;
}

struct Cat;

impl Namer for Cat {
    async fn name(&self) -> String {
        "Neko".to_string()
    }
}

fn boxed() -> Box<dyn Namer> {
    Box::new(Cat)
}
```

</details>

<details>
<summary>پاسخ</summary>

نه — `E0038`، «the trait `Namer` is not dyn compatible»، دقیقاً به همان دلیلی که `Fetcher` بالا شکست: `name` یک `async fn` است، و یک `async fn` جایِ ثابتی تویِ vtable ندارد. رفعش هم همان است — `#[async_trait]` رویِ صفت و رویِ `impl`.

</details>

<details>
<summary>کامپایل می‌شود؟</summary>

```rust
use async_trait::async_trait;

#[async_trait]
trait Namer {
    async fn name(&self) -> String;
}

struct Cat;

impl Namer for Cat {
    async fn name(&self) -> String {
        "Neko".to_string()
    }
}
```

</details>

<details>
<summary>پاسخ</summary>

نه — `E0195`. `#[async_trait]` رویِ خودِ `trait Namer` نشسته و امضایِ `name` را بازنویسی کرده، ولی `impl Namer for Cat` بدونِ آن ماکرو، امضایِ سادهٔ اصلی را دارد. `#[async_trait]` همه‌یا-هیچ است — یا رویِ صفت و همه‌ی `impl`هایش، یا هیچ‌کدام.

</details>

<details>
<summary>درست یا غلط: با یک محیطِ اجرایِ <code>current_thread</code>، اگر یک تسکِ دیگر را spawn کرده باشی و بعد مستقیم (بدونِ <code>spawn_blocking</code>) یک تابعِ همگامِ چندصدمیلی‌ثانیه‌ای صدا بزنی، ممکن است آن تسکِ دیگر همچنان، درست همان وسط، یک قدم جلو برود.</summary>

</details>

<details>
<summary>پاسخ</summary>

غلط. با یک ریسمانِ کارگرِ تنها، تا وقتی تابعِ همگام برنگردد، هیچ‌چیزِ دیگری اصلاً فرصتِ اجرا پیدا نمی‌کند — نه یک قدم، نه نیم‌قدم. کنترل فقط سرِ یک `.await` پس داده می‌شود، و یک تابعِ همگام هیچ `.await`ی ندارد.

</details>

### تعمیر

هر سه مثالِ خراب را درست کن:

۱. `examples/05-dyn-native-async-trait-broken.rs` را با اضافه‌کردنِ `#[async_trait]` رویِ `trait Fetcher` **و** رویِ `impl Fetcher for Server` درست کن.
۲. `examples/06-forgot-async-trait-on-impl-broken.rs` را با اضافه‌کردنِ همان `#[async_trait]`ی که رویِ صفت هست، این‌بار رویِ `impl` هم، درست کن.
۳. `examples/07-spawn-blocking-forgot-move-broken.rs` را با گذاشتنِ `move` جلویِ کلوژرِ `spawn_blocking` درست کن.

### پیاده‌سازی

چهار چیز در `src/lib.rs` — دوتایشان از قبل کامل‌اند، دوتا `todo!()` دارند:

- `sum_range(n: u64) -> u64` — از قبل کامل. جمعِ `0..n` را با جمعِ چرخشی (`wrapping_add`) حساب می‌کند؛ کارِ سنگینِ پردازنده‌ای که تمرینِ زیر می‌بردش.
- `greet_with<G: Greeter>(greeter: &G, name: &str) -> String` — از قبل کامل. `greeter.greet(name)` را `.await` می‌کند و نتیجه را برمی‌گرداند — یک تابعِ جنریکِ کراندار، بدونِ هیچ `dyn`ی.
- **TODO** — `Formal::greet(&self, name: &str) -> String`: رشته‌ی دقیقاً `format!("Good day, {name}.")` را برمی‌گرداند — مثلاً برایِ `name = "Sara"`، رشته‌ی `"Good day, Sara."`.
- **TODO** — `run_cpu_work_off_the_runtime(n: u64) -> u64`: باید نتیجه‌ی `sum_range(n)` را برگرداند، **بدونِ اینکه هرگز مستقیم `sum_range` را داخلِ همین `async fn` صدا بزند** — با `spawn_blocking`، آن را رویِ استخرِ ریسمانِ مسدودکننده‌یِ `tokio` اجرا کن و نتیجه‌اش را برگردان.

```sh
cargo test -p p2-09-04-async-traits-and-blocking
```

### بساز

یک صفتِ `Greeter` دومِ‌شکل — `Casual` — اضافه کن که `greet` رویش دقیقاً رشته‌ی `format!("Hey {name}!")` برمی‌گرداند. حالا که دو نوعِ متفاوت داری که هر دو `Greeter`اند و می‌خواهی هر دو را، پشتِ یک نوعِ واحد، تویِ یک `Vec` نگه داری، خودِ `Greeter` را بازسازی کن: `#[async_trait]` را رویِ تعریفِ صفت بگذار، و رویِ `impl`هایِ `Formal` و `Casual` هر دو. بعد بنویس:

```rust
pub async fn greet_all(greeters: &[Box<dyn Greeter>], name: &str) -> Vec<String>
```

که هر عضوِ `greeters` را به‌ترتیب `.await` می‌کند و نتیجه‌ها را، به همان ترتیب، تویِ یک `Vec` برمی‌گرداند.

### چالش (اختیاری)

`pub async fn sum_many_off_the_runtime(ns: Vec<u64>) -> Vec<u64>` بنویس: به‌ازایِ هر `n` تویِ `ns`، یک `spawn_blocking(move || sum_range(n))` جداگانه بزن — همه‌شان، در یک گذرِ واحد، پیش از هر `.await`ای — بعد یکی‌یکی `.await`شان کن و نتیجه‌ها را به همان ترتیبِ `ns` برگردان. دقیقاً همان الگویِ spawn-همه-اول-بعد-await که [۲.۹.۱](../01-spawn-joinset-structured-concurrency/README.fa.md) برایِ `tokio::spawn` یادت داد — `spawn_blocking` هم `JoinHandle`ای از همان نوع برمی‌گرداند، پس همان الگو، بدونِ هیچ تغییری، این‌جا هم کار می‌کند.

---

## جمع‌بندی

مسیرِ کاملِ همین ماژول، از اول تا آخر: [۲.۹.۱](../01-spawn-joinset-structured-concurrency/README.fa.md) نشانت داد چطور چند تسک را با `spawn` و `JoinSet` هم‌زمان راه بیندازی و منتظرشان بمانی. [۲.۹.۲](../02-select-and-cancellation-safety/README.fa.md) نشانت داد چطور با `select!` رویِ چند `Future` هم‌زمان مسابقه بدهی و یکی را به‌موقع لغو کنی. [۲.۹.۳](../03-streams/README.fa.md) نشانت داد چطور با `Stream` یک دنباله‌ی async را، عضو به عضو، پردازش کنی. امروز، آخرین درس، دو لبه‌ی عملیِ متدهایِ صفتِ async را دید: ایمنیِ شیء — چرا `async fn` تویِ یک صفت نمی‌تواند `dyn` شود، و `#[async_trait]` چطور، به قیمتِ یک تخصیصِ هیپ، این را عوض می‌کند — و واقعیتِ فیزیکیِ کار — چرا یک کارِ سنگین یا مسدودکننده محیطِ اجرا را متوقف می‌کند، و `spawn_blocking` چطور بدونِ آن توقف انجامش می‌دهد. با همین، ماژولِ ۲.۹ — و فاز ۲ — یک بخشِ دیگر هم دارد: [۲.۱۰ — جعبه‌ابزارِ Rust](../../10-rust-toolbox/README.fa.md)، چهار چیزِ باقی‌مانده که به هیچ ماژولی به‌تنهایی تعلق ندارند — تطبیقِ الگو از نزدیک، اولین ماکروی خودت، ویژگی‌هایِ cargo، و `unsafe` وقتی جدی می‌نویسیش.

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| `async fn` تویِ صفت (AFIT) | نوشتنِ `async fn` مستقیم داخلِ یک صفت، بدونِ ماکرو | صفت‌های async، پیش‌فرض — تا وقتی `dyn` لازم نشود |
| ایمنیِ شیء (این‌بار برایِ async) | دلیلِ سوم — بعدِ `Self` با مقدار و متدِ جنریکِ [۲.۳.۷](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.fa.md) — که یک صفت `dyn` نمی‌شود | خطایِ `E0038` رویِ هر `async fn`ِ خامِ پشتِ `dyn` |
| `#[async_trait]` | ماکرویی که `async fn` را به متدی با خروجیِ `Pin<Box<dyn Future<...> + Send>>` بازمی‌نویسد | فقط وقتی `Box<dyn Trait>` واقعاً لازم است |
| `spawn_blocking` | یک کلوژرِ همگام را رویِ استخرِ ریسمانِ مسدودکننده‌یِ جداگانه‌یِ `tokio` اجرا می‌کند | کارِ سنگینِ پردازنده یا تماسِ مسدودکننده‌یِ اجتناب‌ناپذیر |

### الان می‌دانی

- چرا می‌توانی `async fn` را مستقیم تویِ یک صفت بنویسی، بدونِ هیچ ماکرویی، و آن را از پشتِ یک کراندِ جنریک صدا بزنی.
- چرا همان صفتِ خام، پشتِ `Box<dyn Trait>`، با `E0038` شکست می‌خورد — و این دقیقاً همان دیوارِ ایمنیِ شیءِ [۲.۳.۷](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.fa.md) است، از یک زاویه‌ی تازه.
- `#[async_trait]` چطور آن دیوار را برمی‌دارد — با بازنویسیِ متد به یک `Future`ِ جعبه‌شده — و دقیقاً چه هزینه‌ای (یک تخصیصِ هیپ در هر صدا) برایِ این کار می‌پردازی.
- چرا یک کارِ سنگینِ پردازنده یا یک تماسِ مسدودکننده، اگر مستقیم داخلِ یک `async fn` بیفتد، بقیه‌ی تسک‌هایِ همان ریسمانِ کارگر را کاملاً گرسنه نگه می‌دارد.
- `spawn_blocking` چطور همان کار را رویِ یک استخرِ ریسمانِ جداگانه می‌برد، و کِی واقعاً به‌کارش می‌بری — نه برایِ هر تابعِ همگامِ سریع.

### بعداً کامل‌تر می‌بینی

- **صفت‌هایِ async و `spawn_blocking`، تویِ یک بک‌اندِ واقعی** — این درس هیچ‌کدام را نیمه نگذاشت؛ فقط از این‌جا به بعد دیگر «تمرینِ یک درس» نیستند، بخشِ عادیِ کدی می‌شوند که برایِ یک سرویسِ واقعی می‌نویسی — [فاز ۳ — پایه‌هایِ بک‌اند](../../../phase3-backend-foundations/README.fa.md).

### می‌توانی توضیح بدهی؟

- چرا یک `async fn` تویِ یک صفت، از پشتِ یک کراندِ جنریک، بدونِ هیچ مشکلی کار می‌کند ولی پشتِ `Box<dyn Trait>` کامپایل نمی‌شود؟
- `#[async_trait]` واقعاً چه‌کار می‌کند؟ چرا باید هم رویِ صفت باشد هم رویِ هر `impl`ش؟ چه هزینه‌ای می‌پردازی که نسخه‌ی خام نمی‌پردازد؟
- چرا یک کارِ سنگینِ پردازنده، اگر مستقیم داخلِ یک `async fn` صدا زده شود، بقیه‌ی تسک‌هایِ همان ریسمانِ کارگر را گرسنه نگه می‌دارد؟ این را با زبانِ خودِ زمان‌بندیِ مشارکتی توضیح بده.
- `spawn_blocking` چطور این مشکل را حل می‌کند؟ چرا نباید آن را رویِ هر تابعِ همگامی — حتی یک تابعِ سریع — بگذاری؟

---

## بیشتر

- [مرجعِ ایمنیِ شیء (dyn compatibility)](https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility) — همان صفحه‌ای که خودِ کامپایلر، تویِ خطایِ `E0038`، لینکش می‌دهد.
- [مستنداتِ کریتِ `async-trait`](https://docs.rs/async-trait/latest/async_trait/) — شاملِ `#[async_trait(?Send)]`، برایِ وقتی `Future`ات نباید `Send` باشد.
- [مستنداتِ `spawn_blocking`](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html) — امضایِ دقیق و توضیحِ رسمیِ استخرِ ریسمانِ مسدودکننده.
- [فصلِ «Spawning» از کتابِ رسمیِ tokio](https://tokio.rs/tokio/tutorial/spawning) — همان صفحه‌ای که [۲.۸.۶](../../08-concurrency/06-tokio-basics/README.fa.md) معرفی کرد، تمایزِ تسک/ریسمان.

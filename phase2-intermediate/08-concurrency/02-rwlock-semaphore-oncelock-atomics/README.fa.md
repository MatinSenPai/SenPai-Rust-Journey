# ۲.۸.۲ — `RwLock`، `Semaphore`، `OnceLock`/`LazyLock`، اتمیک‌ها

## در یک نگاه

بعد از این درس می‌توانی:

- توضیح بدهی `RwLock` دقیقاً چه چیزی را از `Mutex` جدا می‌کند، و کِی این فرق واقعاً روی توانِ پردازشِ برنامه اثر می‌گذارد — نه فقط اینکه «امن‌تر» یا «سریع‌تر» است.
- یک سمافورِ شمارشیِ کارکن از `Mutex` و `Condvar` بسازی، بگویی چرا `std::sync` خودش این نوع را ندارد، و چرا یک permitِ RAII بهتر از صداکردنِ دستیِ آزادسازی است.
- بینِ `OnceLock`، `LazyLock` و یک اتمیک، برایِ یک مسئله‌ی واقعی یکی را انتخاب کنی — و توضیح بدهی چرا `Mutex`/`RwLock` آن‌جا زیاده‌روی بود.

**زمان:** حدود ۷۵ دقیقه · **پیش‌نیاز:**
[۲.۸.۱ — ریسمان‌ها، `Mutex` و `Arc`](../01-threads-mutex-arc/README.fa.md)

---

## چرا اهمیت دارد

۲.۸.۱ یک ابزار به‌ات داد که با آن هر داده‌یِ مشترک بینِ ریسمان‌ها را امن می‌کنی: `Mutex<T>`. قانونش ساده بود — هر لحظه، حداکثر یک ریسمان اجازه دارد به مقدار دست بزند، چه بخواهد فقط بخواندش، چه بخواهد عوضش کند. این قانون همیشه امن است، ولی همیشه بهینه نیست.

فرض کن یک کش یا یک تنظیماتِ برنامه داری که ده‌ها ریسمانِ هم‌زمان — مثلاً ده‌ها دستگیرنده‌ی درخواست در یک وب‌سرور — دائم می‌خوانندش، و فقط هرچند دقیقه یک‌بار یک ریسمانِ جداگانه به‌روزش می‌کند. با `Mutex`، حتی وقتی همه‌ی آن ریسمان‌ها فقط می‌خواهند بخوانند — کاری که اصلاً با هم تصادفی ندارد — باز هم باید یکی‌یکی صف بکشند، چون `Mutex` اصلاً فرقی بینِ «فقط می‌خواهم نگاه کنم» و «می‌خواهم عوضش کنم» نمی‌گذارد. هزار خواندنِ هم‌زمان که هیچ‌کدام به هم آسیبی نمی‌زدند، به‌خاطرِ قفلی که برایِ محافظت از نوشتن ساخته شده، به هزار خواندنِ پشتِ‌سرِهم تبدیل می‌شوند.

این درس دقیقاً همین شکاف را با `RwLock<T>` پر می‌کند: قفلی که بینِ «فقط‌خواندن» و «خواندن‌و‌نوشتن» فرق می‌گذارد و به هر تعداد خواننده اجازه‌ی هم‌زمان‌بودن می‌دهد. بعدش سه ابزارِ دیگر می‌آید که هرکدام یک مسئله‌ی متفاوتِ هم‌روندی را حل می‌کند: یک سمافورِ شمارشی برایِ محدودکردنِ تعدادِ ریسمان‌هایی که هم‌زمان به یک منبعِ محدود دست می‌زنند (که خودت می‌سازی‌اش — `std::sync` از این نوع ندارد)؛ `OnceLock`/`LazyLock` برایِ مقداردهیِ یک‌بارِ امن، حتی وقتی چند ریسمان هم‌زمان مسابقه می‌دهند کدام‌شان اول برسد؛ و اتمیک‌ها برایِ وقتی که یک عددِ کوچک کافی است و گرفتنِ یک قفلِ کامل زیاده‌روی است.

هر چهارتا از همان دو خانواده‌ی ابزاری می‌آیند که ۲.۸.۱ معرفی کرد — `std::sync` و ریسمان‌های واقعیِ سیستم‌عامل — فقط شکلِ دقیق‌تری از «چه کسی، کِی، به چه چیزی دست می‌زند» می‌پرسند.

---

## مفهوم

### `RwLock<T>`: هر تعداد خواننده، یا دقیقاً یک نویسنده

`RwLock<T>` («خواندن-نوشتن قفل») همان کاری را می‌کند که `Mutex<T>` می‌کرد — یک مقدار را طوری می‌پیچد که دو ریسمان هم‌زمان نتوانند بی‌قاعده بهش دست بزنند — ولی با یک قانونِ دقیق‌تر: **هر تعداد قرضِ خواندن هم‌زمان، یا دقیقاً یک قرضِ نوشتنِ انحصاری، هرگز هر دو با هم.** این دقیقاً همان قاعده‌ی هم‌نامی‌ای است که از [۱.۳.۱](../../../phase1-fundamentals/03-borrowing-and-references/01-shared-and-mutable-refs/README.fa.md) می‌شناسی — فقط این‌بار زمانِ اجرا اعمالش می‌کند، بینِ ریسمان‌ها، نه کامپایلر بینِ خط‌های یک تابع.

```rust
use std::sync::RwLock;

fn main() {
    let value = RwLock::new(10);

    let r1 = value.read().unwrap();
    let r2 = value.read().unwrap(); // خواننده‌ی دوم، هم‌زمان، بدونِ مشکل
    println!("two readers at once: {} and {}", *r1, *r2);
```

`.read()` یک `RwLockReadGuard<T>` برمی‌گرداند — دقیقاً مثلِ `MutexGuard` از ۲.۸.۱، یک اشاره‌گرِ هوشمند که `Deref` دارد و با پایانِ دامنه‌اش قرض را آزاد می‌کند. فرقش با `MutexGuard` این است که می‌توانی چندتایش را هم‌زمان زنده نگه داری — `r1` و `r2` بالا دو قرضِ خواندنِ کاملاً معتبر رویِ همان مقدارند، درست همان‌طور که دو `&T` معمولی رویِ یک متغیر می‌توانستند هم‌زمان زنده باشند.

```rust
    let busy = value.try_write();
    println!("try_write while readers are alive: {}", busy.is_err());

    drop(r1);
    drop(r2);

    let mut w = value.write().unwrap();
    *w += 1;
    println!("after write: {w}");
}
```

`.try_write()` همان کاری را می‌کند که `.write()` می‌کند، ولی به‌جایِ صبرکردن، بلافاصله یک `Err` برمی‌گرداند اگر قفل در دسترس نباشد — این‌جا از آن استفاده کردیم تا بدونِ نیاز به ریسمانِ دوم، ثابت کنیم تا وقتی `r1`/`r2` زنده‌اند نوشتن ممکن نیست. `.write()` (بدونِ `try_`) یک `RwLockWriteGuard<T>` می‌دهد که هم `Deref` دارد هم `DerefMut` — چون این‌بار انحصاری است، اجازه‌ی نوشتن هم دارد.

```sh
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 01-rwlock-concurrent-reads
```

```text
two readers at once: 10 and 10
try_write while readers are alive: true
after write: 11
```

```senpai-visual
{"kind":"concurrency","labels":["خواننده‌ی A: read()","خواننده‌ی B هم: read()","هر دو هم‌زمان می‌خوانند","نویسنده: write() و باید صبر کند","هر دو گاردِ خواندن آزاد می‌شوند","نویسنده حالا تنها ادامه می‌دهد"]}
```

### چرا این فرق واقعاً روی توانِ پردازش اثر می‌گذارد: یک کشِ خواندن‌محور

مثالِ کلاسیک — و همانی که این تفاوت را واقعاً به نفعت تمام می‌کند — دقیقاً همان چیزی است که در «چرا اهمیت دارد» توصیف شد: یک کش یا تنظیماتِ مشترک که خواندنش زیاد است و نوشتنش کم. چند ریسمانِ خواننده و یک ریسمانِ نویسنده را واقعی کن:

```rust
use std::sync::{Arc, RwLock};
use std::thread;

fn main() {
    let cache = Arc::new(RwLock::new(vec![
        String::from("Frieren"),
        String::from("Bocchi the Rock!"),
    ]));

    let before: Vec<_> = (0..3)
        .map(|id| {
            let cache = Arc::clone(&cache);
            thread::spawn(move || (id, cache.read().unwrap().len()))
        })
        .collect();
```

همان `Arc` که ۲.۸.۱ به‌ات یاد داد — چون `RwLock<T>` هم مثلِ `Mutex<T>` دقیقاً یک مالک دارد، و هر ریسمان به کپیِ خودش از آن مالکیت نیاز دارد.

```rust
    for handle in before {
        let (id, count) = handle.join().unwrap();
        println!("reader {id} saw {count} cached titles");
    }
```

سه خواننده تمام شدند. حالا نویسنده:

```rust
    let writer_cache = Arc::clone(&cache);
    thread::spawn(move || {
        writer_cache
            .write()
            .unwrap()
            .push(String::from("Made in Abyss"));
    })
    .join()
    .unwrap();

    let after = cache.read().unwrap().len();
    println!("after the writer, the cache holds {after} titles");
}
```

```sh
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 02-rwlock-cache-readers-and-writer
```

```text
reader 0 saw 2 cached titles
reader 1 saw 2 cached titles
reader 2 saw 2 cached titles
after the writer, the cache holds 3 titles
```

اگر این را با `Mutex<Vec<String>>` می‌نوشتی، کد تقریباً یکسان می‌ماند — همه‌جا `.lock()` به‌جایِ `.read()`/`.write()` — ولی معنایش عوض می‌شد: سه خواننده‌ی بالا، حتی با اینکه هیچ‌کدام قرار نبود چیزی بنویسد، مجبور بودند صف بکشند و یکی‌یکی قفل را بگیرند و رها کنند، دقیقاً به‌همان‌اندازه که اگر واقعاً می‌خواستند بنویسند. با `RwLock`، هر سه‌شان می‌توانند هم‌زمان `.read()` را نگه دارند؛ فقط وقتی نویسنده می‌رسد و `.write()` می‌خواهد، همه چیز واقعاً صف می‌کشد. هرچه خواندن نسبت به نوشتن غالب‌تر باشد — دقیقاً حالتِ یک کش یا تنظیماتِ برنامه — این فرق بیشتر به‌ات برمی‌گردد.

### وقتی `std::sync` سمافور ندارد: خودت بسازش

سؤالِ بعدی شکلِ دیگری دارد: نه «چه کسی می‌تواند بنویسد»، بلکه «چند ریسمان هم‌زمان اجازه دارند به یک منبعِ محدود دست بزنند». مثلاً یک استخرِ اتصال با ظرفیتِ ثابت — می‌خواهی حداکثر ۲ ریسمان هم‌زمان از آن استفاده کنند، نه بیشتر، ولی هر تعداد ریسمان می‌توانند صف بکشند تا نوبت‌شان برسد. ابزارِ کلاسیکِ این مسئله «سمافورِ شمارشی (counting semaphore)» نام دارد.

**`std::sync::Semaphore` وجود ندارد** — این نوع در کتابخانه‌ی استانداردِ Rust نیست، و این عمداً است، نه یک کمبود که فراموش کرده باشی چک کنی. خبرِ خوب این است که با دو ابزاری که از ۲.۸.۱ داری — `Mutex` — به‌علاوه‌ی یک ابزارِ تازه به‌اسمِ `Condvar` («condition variable»، متغیرِ شرط)، می‌شود یک سمافورِ کوچک و درست ساخت.

```rust
use std::sync::{Condvar, Mutex};

struct Semaphore {
    available: Mutex<usize>,
    changed: Condvar,
}
```

ایده ساده است: `available` تعدادِ permitِ آزاد را نگه می‌دارد، پشتِ یک `Mutex` معمولی. `Condvar` کاری می‌کند که `Mutex` به‌تنهایی نمی‌تواند: می‌گذارد یک ریسمان *صبر کند* تا شرطی («حداقل یک permit آزاد است») درست شود، بدونِ اینکه در آن حین قفل را برای بقیه اشغال نگه دارد — و می‌گذارد ریسمانِ دیگری که آن شرط را درست کرده، صریحاً بیدارش کند.

```rust
struct Permit<'a> {
    semaphore: &'a Semaphore,
}

impl Drop for Permit<'_> {
    fn drop(&mut self) {
        let mut available = self.semaphore.available.lock().unwrap();
        *available += 1;
        self.semaphore.changed.notify_one();
    }
}
```

قبل از اینکه `acquire` را بنویسیم، همین گاردِ برگشتی را ساختیم. دلیلش همانی است که ۲.۸.۱ رویِ `MutexGuard` نشانت داد: اگر `acquire`/`release` را دو تابعِ جدا و صداکردنی‌-با-دست بنویسی، هر مسیرِ برگشتِ زودهنگام یا پنیک که `release()` را رد کند، آن permit را برایِ همیشه گم می‌کند — و هر ریسمانِ دیگری که منتظرش است، تا ابد صبر می‌کند. با پیچیدنِ permit در یک نوعی که [Drop و RAII](../../../phase1-fundamentals/02-ownership-and-memory/05-drop-and-raii/README.fa.md) دارد، آزادسازی خودکار و تضمینی می‌شود — دقیقاً همان معامله‌ای که `MutexGuard` قبلاً بهت داد.

```rust
impl Semaphore {
    fn new(permits: usize) -> Self {
        Semaphore {
            available: Mutex::new(permits),
            changed: Condvar::new(),
        }
    }
}
```

و حالا خودِ `acquire`:

```rust
impl Semaphore {
    fn acquire(&self) -> Permit<'_> {
        let mut available = self.available.lock().unwrap();
        available = self
            .changed
            .wait_while(available, |count| *count == 0)
            .unwrap();
        *available -= 1;
        Permit { semaphore: self }
    }
}
```

`.wait_while(guard, condition)` دقیقاً همان چیزی است که یک سمافور نیاز دارد: قفل را قرض می‌گیرد، و تا وقتی `condition` روی محتوایش `true` می‌دهد (این‌جا: «صفر permit آزاد است»)، قفل را رها می‌کند و می‌خوابد؛ هر بار که یک `notify_one()`/`notify_all()` بیدارش کند، دوباره شرط را چک می‌کند — نه اینکه کورکورانه فرض کند حالا حتماً درست است. (این نکته مهم است: یک `.wait()` تنها و یک `if` به‌جایِ `while` می‌توانست یک **بیداریِ خیالی [spurious wakeup]** را قبول کند و اشتباهاً یک permit صفر را کم کند؛ `wait_while` این حلقه را برایت درست می‌نویسد.) وقتی شرط دیگر درست نباشد، `wait_while` گاردِ `MutexGuard` را پس می‌دهد، همان جوری که یک `.lock()` معمولی می‌داد.

```senpai-visual
{"kind":"queue","labels":["acquire() — شرط نادرست، می‌خوابد","Permitِ یک ریسمانِ دیگر drop می‌شود","notify_one() خواب‌رفته را بیدار می‌کند","شرط دوباره چک می‌شود — حالا درست است","permit گرفته می‌شود، acquire() برمی‌گردد"]}
```

```rust
let sem = Semaphore::new(2);
{
    let _a = sem.acquire();
    let _b = sem.acquire();
    println!(
        "acquired 2 permits, available: {}",
        *sem.available.lock().unwrap()
    );
} // هر دو permit همین‌جا آزاد می‌شوند، وقتی `_b` و بعد `_a` دراپ می‌شوند

println!(
    "after scope ends, available: {}",
    *sem.available.lock().unwrap()
);
```

```sh
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 03-semaphore-permits-and-raii-release
```

```text
acquired 2 permits, available: 0
after scope ends, available: 2
```

هیچ‌جا صریحاً `.release()` صدا نزدیم — بستنِ بلوک هردو `Permit` را دراپ کرد و خودشان permit را پس دادند، دقیقاً همان اتفاقی که با دو `MutexGuard` در پایانِ یک بلوک می‌افتاد.

حالا کاربردِ واقعی: محدودکردنِ چند ریسمان که هم‌زمان به یک منبعِ محدود دست می‌زنند — مثلِ استخرِ اتصالِ بالا. کمی زودتر از موعد سراغِ `AtomicUsize` می‌رود — بخشِ «اتمیک‌ها» پایین‌تر جایی است که این نوع واقعاً توضیح داده می‌شود — ولی این‌جا فقط ابزارِ اثبات است، نه نکته‌ی اصلیِ مثال، پس یک پیش‌نمایشِ یک‌خطی کافی است: یک عددِ صحیحِ کوچک که چند ریسمان می‌توانند بدونِ هیچ `Mutex`ی امن بهش اضافه کنند یا ازش کم کنند.

```rust
let semaphore = Arc::new(Semaphore::new(2));
let in_flight = Arc::new(AtomicUsize::new(0));

let handles: Vec<_> = (0..4)
    .map(|id| {
        let semaphore = Arc::clone(&semaphore);
        let in_flight = Arc::clone(&in_flight);
        thread::spawn(move || {
            let _permit = semaphore.acquire();
```

تا این‌جا `acquire()` یا فوراً برگشته یا صبر کرده تا permitی آزاد شود. حالا که permit دستِ این ریسمان است:

```rust
            let now = in_flight.fetch_add(1, Ordering::SeqCst) + 1;
            assert!(now <= 2, "more than 2 workers held the resource at once");
            thread::sleep(Duration::from_millis(20));
            in_flight.fetch_sub(1, Ordering::SeqCst);
            id
        })
    })
    .collect();
```

چهار ریسمان، ولی فقط ۲ permit — پس همیشه دست‌کم دو تا از آن‌ها منتظرِ `acquire()` می‌مانند تا یکی از اولی‌ها permitش را (از طریق دراپ‌شدنِ `_permit`) پس بدهد. `in_flight` و آن `assert!` صرفاً برایِ اثباتِ همین قاعده‌اند؛ چیزی که در «دست‌به‌کد» می‌بینی چاپِ ترتیبِ join است، نه یک عددِ رقابتی — چون آن عدد از اجرایی به اجرایِ دیگر می‌توانست فرق کند، ولی نتیجه‌ی این `assert!` هرگز.

```sh
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 04-semaphore-bounds-worker-threads
```

```text
worker 0 acquired and released the resource
worker 1 acquired and released the resource
worker 2 acquired and released the resource
worker 3 acquired and released the resource
never more than 2 workers held the resource at the same time
```

خطِ آخر فقط وقتی چاپ می‌شود که هیچ `assert!`ی پنیک نکرده باشد — یعنی سمافور واقعاً هیچ‌وقت گذاشت بیش از ۲ ریسمان هم‌زمان permit داشته باشند.

> این کدِ بالا کاملاً همزمان و روی ریسمان‌های واقعیِ سیستم‌عامل کار می‌کند. اگر همین ایده را در کدِ **async** بنویسی — چیزی که با `async fn` نوشته می‌شود — به‌جای این سمافورِ دستی سراغِ `tokio::sync::Semaphore` می‌روی؛ [۲.۸.۶](../06-tokio-basics/README.fa.md) رانتایمِ واقعیِ async را معرفی می‌کند. تا آن‌جا، همین نسخه‌ی مبتنی‌بر `Condvar` ابزارِ درستت است.

### `OnceLock`: مقداردهیِ یک‌باره، حتی زیرِ مسابقه

مسئله‌ی بعدی این است: یک مقدار داری که ساختنش گران است — پارس‌کردنِ یک فایلِ تنظیمات، ساختنِ یک جدولِ جست‌وجو — و چند ریسمان باید بخوانندش. می‌خواهی دقیقاً یک‌بار ساخته شود، مهم نیست چند ریسمان هم‌زمان اولین‌بار بخواهندش.

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;
use std::thread;

static INIT_COUNT: AtomicUsize = AtomicUsize::new(0);
static CONFIG: OnceLock<String> = OnceLock::new();

fn config() -> &'static str {
    CONFIG.get_or_init(|| {
        INIT_COUNT.fetch_add(1, Ordering::SeqCst);
        String::from("max_connections=64")
    })
}
```

`OnceLock<T>::new()` یک جایِ خالی می‌سازد. `.get_or_init(closure)` دقیقاً یک قانون دارد: اگر جا هنوز خالی است، `closure` را اجرا می‌کند و نتیجه را همان‌جا نگه می‌دارد؛ اگر پر است، `closure` را اصلاً صدا نمی‌زند و فقط مقدارِ موجود را برمی‌گرداند. وقتی چند ریسمان هم‌زمان `.get_or_init()` را صدا بزنند، `OnceLock` تضمین می‌کند فقط یکی‌شان واقعاً `closure` را اجرا می‌کند — بقیه صبر می‌کنند تا آن یکی تمام شود، بعد همان نتیجه را می‌گیرند. `INIT_COUNT` این‌جا فقط برایِ اثباتِ همین قول است، نه بخشی از الگو.

```rust
fn main() {
    let handles: Vec<_> = (0..8).map(|_| thread::spawn(config)).collect();
    for handle in handles {
        handle.join().unwrap();
    }
    println!("config: {}", config());
    println!(
        "init closure ran {} time(s)",
        INIT_COUNT.load(Ordering::SeqCst)
    );
}
```

```sh
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 05-oncelock-init-once
```

```text
config: max_connections=64
init closure ran 1 time(s)
```

هشت ریسمان هم‌زمان `config()` را صدا زدند، و بازهم کلوژرِ ساختن دقیقاً یک‌بار اجرا شد. این کدِ بالا را چندبار اجرا کن — عددِ آخر همیشه ۱ می‌ماند، مهم نیست ترتیبِ واقعیِ اجرا شدنِ آن ۸ ریسمان چه بود.

```senpai-visual
{"kind":"concept","labels":["۸ ریسمان config() را صدا می‌زنند","اولینِ آن‌ها برنده‌ی مسابقه می‌شود","کلوژرِ مقداردهی را همان یکی اجرا می‌کند","بقیه فقط کمی صبر می‌کنند","OnceLock نتیجه را نگه می‌دارد","همه‌ی ریسمان‌ها همان مقدار را می‌گیرند"]}
```

### `LazyLock`: همان قول، به شکلِ یک `static`

[۱.۱.۱](../../../phase1-fundamentals/01-foundations/01-variables-mutability-shadowing/README.fa.md) به‌ات قول داد که وقتی به حالتِ اشتراکی بینِ ریسمان‌ها برسی، `static` دوباره پیدایش می‌شود. `LazyLock<T>` دقیقاً همان لحظه است: یک `static` که مقدارش را فقط در اولین دسترسی می‌سازد، هر جایِ برنامه که آن دسترسی اتفاق بیفتد.

```rust
use std::sync::LazyLock;
use std::thread;

static SQUARES: LazyLock<Vec<u32>> = LazyLock::new(|| (0..10).map(|n| n * n).collect());

fn main() {
    let handles: Vec<_> = (0..4)
        .map(|id| thread::spawn(move || (id, SQUARES[id])))
        .collect();
    for handle in handles {
        let (id, value) = handle.join().unwrap();
        println!("SQUARES[{id}] = {value}");
    }
    println!("table has {} entries", SQUARES.len());
}
```

```sh
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 06-lazylock-static-lookup-table
```

```text
SQUARES[0] = 0
SQUARES[1] = 1
SQUARES[2] = 4
SQUARES[3] = 9
table has 10 entries
```

`SQUARES[id]` و `SQUARES.len()` بدونِ هیچ `.get()` یا `*` صریحی کار می‌کنند، چون `LazyLock<T>` یک `Deref<Target = T>` دارد — همان تبدیلِ ضمنیِ ارجاعی که از [Deref و AsRef](../../04-lifetimes-and-conversion/03-deref-asref-borrow/README.fa.md) می‌شناسی. فرقِ عملیِ `OnceLock` و `LazyLock` همین است: با `OnceLock` هر جایی که مقدار را می‌خواهی، خودت صریحاً `.get_or_init(closure)` را صدا می‌زنی؛ با `LazyLock` دستورِ ساختن یک‌بار، در تعریفِ خودِ `static`، نوشته می‌شود، و از آن به بعد مثلِ یک `static` معمولی به‌کارش می‌بری — سازوکارِ زیرش (چه کسی اول رسید، بقیه چه‌کار کردند) دقیقاً همانی است که `OnceLock` دارد.

### اتمیک‌ها: یک مقدارِ کوچک، بدونِ هیچ قفلی

بعضی وقت‌ها چیزی که بینِ ریسمان‌ها مشترک است آن‌قدر کوچک است — یک شمارنده، یک پرچمِ درست/غلط — که گرفتنِ یک `Mutex` کاملاً زیاده‌روی است. `std::sync::atomic` نوع‌هایی مثلِ `AtomicUsize`، `AtomicBool` و امثالش دارد: هر عملیاتِ رویِ یکی از این‌ها (خواندن، نوشتن، جمع‌زدن) در یک دستورِ سخت‌افزاریِ تک و غیرِقابلِ‌تقسیم اتفاق می‌افتد — نه یک قفل که «بگیر، بخوان، بنویس، رها کن» را انجام بدهد، بلکه خودِ عملیات، سرِ خودش، امن است.

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

fn main() {
    let counter = Arc::new(AtomicUsize::new(0));
```

چهار ریسمان، هرکدام هزار بار `fetch_add` می‌زنند:

```rust
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let counter = Arc::clone(&counter);
            thread::spawn(move || {
                for _ in 0..1000 {
                    counter.fetch_add(1, Ordering::SeqCst);
                }
            })
        })
        .collect();
```

همه را join کن و جمعِ نهایی را بخوان:

```rust
    for handle in handles {
        handle.join().unwrap();
    }

    println!("final count: {}", counter.load(Ordering::SeqCst));
}
```

```sh
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 07-atomics-lock-free-counter
```

```text
final count: 4000
```

چهار ریسمان، هرکدام ۱۰۰۰ بار `fetch_add(1, ...)` — و جوابِ نهایی همیشه دقیقاً ۴۰۰۰ است، هر بار که اجرایش کنی، مهم نیست آن ۴۰۰۰ جمع‌زدن چه ترتیبِ واقعی‌ای بینِ چهار ریسمان داشتند. `fetch_add` مقدارِ *پیش از* جمع را برمی‌گرداند؛ اگر بخواهی مقدارِ *بعدِ* جمع را، خودت یک واحد بهش اضافه می‌کنی — دقیقاً همان کاری که در بخشِ تمرین با `HitCounter::hit` می‌خواهی.

سه متدِ پایه همین‌جا کاملِ می‌شوند: `.load(ordering)` مقدارِ فعلی را می‌خواند (همان‌طور که بالا در `counter.load(Ordering::SeqCst)` دیدی)، `.store(value, ordering)` بی‌قیدوشرط مقدارِ تازه می‌نویسد (بدونِ خواندنِ قبلی، بدونِ برگرداندنِ چیزی)، و `.fetch_add(amount, ordering)` هر دو کار را با هم، در یک عملیاتِ تکِ اتمیک، انجام می‌دهد.

هر متدِ رویِ یک نوعِ اتمیک یک `Ordering` می‌گیرد — این‌جا همه‌جا `Ordering::SeqCst` («sequentially consistent») نوشتیم. قانونِ این درس ساده است: **`SeqCst` همیشه انتخابِ امن و آسان‌برایِ‌استدلال‌کردن است.** ترتیب‌هایِ ضعیف‌تر (`Relaxed`، `Acquire`، `Release`) وجود دارند، برایِ کارهایِ کاراییِ پیشرفته که این دوره واردشان نمی‌شود — مدلِ کاملِ حافظه‌ی اتمیکِ Rust خودش موضوعی برایِ یک درسِ کامل است، نه یک زیربخش. تا وقتی دلیلِ مشخصی برایِ چیزِ دیگری نداری، `SeqCst` بنویس.

الگویِ دومِ رایج، وقتی به‌جایِ جمع‌زدن باید «فقط اگر هنوز فلان مقدار است، عوضش کن» بپرسی، `compare_exchange` است:

```rust
use std::sync::atomic::{AtomicBool, Ordering};

fn main() {
    let started = AtomicBool::new(false);
    let first = started.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst);
    let second = started.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst);
    println!("first attempt:  {first:?}");
    println!("second attempt: {second:?}");
}
```

```text
first attempt:  Ok(false)
second attempt: Err(true)
```

`compare_exchange(current, new, success_order, failure_order)` می‌پرسد: «اگر مقدارِ فعلی دقیقاً `current` است، بگذارش `new` — و همین یک حرکت را با هم انجام بده، بدونِ اینکه ریسمانِ دیگری بینِ چک‌کردن و عوض‌کردن جا خوش کند.» اگر واقعاً `current` بود، عوضش می‌کند و `Ok(مقدارِ_قبلی)` می‌دهد. اگر نبود (یکی دیگر زودتر رسیده)، هیچ‌چیز را عوض نمی‌کند و `Err(مقدارِ_واقعیِ_فعلی)` می‌دهد. بالا: اولین `compare_exchange` مقدار را از `false` به `true` می‌برد و `Ok(false)` می‌گیرد؛ دومین، چون مقدار دیگر `true` است نه `false`، چیزی عوض نمی‌کند و `Err(true)` می‌گیرد. این دقیقاً یک «پرچمِ یک‌باره» است: هر تعداد ریسمان می‌توانند این را صدا بزنند، ولی فقط یکی‌شان `Ok` می‌گیرد.

```rust
let claimed = Arc::new(AtomicBool::new(false));
let winners = Arc::new(AtomicUsize::new(0));
let handles: Vec<_> = (0..6)
    .map(|_| {
        let claimed = Arc::clone(&claimed);
        let winners = Arc::clone(&winners);
        thread::spawn(move || {
```

هر ریسمان دقیقاً همان `compare_exchange` بالا را امتحان می‌کند:

```rust
            let won = claimed
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok();
            if won {
                winners.fetch_add(1, Ordering::SeqCst);
            }
        })
    })
    .collect();
```

```sh
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 08-atomics-compare-exchange-flag
```

```text
threads that won the race to initialize: 1
```

شش ریسمان واقعاً روی همان یک `AtomicBool` مسابقه می‌دهند، و بازهم عددِ برنده‌ها همیشه دقیقاً ۱ است — همان تضمینی که `compare_exchange` می‌دهد، این‌بار با ریسمان‌هایِ واقعی به‌جایِ دو صدازدنِ پشتِ‌سرِهم. این همان مسئله‌ای است که `OnceLock` هم حل می‌کند؛ فرق این‌جا این است که خودت مستقیم با پرچم کار می‌کنی، نه با یک نوعی که این حلقه را برایت پنهان کرده.

```senpai-visual
{"kind":"concurrency","labels":["مقدارِ فعلی را load کن","مقدارِ منهایِ ۱ را حساب کن","compare_exchange(current, new)","یکی دیگر عوضش کرده بود: دوباره","کسی عوضش نکرده بود: موفق","مقدار عوض شد، منبع گرفته شد"]}
```

### راهنمای انتخاب: کدام‌یک، کِی

- **`Mutex`** — دسترسیِ انحصاریِ عمومی؛ وقتی خواندن و نوشتن هردو زیادند یا فرقی باهم ندارند.
- **`RwLock`** — وقتی خواندن به‌مراتب بیشتر از نوشتن است و می‌خواهی خواننده‌ها هم‌زمان پیش بروند.
- **سمافور (`Mutex` + `Condvar` دستی)** — وقتی می‌خواهی تعدادِ ریسمان‌هایی که هم‌زمان به یک منبعِ محدود دست می‌زنند را کران‌دار کنی.
- **`OnceLock` / `LazyLock`** — راه‌اندازیِ یک‌بارِ امن؛ یک مقدار که ساختنش گران است ولی همه فقط می‌خوانندش.
- **اتمیک‌ها** — یک مقدارِ کوچکِ تنها (شمارنده، پرچم) که گرفتنِ یک قفلِ کامل برایش زیاده‌روی است.

---

## دست‌به‌کد

```sh
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 01-rwlock-concurrent-reads
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 02-rwlock-cache-readers-and-writer
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 03-semaphore-permits-and-raii-release
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 04-semaphore-bounds-worker-threads
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 05-oncelock-init-once
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 06-lazylock-static-lookup-table
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 07-atomics-lock-free-counter
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 08-atomics-compare-exchange-flag
```

بعد چهارتای خراب:

```sh
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 09-rwlock-move-without-arc --features broken
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 10-write-through-read-guard --features broken
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 11-oncelock-set-twice-panics --features broken
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 12-atomics-no-direct-equality --features broken
```

بعد این‌ها را امتحان کن:

۱. در `04-semaphore-bounds-worker-threads`، ظرفیتِ سمافور را از ۲ به ۱ عوض کن و تعدادِ ریسمان‌ها را به ۶ برسان. برنامه هنوز درست تمام می‌شود؟ چرا زمانِ اجرایش کمی بیشتر می‌شود؟
۲. در `07-atomics-lock-free-counter`، `Ordering::SeqCst` را همه‌جا با `Ordering::Relaxed` عوض کن. عددِ نهایی هنوز ۴۰۰۰ می‌ماند؟ (این کار عمداً از چیزی که این درس تدریس می‌کند رد می‌شود — فقط برایِ دیدنِ اینکه کامپایلر مانعت نمی‌شود.)
۳. در `05-oncelock-init-once`، `INIT_COUNT.fetch_add` را قبل از خطِ `String::from(...)` بردار و بعدِ آن بگذار. جوابِ نهایی عوض می‌شود؟ باید بشود؟

---

## خطاهایی که خواهی دید

### `E0382` — `RwLock` هم به همان `Arc` نیاز دارد که `Mutex` داشت

```text
error[E0382]: use of moved value: `cache`
  --> phase2-intermediate\08-concurrency\02-rwlock-semaphore-oncelock-atomics\examples\09-rwlock-move-without-arc.rs:14:23
   |
12 |     let cache = RwLock::new(String::from("v1"));
   |         ----- move occurs because `cache` has type `std::sync::RwLock<String>`, which does not implement the `Copy` trait
13 |     for _ in 0..3 {
   |     ------------- inside of this loop
14 |         thread::spawn(move || {
   |                       ^^^^^^^ value moved into closure here, in previous iteration of loop
15 |             let guard = cache.read().unwrap();
   |                         ----- use occurs due to use in closure

For more information about this error, try `rustc --explain E0382`.
error: could not compile `p2-08-02-rwlock-semaphore-oncelock-atomics` (example "09-rwlock-move-without-arc") due to 1 previous error
```

**کامپایلر به چه اعتراض دارد:** `move` هر بار که حلقه اجرا شود، `cache` را کاملاً به کلوژرِ همان تکرار می‌دهد. تکرارِ اول این کار را کرد و مالکیتِ `cache` را گرفت؛ تا تکرارِ دوم برسد، `cache` دیگر چیزی برایِ گرفتن ندارد — دقیقاً همان چیزی که پیام می‌گوید: «value moved into closure here, in previous iteration of loop».

**راه‌حل:** دقیقاً همان کاری که ۲.۸.۱ با `Mutex` یادت داد — یک `Arc` دورش بگیر و برایِ هر ریسمان یک کپیِ آن `Arc` را `move` کن، نه خودِ `RwLock` را:

```rust
let cache = Arc::new(RwLock::new(String::from("v1")));
for _ in 0..3 {
    let cache = Arc::clone(&cache);
    thread::spawn(move || {
        let guard = cache.read().unwrap();
        println!("{guard}");
    });
}
```

**چرا این راه‌حل است:** `Arc::clone` مقدارِ رویِ هیپ را کپی نمی‌کند، فقط شمارنده‌ی مالکان را یکی بالا می‌برد و یک دستگیره‌ی تازه به همان `RwLock` می‌دهد. حالا هر تکرار `move` می‌کند، ولی چیزی که `move` می‌شود یک `Arc` تازه است، نه خودِ `cache`ی اصلی — پس مالکیتِ اصلی هیچ‌وقت تمام نمی‌شود.

### `E0594` — نمی‌شود از پشتِ یک گاردِ خواندن نوشت

```text
error[E0594]: cannot assign to data in dereference of `std::sync::RwLockReadGuard<'_, i32>`
  --> phase2-intermediate\08-concurrency\02-rwlock-semaphore-oncelock-atomics\examples\10-write-through-read-guard.rs:12:5
   |
12 |     *guard += 1;
   |     ^^^^^^^^^^^ cannot assign
   |
   = help: trait `DerefMut` is required to modify through a dereference, but it is not implemented for `std::sync::RwLockReadGuard<'_, i32>`

For more information about this error, try `rustc --explain E0594`.
error: could not compile `p2-08-02-rwlock-semaphore-oncelock-atomics` (example "10-write-through-read-guard") due to 1 previous error
```

**کامپایلر به چه اعتراض دارد:** `counter.read()` یک `RwLockReadGuard<i32>` می‌دهد — گاردی که فقط `Deref` دارد، نه `DerefMut`، دقیقاً به همان دلیلی که `Ref<T>`ِ `RefCell` (از [۲.۶.۵](../../06-smart-pointers/05-refcell-and-interior-mutability/README.fa.md)) فقط `Deref` داشت: این گارد قولِ «فقط خواندن» را داده، و کامپایلر آن قول را نگه می‌دارد.

**راه‌حل:** برایِ نوشتن، `.write()` بخواه، نه `.read()`:

```rust
let counter = RwLock::new(0);
let mut guard = counter.write().unwrap();
*guard += 1;
println!("{guard}");
```

**چرا این راه‌حل است:** `.write()` یک `RwLockWriteGuard<i32>` می‌دهد که هم `Deref` دارد هم `DerefMut` — چون قولش «فقط من، انحصاری» است، نه «هرکسی که بخواهد نگاه کند». اگر همان لحظه یک قرضِ خواندنِ دیگر هم زنده بود، `.write()` تا آزادشدنش صبر می‌کرد؛ ولی وقتی واقعاً می‌خواهی بنویسی، این ابزارِ درست است، نه تلاش برایِ نوشتن از پشتِ یک گاردِ خواندن.

### پنیکِ زمانِ اجرا — `.set()`ِ دومِ `OnceLock` همیشه `Err` است

```text
thread 'main' (22608) panicked at phase2-intermediate\08-concurrency\02-rwlock-semaphore-oncelock-atomics\examples\11-oncelock-set-twice-panics.rs:12:40:
called `Result::unwrap()` on an `Err` value: "second"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**کامپایلر به چه اعتراض دارد:** این حتی خطایِ کامپایلر نیست — برنامه ساخته شد و اجرا شد. `OnceLock::set(value)` نوعش `Result<(), T>` است: اگر جا خالی بود، پرش می‌کند و `Ok(())` می‌دهد؛ اگر از قبل پر بود (دقیقاً همین‌جا، چون خطِ قبل همین را پر کرده)، هیچ‌چیز عوض نمی‌کند و `Err(همان_مقداری_که_دادی)` برمی‌گرداند. `.unwrap()` رویِ آن `Err("second")` پنیک می‌کند.

**راه‌حل:** اگر مقداردهیِ یک‌بار همان چیزی است که می‌خواهی، `.get_or_init()` بخواه، نه `.set()`:

```rust
let config: OnceLock<String> = OnceLock::new();
config.get_or_init(|| String::from("first"));
config.get_or_init(|| String::from("second")); // نادیده گرفته می‌شود، پنیک نمی‌گیرد
println!("{:?}", config.get());
```

**چرا این راه‌حل است:** `.set()` برایِ جایی است که خودت مطمئنی این اولین‌بار است و می‌خواهی شکستش را ببینی (مثلاً موقعِ راه‌اندازیِ برنامه). `.get_or_init()` برایِ حالتِ عادی‌تر است — «هر جا برایِ اولین‌بار به این نیاز پیدا کنی، بسازش؛ هر جایِ دیگر، همان قبلی را بده» — و هیچ‌وقت با یک صدازدنِ دومِ بی‌ضرر پنیک نمی‌گیرد.

### `E0369` — نمی‌شود دو مقدارِ اتمیک را مستقیم مقایسه کرد

```text
error[E0369]: binary operation `==` cannot be applied to type `Atomic<usize>`
   --> phase2-intermediate\08-concurrency\02-rwlock-semaphore-oncelock-atomics\examples\12-atomics-no-direct-equality.rs:13:10
    |
 13 |     if a == b {
    |        - ^^ - Atomic<usize>
    |        |
    |        Atomic<usize>
    |
note: `Atomic<usize>` does not implement `PartialEq`
   --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\sync\atomic.rs:366:1
    |
366 | pub struct Atomic<T: AtomicPrimitive> {
    | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ `Atomic<usize>` is defined in another crate

For more information about this error, try `rustc --explain E0369`.
error: could not compile `p2-08-02-rwlock-semaphore-oncelock-atomics` (example "12-atomics-no-direct-equality") due to 1 previous error
```

(روی این تولکین، پیام `AtomicUsize` را با نامِ واقعیِ زیرینش نشان می‌دهد — `Atomic<usize>` — چون کتابخانه‌ی استاندارد `AtomicUsize` را الان به‌عنوانِ یک اسمِ مستعار برایِ همین نوعِ جنریک تعریف می‌کند؛ خودِ `AtomicUsize` که در کد نوشتی همین است، فقط کامپایلر با تعریفِ واقعی‌اش جوابت را می‌دهد.)

**کامپایلر به چه اعتراض دارد:** نوع‌هایِ اتمیک عمداً `PartialEq` پیاده نمی‌کنند. اگر پیاده می‌کردند، `a == b` بی‌سروصدا دو تا `.load()`ِ جدا انجام می‌داد — یکی برایِ `a`، یکی برایِ `b` — بینِ آن دو هر ریسمانِ دیگری می‌توانست هرکدام را عوض کند، و مقایسه‌ات چیزی را با هم می‌سنجید که هرگز واقعاً هم‌زمان نبودند.

**راه‌حل:** خودت صریح `.load()` کن، بعد عددهایِ معمولی را مقایسه کن:

```rust
let a = AtomicUsize::new(1);
let b = AtomicUsize::new(1);
if a.load(Ordering::SeqCst) == b.load(Ordering::SeqCst) {
    println!("equal");
}
```

**چرا این راه‌حل است:** حالا واضح است که دو خواندنِ جداگانه در دو لحظه‌ی جداگانه انجام شده — دقیقاً همان چیزی که همیشه اتفاق می‌افتاد، فقط این‌بار کامپایلر مجبورت کرده ببینیش، به‌جایِ اینکه `==` آن را پشتِ یک نحوِ آشنا پنهان کند.

---

## تمرین

### گرم‌کردن

<details>
<summary>این چه چاپ می‌کند؟</summary>

```rust
use std::sync::RwLock;

let v = RwLock::new(5);
let a = v.read().unwrap();
let b = v.read().unwrap();
println!("{}", *a + *b);
```

</details>

<details>
<summary>پاسخ</summary>

```text
10
```

`RwLock` هر تعداد قرضِ خواندنِ هم‌زمان را اجازه می‌دهد؛ `a` و `b` دو قرضِ کاملاً معتبرِ همان مقدار‌اند، هردو ۵.

</details>

<details>
<summary>این کامپایل می‌شود؟</summary>

```rust
let counter = RwLock::new(0);
let guard = counter.read().unwrap();
*guard += 1;
```

</details>

<details>
<summary>پاسخ</summary>

نه. `.read()` یک `RwLockReadGuard` می‌دهد که فقط `Deref` دارد، نه `DerefMut` — نوشتن از پشتش دقیقاً همان `E0594`ای است که در «خطاهایی که خواهی دید» دیدی. برایِ نوشتن باید `.write()` بخواهی.

</details>

<details>
<summary>بعدِ این بلوک، چند permit در سمافور آزاد است؟</summary>

```rust
let sem = Semaphore::new(2);
{
    let _a = sem.acquire();
    let _b = sem.acquire();
}
```

</details>

<details>
<summary>پاسخ</summary>

```text
2
```

هردو `Permit` با بسته‌شدنِ بلوک دراپ می‌شوند — `_b` اول، بعد `_a` — و `Drop`شان هر دو permit را پس می‌دهد. سمافور به همان ظرفیتِ اولش برمی‌گردد.

</details>

<details>
<summary>پیامِ «computing...» چند بار چاپ می‌شود؟</summary>

```rust
let cache: OnceLock<u32> = OnceLock::new();
cache.get_or_init(|| {
    println!("computing...");
    42
});
cache.get_or_init(|| {
    println!("computing...");
    42
});
println!("{:?}", cache.get());
```

</details>

<details>
<summary>پاسخ</summary>

```text
computing...
Some(42)
```

فقط یک‌بار. صدازدنِ دومِ `.get_or_init()` می‌بیند جا از قبل پر است و اصلاً کلوژرش را اجرا نمی‌کند.

</details>

<details>
<summary>مقدارِ نهایی چیست؟</summary>

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

let counter = AtomicUsize::new(0);
counter.fetch_add(5, Ordering::SeqCst);
counter.fetch_add(3, Ordering::SeqCst);
println!("{}", counter.load(Ordering::SeqCst));
```

</details>

<details>
<summary>پاسخ</summary>

```text
8
```

هر `fetch_add` مقدارِ داده‌شده را به مقدارِ فعلی اضافه می‌کند: ۰ + ۵ = ۵، بعد ۵ + ۳ = ۸.

</details>

### تعمیر

هر چهار مثالِ خراب را درست کن:

۱. `examples/09-rwlock-move-without-arc.rs` را با یک `Arc<RwLock<String>>` درست کن، دقیقاً همان الگویی که ۲.۸.۱ با `Mutex` یادت داد.
۲. `examples/10-write-through-read-guard.rs` را با عوض‌کردنِ `.read()` به `.write()` درست کن.
۳. `examples/11-oncelock-set-twice-panics.rs` را طوری درست کن که پنیک نگیرد — بدونِ حذف‌کردنِ صدازدنِ دوم.
۴. `examples/12-atomics-no-direct-equality.rs` را با `.load()` کردنِ هر دو طرف پیش از `==` درست کن.

### پیاده‌سازی

سه نوع در `src/lib.rs`:

```sh
cargo test -p p2-08-02-rwlock-semaphore-oncelock-atomics
```

`LazyGreeting` یک `OnceLock<String>` را به‌عنوانِ فیلد نگه می‌دارد — نه یک `static`، بلکه یک نمونه‌یِ خودش، برایِ هر مقدارِ `LazyGreeting`. `HitCounter` فقط یک `AtomicUsize` است. `ResourcePool` همان ایده‌یِ محدودکردنِ دسترسیِ سمافور را با یک حلقه‌یِ `compare_exchange` پیاده می‌کند — بدونِ `Condvar`، بدونِ صبرکردن؛ اگر جایی نبود، بلافاصله `false` برمی‌گردد. هر متد را دقیقاً از رویِ کامنتِ مستندسازِ بالایش پیاده کن.

### بساز

یک کشِ کوچکِ خودت طراحی کن — یک جدولِ ترجمه، یک عکسِ لحظه‌ای از جدولِ امتیازها، یک بلوکِ تنظیمات، هرچه دوست داری — پشتِ یک `RwLock<T>`. یک متدِ «بخوان»ی بده که از چند ریسمانِ هم‌زمان صدا زدنی باشد، و یک متدِ «بنویس»ی که مقدار را عوض کند. دست‌کم ۳ ریسمانِ خواننده و ۱ ریسمانِ نویسنده بساز تا مطمئن شوی هم کامپایل می‌شود، هم رفتارش همان چیزی است که انتظار داری.

### چالش (اختیاری)

**بخشِ یک.** به `Semaphore`ای که در «مفهوم» ساختی، یک متدِ `try_acquire(&self) -> Option<Permit<'_>>` اضافه کن که هیچ‌وقت صبر نمی‌کند: اگر permitی آزاد بود، همان کارِ `acquire` را بکن و `Some(permit)` برگردان؛ اگر نبود، بدونِ لمس‌کردنِ چیزی `None` برگردان. (سرنخ: به یک `Mutex::lock()` معمولی نیاز داری، نه `wait_while`.)

**بخشِ دو.** (این یکی جلوتر را نگاه می‌کند.) در این درس، ریسمان‌ها با به‌اشتراک‌گذاشتنِ *دسترسی* به یک مقدار هماهنگ شدند — یک `RwLock`، یک سمافور. [۲.۸.۳](../03-channels-message-passing/README.fa.md) یک مدلِ کاملاً متفاوت را نشان می‌دهد: به‌جایِ اشتراکِ دسترسی، ریسمان‌ها *مقدارهایِ مالک‌دار* را بینِ خودشان می‌فرستند. برایِ یک صفِ کار که چند ریسمانِ کارگر یکی‌یکی وظیفه برمی‌دارند، حدس بزن — سراغِ یک سمافور می‌روی یا سراغِ آن مدلِ تازه؟ دلیلت را برایِ خودت بنویس، بعد در ۲.۸.۳ جوابت را بررسی کن.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| `RwLock<T>` | هر تعداد قرضِ خواندن هم‌زمان، یا دقیقاً یک قرضِ نوشتنِ انحصاری | کشِ خواندن‌محور، تنظیماتِ به‌ندرت‌تغییرکننده |
| `RwLockReadGuard` / `RwLockWriteGuard` | گاردهایِ `.read()`/`.write()` | فقط دومی `DerefMut` دارد |
| `Condvar` | «صبر کن تا شرطی درست شود، بدونِ اشغالِ قفل در این حین» | زیرِ هر ابزارِ صبرکردن‌محور، مثلِ سمافور |
| سمافورِ شمارشی | محدودکردنِ تعدادِ ریسمان‌هایِ هم‌زمانِ روی یک منبع | `std::sync` ندارد؛ از `Mutex`+`Condvar` می‌سازیش |
| `OnceLock<T>` | `.get_or_init()`: کلوژر فقط یک‌بار اجرا می‌شود، حتی زیرِ مسابقه | مقداردهیِ گران، خوانده‌شده از چند ریسمان |
| `LazyLock<T>` | همان `OnceLock`، به‌شکلِ اعلانیِ یک `static` | جدولِ جست‌وجو یا مقدارِ سراسریِ محاسبه‌شونده |
| نوعِ اتمیک (`AtomicUsize`، ...) | خواندن/نوشتن/جمع‌زدن در یک عملیاتِ تک، بدونِ قفل | یک مقدارِ کوچکِ تنها |
| `compare_exchange` | «اگر هنوز فلان است، عوضش کن — همه در یک حرکت» | شمارنده‌هایِ بدونِ‌قفل، پرچمِ یک‌باره |
| `Ordering::SeqCst` | ترتیبِ حافظه‌یِ پیش‌فرضِ امن | همیشه، مگر دلیلِ مشخصی برایِ چیزِ دیگری داشته باشی |

### الان می‌دانی

- `RwLock<T>` هر تعداد قرضِ خواندن را هم‌زمان اجازه می‌دهد، ولی نوشتن همچنان انحصاری است — و این فرق واقعاً روی توانِ پردازشِ یک بارِکاریِ خواندن‌محور اثر می‌گذارد، نه فقط رویِ کاغذ.
- `std::sync::Semaphore` وجود ندارد؛ یک سمافورِ شمارشیِ درست را می‌شود با یک `Mutex<usize>` و یک `Condvar` ساخت، و یک permitِ RAII دقیقاً همان تضمینِ خودکارِ آزادسازی را می‌دهد که `MutexGuard` می‌داد.
- `OnceLock::get_or_init` تضمین می‌کند کلوژرِ ساختن، مهم نیست چند ریسمان هم‌زمان مسابقه بدهند، دقیقاً یک‌بار اجرا می‌شود؛ `LazyLock` همان تضمین را به‌شکلِ یک `static` می‌دهد.
- اتمیک‌ها یک مقدارِ کوچک را بدونِ هیچ قفلی امن می‌کنند؛ `fetch_add` برایِ شمارنده‌هایِ بدونِ‌قفل است، `compare_exchange` برایِ «فقط اگر هنوز فلان است، عوضش کن».
- `Ordering::SeqCst` همیشه انتخابِ امنِ پیش‌فرض است؛ ترتیب‌هایِ ضعیف‌تر برایِ کاراییِ پیشرفته‌اند و این دوره واردشان نمی‌شود.
- پنج ابزار حالا داری — `Mutex`، `RwLock`، سمافور، `OnceLock`/`LazyLock`، اتمیک — و یک قاعده برایِ انتخاب بینشان، نه فقط یک لیست از نام‌ها.

### بعداً کامل‌تر می‌بینی

- **چرا بعضی نوع‌ها اصلاً اجازه‌ی عبور از مرزِ ریسمان‌ها را ندارند، و `Send`/`Sync` دقیقاً چه چیزی را تضمین می‌کنند** — [۲.۸.۴ — `Send` و `Sync`](../04-send-and-sync/README.fa.md)
- **فرستادنِ مقدارهایِ مالک‌دار بینِ ریسمان‌ها، به‌جایِ اشتراکِ دسترسی** — [۲.۸.۳ — کانال‌ها و پیام‌رسانی](../03-channels-message-passing/README.fa.md)
- **کار با همین شکل‌های مسئله وقتی کد دیگر بر پایه‌ی ریسمان نیست بلکه async است** — [ماژولِ ۲.۹ — async در عمل](../../09-async-in-practice/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا `RwLock` می‌تواند چند خواننده‌ی هم‌زمان را اجازه بدهد، ولی `Mutex` هیچ‌وقت؟
- چرا `std::sync::Semaphore` وجود ندارد، و سمافورِ خودت از چه دو تکه ساخته شده؟
- یک permitِ RAII چه چیزی را تضمین می‌کند که یک `release()` دستی تضمین نمی‌داد؟
- فرقِ عملیِ `OnceLock` و `LazyLock` چیست؟
- چرا نمی‌شود دو مقدارِ اتمیک را مستقیم با `==` مقایسه کرد؟
- اگر یک مقدارِ کوچکِ مشترک داری، از کجا می‌فهمی وقتش رسیده که `Mutex` را کنار بگذاری و سراغِ یک اتمیک بروی؟

---

## بیشتر

- [کتابِ Rust — `RwLock<T>`](https://doc.rust-lang.org/std/sync/struct.RwLock.html) — مستنداتِ کامل، شاملِ `try_read`/`try_write`.
- [`std::sync::Condvar`](https://doc.rust-lang.org/std/sync/struct.Condvar.html) — `wait`، `wait_while`، `notify_one`، `notify_all`.
- [`std::sync::OnceLock`](https://doc.rust-lang.org/std/sync/struct.OnceLock.html) و [`std::sync::LazyLock`](https://doc.rust-lang.org/std/sync/struct.LazyLock.html) — مستنداتِ کاملِ هر دو نوع.
- [`std::sync::atomic` — نمایِ کلیِ ماژول](https://doc.rust-lang.org/std/sync/atomic/index.html) — فهرستِ کاملِ نوع‌ها و متدها.
- [The Rustonomicon — atomics](https://doc.rust-lang.org/nomicon/atomics.html) — برایِ وقتی کنجکاو شدی `Relaxed`/`Acquire`/`Release` دقیقاً چه معنایی دارند؛ همان‌جا هم می‌گوید `SeqCst` پیش‌فرضِ امن است وقتی مطمئن نیستی.

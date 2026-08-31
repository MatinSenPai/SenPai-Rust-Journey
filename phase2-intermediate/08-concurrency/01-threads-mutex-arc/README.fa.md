# ۲.۸.۱ — ریسمان‌ها، `Mutex`، `Arc`

## در یک نگاه

بعد از این درس می‌توانی:

- یک ریسمانِ واقعیِ سیستم‌عامل را با `thread::spawn` اجرا کنی، نتیجه‌اش را از راهِ یک `JoinHandle` و `.join()` پس بگیری، و خطایِ کامپایلِ رایجِ اولین تلاش — کلوژری که بدونِ `move` یک متغیرِ محلی را قرض می‌گیرد — را خودت بخوانی و رفع کنی.
- وضعیتِ تغییرپذیر را با `Arc<Mutex<T>>` بینِ چند ریسمان امن به اشتراک بگذاری، و در یک جمله بگویی چرا `Rc`ِ ۲.۶.۳ و `RefCell`ِ ۲.۶.۵ اینجا جایگزین ندارند.
- یک `Err`ِ ناشی از مسموم‌شدنِ قفل را بخوانی، بگویی «مسموم‌شدن» دقیقاً یعنی چه، و تصمیم بگیری کدِ خودت با آن چه باید بکند.

**زمان:** حدود ۶۵ دقیقه · **پیش‌نیاز:**
[۲.۶.۳ — `Rc` و `Arc`](../../06-smart-pointers/03-rc-and-arc/README.fa.md) ·
[۲.۶.۵ — `RefCell`، `Cell` و معامله‌ی پنیکِ زمانِ اجرا](../../06-smart-pointers/05-refcell-and-interior-mutability/README.fa.md)

---

## چرا اهمیت دارد

قاعده‌ی هم‌نامی را از فاز ۱ به یاد داری: هر تعداد قرضِ اشتراکی، یا دقیقاً یک قرضِ تغییرپذیر — هرگز هر دو با هم. تا همین‌جایِ دوره، کامپایلر این قاعده را رویِ کدی اجرا می‌کرد که همیشه رویِ یک ریسمانِ واحد، از بالا به پایین، پیش می‌رفت. امروز این فرض کنار می‌رود: کدی که می‌نویسی می‌تواند رویِ چند ریسمانِ کاملاً مجزا، هم‌زمان، رویِ هسته‌های مختلفِ همان پردازنده اجرا شود — موازی‌سازیِ واقعی، نه شبیه‌سازی‌اش.

۲.۶.۳ با یک وعده تمام شد، نه فقط یک جمع‌بندی: «مکانیزمِ دقیقِ عبور از مرزِ ریسمان‌ها و خودِ `Send` مالِ ماژول ۸ است؛ امروز فقط این واقعیت را لازم داری: `Rc` عمداً تویِ همان یک ریسمان می‌ماند.» ۲.۶.۵ هم دقیقاً همان وعده را با یک آزمایش بست: یک `Rc<RefCell<i32>>` بساز و بگذارش تویِ کلوژرِ یک `std::thread::spawn` — کامپایل نمی‌شود، و کامپایلر صریحاً می‌گوید که `Rc<RefCell<i32>>` را نمی‌شود امن بینِ ریسمان‌ها فرستاد. امروز جایی است که هر دو وعده نقد می‌شود: همان دو تایی که تویِ یک ریسمان قابلِ‌اعتماد بودند، حالا معلوم می‌شود چرا بینِ چند ریسمان دیگر قابلِ‌اعتماد نیستند — و با چه چیزی باید عوضشان کنی.

اگر با پایتون کار کرده باشی، شکلِ کلیِ مسئله را از قبل می‌شناسی. `threading.Thread` تویِ CPython واقعاً یک ریسمانِ سطحِ سیستم‌عامل می‌سازد — نه شبیه‌سازی، ریسمانِ واقعی. اما GIL (قفلِ سراسریِ مفسر) تضمین می‌کند در هر لحظه فقط یکی از آن ریسمان‌ها دارد بایت‌کدِ پایتون را اجرا می‌کند؛ برایِ کدی که گلوگاهش پردازنده است، این یعنی `threading` هیچ موازی‌سازیِ واقعی‌ای نمی‌دهد — برایِ آن باید سراغِ `multiprocessing` بروی، یعنی پردازش‌هایِ کاملاً مجزا، بدونِ هیچ حافظه‌ی مشترکی. کدی که گلوگاهش ورودی/خروجی است، از `threading` واقعاً سود می‌برد، چون I/O در حینِ انتظار GIL را آزاد می‌کند. جایی که این پل می‌شکند همین‌جاست: نبودِ GIL در Rust یعنی هسته‌های پردازنده واقعاً هم‌زمان کدِ Rust را اجرا می‌کنند، بدونِ هیچ قفلِ پنهانی که پشتِ صحنه چیزی را برایت سریال کند — و دقیقاً همین است که کامپایلرِ Rust را مجبور می‌کند اینجا سخت‌گیرتر از همیشه باشد.

---

## مفهوم

### یک ریسمانِ واقعیِ سیستم‌عامل: `thread::spawn`، `JoinHandle`، `.join()`

```rust
use std::thread;

let handle = thread::spawn(|| {
    println!("hello from the spawned thread");
    2 + 2
});
```

```rust
let result = handle.join().unwrap(); // صبر می‌کند تا ریسمان تمام شود
println!("the spawned thread returned: {result}");
```

```text
hello from the spawned thread
the spawned thread returned: 4
```

متدِ `thread::spawn` کلوژرت را تحویلِ سیستم‌عامل می‌دهد، و او دقیقاً رویِ یک **ریسمان (thread)** کاملاً مجزا در سطحِ خودِ سیستم‌عامل اجرایش می‌کند — یک ریسمانِ واقعی، زمان‌بندی‌شده به دستِ سیستم‌عامل، قادر به اجرایِ واقعاً موازی رویِ هسته‌ای دیگر.

خروجیِ `spawn` یک `JoinHandle<T>` است، که آن `T` همان چیزی است که کلوژرت برمی‌گرداند — اینجا `i32`. صدا زدنِ `.join()` ریسمانِ فراخواننده را متوقف می‌کند تا ریسمانِ تولیدشده کارش را تمام کند، و در نهایت یک مقدار از این شکل برمی‌گرداند:

```text
Result<T, Box<dyn Any + Send>>
```

`Ok(T)` اگر کلوژر عادی تمام شود، `Err(...)` اگر **پنیک** کند. برایِ همین تقریباً همه‌جا `.join().unwrap()` می‌بینی: یعنی داری صریحاً می‌گویی «انتظار دارم این ریسمان موفق شود؛ اگر نشد، همین‌جا با سروصدا پنیک کن». کدی که واقعاً باید شکستِ یک ریسمانِ کارگر را تحمل کند، به‌جایِ `unwrap`، باید رویِ همان `Result` یک `match` بزند.

```senpai-visual
{"kind":"concurrency","labels":["main thread","thread::spawn","child thread runs","join() blocks","result returned"]}
```

### چرا کلوژرِ `spawn` باید مالکِ چیزی باشد که استفاده می‌کند

اگر کلوژر بخواهد یک متغیرِ محلی را فقط قرض بگیرد، بدونِ `move`:

```text
error[E0373]: closure may outlive the current function, but it borrows `data`, which is owned by the current function
```

جزئیاتِ کامل در «خطاهایی که خواهی دید»؛ `examples/05-borrow-without-move-broken.rs` دقیقاً همین را می‌سازد. راهِ‌حل، خودِ `move`:

```rust
let data = vec![1, 2, 3];

let handle = thread::spawn(move || {
    println!("the spawned thread owns: {data:?}");
});
handle.join().unwrap();
```

```text
the spawned thread owns: [1, 2, 3]
```

کلمه‌ی کلیدیِ `move` کلوژر را مجبور می‌کند مالکیتِ هرچه به آن ارجاع می‌دهد را کامل بگیرد، نه فقط قرضش کند — و این یک قانون است، نه سلیقه: امضایِ `spawn` کلوژرش را `'static` می‌خواهد، یعنی نباید هیچ قرضی نگه دارد که ممکن است زودتر از خودِ ریسمان از بین برود. دلیلش را با هم مرور کنیم: تابعی که `spawn` را صدا زده می‌تواند برگردد و قابِ پشته‌اش — همان‌جایی که `data`یِ قرضی زندگی می‌کرد — از بین برود، درست پیش از آنکه ریسمانِ تولیدشده اصلاً فرصتِ استفاده از آن را پیدا کند. کامپایلر هیچ راهی برایِ اثباتِ اینکه والد صبر می‌کند ندارد، پس همان اول از کامپایلِ کلوژری که داده‌ی محلی را قرض می‌گیرد سر باز می‌زند.

اگر با پایتون آمده باشی: `threading.Thread(target=lambda: print(data)).start()` بدونِ فکرکردن کار می‌کند، چون سیستمِ زباله‌روبِ مبتنی‌بر شمارشِ ارجاعِ پایتون، `data` را تا وقتی چیزی به آن اشاره کند — از جمله همان کلوژر، حتی از پشتِ مرزِ ریسمان — زنده نگه می‌دارد. اما همین‌جا تشبیه می‌شکند: زباله‌روبِ پایتون فقط جلویِ **آزاد شدنِ زودهنگام** را می‌گیرد، نه مسابقه‌ی داده رویِ همان مقدارِ مشترک؛ اگر دو ریسمانِ پایتون هم‌زمان همان شیء را تغییر بدهند، GIL به‌تنهایی جلویِ هر باگِ منطقی‌ای را نمی‌گیرد. راه‌حلِ Rust — مالکیت به‌علاوه‌یِ `'static` — دقیقاً همان تصمیمِ «این داده تا کِی معتبر است؟» را از یک اتفاقِ زمانِ‌اجرا به یک اثباتِ زمانِ‌کامپایل تبدیل می‌کند.

### مالکیتِ اشتراکی بینِ ریسمان‌ها: چرا `Arc`، نه `Rc`

فرض کن چند ریسمان باید یک مقدارِ یکسان را ببینند. مثلِ همیشه، هر مقدار دقیقاً یک مالک می‌خواهد — اما اینجا چند ریسمان هرکدام می‌خواهند مالکِ همان مقدار باشند. ۲.۶.۳ دقیقاً همین مسئله را حل کرد: همان `AppConfig`ی که آنجا با `Rc::new` ساختی و با `Rc::clone` بینِ یک handler و یک logger شریک شدی، مالکیتِ اشتراکی می‌دهد — چند دستگیره‌ی هم‌زمان به یک مقدارِ رویِ هیپ. اما همان درس یک قاعده را هم صریح گفت: `Rc` عمداً هرگز نباید از مرزِ یک ریسمان عبور کند، چون شمارنده‌اش یک عددِ صحیحِ معمولی و غیراتمی است — دو ریسمان که هم‌زمان رویِ همان شمارنده «یکی اضافه کن» را اجرا کنند، می‌توانند یک بار افزایش را گم کنند، و شمارنده زودتر از موعد به صفر برسد در حالی که هنوز یک دستگیره‌ی زنده جایی هست. مکانیزمِ دقیقی که کامپایلر با آن این را رد می‌کند — صفتِ `Send` — کارِ ۲.۸.۴ است؛ همین‌جا فقط قاعده‌ی عملی را لازم داری: همین که پایِ بیش از یک ریسمان به میان بیاید، `Rc` را با `Arc` عوض کن.

API‌اش حرف‌به‌حرف همانِ `Rc` است — `Arc::new`، `Arc::clone`، همان `strong_count` — فقط شمارنده‌اش با دستورالعمل‌هایِ **اتمیک** بالا و پایین می‌رود، که حتی وقتی چند هسته هم‌زمان به همان حافظه دست بزنند هم واقعاً تقسیم‌ناپذیرند. همین اتمیک‌بودن است که `Arc` را امن برایِ اشتراک بینِ ریسمان‌ها می‌کند، به قیمتِ کندترشدنِ هر کلون نسبت به `Rc`.

یک نکته: اگر `Arc` را فراموش کنی و بخواهی خودِ `Mutex` را با `move` دو بار به دو کلوژر بدهی، دقیقاً به یک خطایِ آشنا می‌خوری — همان «استفاده بعد از انتقال»یِ فاز ۱، این‌بار سرِ مرزِ `thread::spawn`؛ کاملش در «خطاهایی که خواهی دید» است.

### تغییرِ همان دادۀ مشترک: چرا `Mutex`، نه `RefCell`

`Arc` به‌تنهایی فقط اشتراک می‌دهد، نه تغییر — دقیقاً همان محدودیتی که ۲.۶.۳ رویِ `Rc` هم گفت: هر مالکی که از یک `Arc<T>` می‌گیری، فقط `&T` است. اگر ریسمان‌ها بخواهند همان مقدار را هم عوض کنند، به تغییرپذیریِ درونی نیاز داری — دقیقاً همان مسئله‌ای که ۲.۶.۵، `RefCell`، برایش حل کرد. اما `RefCell` هم برایِ همان دلیلِ `Rc` امن نیست: شمارنده‌هایِ قرضی‌اش که تویِ `Ref`/`RefMut` ردشان را نگه می‌دارد، هیچ‌کدام اتمیک نیستند. دو ریسمان که هم‌زمان `.borrow_mut()` بزنند، می‌توانند هر دو باور کنند تنها قرضِ زنده‌اند — دقیقاً همان مسابقه‌ی داده‌ای که قاعده‌ی هم‌نامی قرار بود غیرممکنش کند. مکانیزمِ دقیقش هم بازِ ۲.۸.۴ است؛ قاعده‌ی عملی همان الگویِ قبلی است: `RefCell` را با `Mutex` عوض کن.

`Mutex<T>` (مخففِ mutual exclusion، یعنی «انحصارِ متقابل») یک مقدار را طوری می‌پیچد که در هر لحظه فقط یک ریسمان بتواند بهش دسترسی داشته باشد — نه فقط برایِ نوشتن؛ برخلافِ `RefCell` که چند `Ref`ِ هم‌زمان را مجاز می‌داند، `.lock()` همیشه دسترسیِ انحصاری می‌دهد، حتی برایِ خواندنِ صرف. آن شکلِ آشناترِ `RefCell` — چند خواننده یا یک نویسنده — بینِ ریسمان‌ها هم هست، فقط اسمش `RwLock` است، موضوعِ ۲.۸.۲.

مالکیت و تغییرپذیری با هم: یک `Arc<Mutex<T>>` بساز، به ازایِ هر ریسمان یک `Arc::clone` بگیر، و هرکدام از پشتِ `Mutex` مقدار را تغییر بدهند:

```rust
let counter = Arc::new(Mutex::new(0));
let mut handles = Vec::new();

for _ in 0..10 {
    let counter = Arc::clone(&counter);
    handles.push(thread::spawn(move || {
        let mut guard = counter.lock().unwrap();
        *guard += 1;
    }));
}
```

```rust
for handle in handles {
    handle.join().unwrap();
}
println!("final count: {}", *counter.lock().unwrap());
```

```text
final count: 10
```

ده ریسمان، هرکدام دقیقاً یک بار شمارنده را بالا می‌برند؛ ترتیبِ دقیقِ اینکه کدام ریسمان زودتر قفل را می‌گیرد هر بار که اجرا کنی فرق می‌کند، اما چون `.join()` همه را پیش از خواندنِ نتیجه صبر می‌کند، عددِ نهایی همیشه ۱۰ است — همیشه.

```senpai-visual
{"kind":"ownership","labels":["Mutex<i32> on heap","Arc clone: thread A","Arc clone: thread B","Arc clone: thread C"]}
```

### `.lock()` یک گارد برمی‌گرداند — دقیقاً مثلِ `Ref`/`RefMut`یِ ۲.۶.۵

این باید آشنا باشد: `.lock()` یک `MutexGuard<T>` برمی‌گرداند (پیچیده‌شده تویِ یک `Result`، که بخشِ بعدی سراغش می‌رویم) — یک گاردِ RAII، دقیقاً همان الگویِ `Ref`/`RefMut`یِ ۲.۶.۵. بازش کن (`*guard`) تا مقدارِ درونی را بخوانی یا بنویسی؛ وقتی از اسکوپ خارج شود، `Drop` قفل را خودش، خودکار، آزاد می‌کند — تقریباً هیچ‌وقت خودت `.unlock()` صدا نمی‌زنی.

یک نکته‌ی واقعی و تیز: چون `MutexGuard` خودش یک قرض است (از پشتِ `Mutex` می‌آید)، همان قاعده‌های دامنه‌ی قرض رویش اجرا می‌شوند. یک عبارتِ بازگشتیِ لخت که در همان انتهایِ بلوکی که `Mutex` را محلی می‌سازد یک گارد را باز می‌کند، به `E0597` می‌خورد — جزئیاتِ کامل و راهِ‌حل در «خطاهایی که خواهی دید».

### مسموم‌شدنِ قفل: یک پنیک در حینِ نگه‌داشتنِ قفل

`.lock()` واقعاً یک `Result<MutexGuard<T>, PoisonError<MutexGuard<T>>>` برمی‌گرداند، نه یک `MutexGuard<T>`یِ خام. دلیلش: اگر ریسمانی درست در همان لحظه‌ای که قفل را در دست دارد پنیک کند، آن `Mutex` **مسموم (poisoned)** می‌شود — چون داده‌ی پشتِ آن قفل شاید نیمه‌آپدیت‌شده و خراب مانده باشد. از آن لحظه به بعد، هر `.lock()`ِ دیگری، رویِ هر ریسمانی، به‌جایِ گرفتنِ قفل، `Err` برمی‌گرداند.

```rust
let counter = Arc::new(Mutex::new(0));
let poisoner = Arc::clone(&counter);

let handle = thread::spawn(move || {
    let mut guard = poisoner.lock().unwrap();
    *guard += 1;
    panic!("simulated failure mid-update");
});
let _ = handle.join(); // Err — پنیک کرد؛ اینجا منتشرش نمی‌کنیم
```

```rust
let value = match counter.lock() {
    Ok(guard) => *guard,
    Err(poisoned) => *poisoned.into_inner(),
};
println!("value after recovery: {value}");
```

```text
thread '<unnamed>' (30132) panicked at phase2-intermediate\08-concurrency\01-threads-mutex-arc\examples\04-lock-poisoning-recovery.rs:17:9:
simulated failure mid-update
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
value after recovery: 1
```

(آن عددِ داخلِ پرانتز شناسه‌ی همان ریسمان است و هر بار که اجرا کنی فرق می‌کند؛ بقیه‌ی پیام همیشه همین است.)

پیامِ پنیک رویِ stderr چاپ می‌شود چون یک ریسمانِ غیرِاصلی پنیک کرده — فقط همان ریسمان می‌میرد، نه کلِ برنامه؛ ریسمانِ اصلی ادامه می‌دهد و برنامه با کدِ خروجِ ۰ تمام می‌شود. شاخه‌ی `Err` نشان می‌دهد قفل مسموم شده، و `poisoned.into_inner()` گاردِ محافظت‌شده را — با وجودِ مسموم‌بودن — همان‌طور پس می‌دهد؛ این یک انتخابِ مشروع است وقتی خودت مطمئنی داده‌ی پشتِ قفل، با وجودِ آن پنیک، هنوز قابلِ‌استفاده مانده.

اگر به‌جایِ این `match`، ساده رویِ `.lock()` هم `.unwrap()` بزنی، مسمومیت زنجیره‌ای می‌شود: ریسمانِ بعدی هم پنیک می‌کند. `examples/08-unwrap-poisoned-lock-broken.rs` دقیقاً همین را می‌سازد؛ متنِ کاملِ آن پنیکِ دوم در «خطاهایی که خواهی دید» است.

```senpai-visual
{"kind":"concurrency","labels":["thread locks and panics","Mutex becomes poisoned","next lock() returns Err","poisoned.into_inner() recovers"]}
```

---

## دست‌به‌کد

```sh
cargo run -p p2-08-01-threads-mutex-arc --example 01-spawn-and-join
cargo run -p p2-08-01-threads-mutex-arc --example 02-move-required
cargo run -p p2-08-01-threads-mutex-arc --example 03-shared-counter
cargo run -p p2-08-01-threads-mutex-arc --example 04-lock-poisoning-recovery
```

بعد چهارتاییِ خراب:

```sh
cargo run -p p2-08-01-threads-mutex-arc --example 05-borrow-without-move-broken --features broken
cargo run -p p2-08-01-threads-mutex-arc --example 06-mutex-without-arc-broken --features broken
cargo run -p p2-08-01-threads-mutex-arc --example 07-bare-tail-guard-broken --features broken
cargo run -p p2-08-01-threads-mutex-arc --example 08-unwrap-poisoned-lock-broken --features broken
```

بعد این‌ها را امتحان کن:

1. تویِ `03-shared-counter.rs`، عددِ حلقه را از ۱۰ به ۱۰۰ عوض کن — بازهم هر بار همان عددِ نهایی را می‌بینی؟
2. تویِ `04-lock-poisoning-recovery.rs`، خطِ `panic!` را کامنت کن — این بار کدام شاخه اجرا می‌شود، و چه چاپ می‌کند؟
3. تویِ `01-spawn-and-join.rs`، کلوژر را طوری عوض کن که به‌جایِ `2 + 2`، یک `String` برگرداند — نوعِ `JoinHandle` و نوعِ چیزی که `.join().unwrap()` می‌دهد چطور عوض می‌شود؟

---

## خطاهایی که خواهی دید

### `E0373` — کلوژرِ بدونِ `move` نمی‌تواند دیتایِ محلی را قرض بگیرد

```text
error[E0373]: closure may outlive the current function, but it borrows `data`, which is owned by the current function
  --> phase2-intermediate\08-concurrency\01-threads-mutex-arc\examples\05-borrow-without-move-broken.rs:10:32
   |
10 |     let handle = thread::spawn(|| {
   |                                ^^ may outlive borrowed value `data`
11 |         println!("{data:?}");
   |                    ---- `data` is borrowed here
   |
note: function requires argument type to outlive `'static`
  --> phase2-intermediate\08-concurrency\01-threads-mutex-arc\examples\05-borrow-without-move-broken.rs:10:18
   |
10 |       let handle = thread::spawn(|| {
   |  __________________^
11 | |         println!("{data:?}");
12 | |     });
   | |______^
help: to force the closure to take ownership of `data` (and any other referenced variables), use the `move` keyword
   |
10 |     let handle = thread::spawn(move || {
   |                                ++++

For more information about this error, try `rustc --explain E0373`.
```

**کامپایلر دقیقاً از چه چیزی شاکی است:** امضایِ `thread::spawn` کلوژرش را `'static` می‌خواهد. کلوژرِ بالا `data` را فقط قرض می‌گیرد (`||`، نه `move ||`)، و آن قرض عمرش به همان اسکوپِ `main` بسته است — کوتاه‌تر از عمرِ ریسمانی که ممکن است بعد از تمام‌شدنِ `main` هم زنده بماند.

**راهِ‌حل:** خودِ کامپایلر می‌گوید — `move` اضافه کن:

```rust
let handle = thread::spawn(move || {
    println!("{data:?}");
});
```

**چرا این راهِ‌حل درست است:** `move` کلوژر را مجبور می‌کند به‌جایِ قرض‌گرفتن، مالکیتِ `data` را کامل بگیرد. حالا کلوژر دیگر به چیزی بیرون از خودش وابسته نیست — هرجا برود، `data` هم با خودش می‌برد، پس دیگر مهم نیست `main` کِی برمی‌گردد.

### `E0382` — انتقالِ یک `Mutex` خام به دو کلوژر

```text
error[E0382]: use of moved value: `counter`
  --> phase2-intermediate\08-concurrency\01-threads-mutex-arc\examples\06-mutex-without-arc-broken.rs:15:19
   |
 9 |     let counter = Mutex::new(0);
   |         ------- move occurs because `counter` has type `std::sync::Mutex<i32>`, which does not implement the `Copy` trait
10 |
11 |     thread::spawn(move || {
   |                   ------- value moved into closure here
12 |         *counter.lock().unwrap() += 1;
   |          ------- variable moved due to use in closure
...
15 |     thread::spawn(move || {
   |                   ^^^^^^^ value used here after move
16 |         *counter.lock().unwrap() += 1;
   |          ------- use occurs due to use in closure

For more information about this error, try `rustc --explain E0382`.
```

**کامپایلر دقیقاً از چه چیزی شاکی است:** `Mutex<i32>` هیچ فرقی با هر مقدارِ غیرِ`Copy`ِ دیگری که فاز ۱ دیدی ندارد. کلوژرِ اول با `move` مالکیتِ `counter` را کامل گرفت؛ کلوژرِ دوم می‌خواهد دوباره همان مالکیت را بگیرد، و چیزی برایش نمانده — دقیقاً همان «استفاده بعد از انتقال»یِ فاز ۱، این‌بار سرِ مرزِ `thread::spawn`.

**راهِ‌حل:** `counter` را تویِ یک `Arc` بپیچ، و به‌جایِ خودِ `counter`، هر بار یک `Arc::clone` تازه بفرست:

```rust
let counter = Arc::new(Mutex::new(0));
let a = Arc::clone(&counter);
let b = Arc::clone(&counter);
thread::spawn(move || *a.lock().unwrap() += 1);
thread::spawn(move || *b.lock().unwrap() += 1);
```

**چرا این راهِ‌حل درست است:** حالا هر کلوژر مالکِ دستگیره‌ی خودش است — یک `Arc<Mutex<i32>>` جداگانه که به همان تخصیصِ زیرین اشاره می‌کند — نه مالکِ خودِ `Mutex`. همین دقیقاً همان کاری است که «مالکیتِ اشتراکی» تویِ بخشِ مفهوم برایش ساخته شده بود.

### `E0597` — بازکردنِ گارد در یک عبارتِ بازگشتیِ لخت

```text
error[E0597]: `counter` does not live long enough
  --> phase2-intermediate\08-concurrency\01-threads-mutex-arc\examples\07-bare-tail-guard-broken.rs:13:6
   |
12 |     let counter = Arc::new(Mutex::new(5));
   |         ------- binding `counter` declared here
13 |     *counter.lock().unwrap()
   |      ^^^^^^^----------------
   |      |
   |      borrowed value does not live long enough
   |      a temporary with access to the borrow is created here ...
14 | }
   | -
   | |
   | `counter` dropped here while still borrowed
   | ... and the borrow might be used here, when that temporary is dropped and runs the `Drop` code for type `std::sync::MutexGuard`
   |
   = note: the temporary is part of an expression at the end of a block;
           consider forcing this temporary to be dropped sooner, before the block's local variables are dropped
help: for example, you could save the expression's value in a new local variable `x` and then make `x` be the expression at the end of the block
   |
13 |     let x = *counter.lock().unwrap(); x
   |     +++++++                         +++

For more information about this error, try `rustc --explain E0597`.
```

**کامپایلر دقیقاً از چه چیزی شاکی است:** `counter.lock()` یک `MutexGuard` موقت می‌سازد که از پشتِ `counter` قرض گرفته. چون این عبارت، عبارتِ بازگشتیِ **آخرِ** بلوک است، آن گاردِ موقت تا همان انتهایِ بلوک زنده نگه داشته می‌شود — دقیقاً همان‌جایی که خودِ `counter` هم قرار است پاک شود. دو تا نمی‌توانند هم‌زمان از بین بروند وقتی یکی هنوز از دیگری قرض گرفته.

**راهِ‌حل:** خودِ کامپایلر پیشنهاد می‌دهد — مقدار را اول تویِ یک `let` بریز:

```rust
fn current_value() -> i32 {
    let counter = Arc::new(Mutex::new(5));
    let value = *counter.lock().unwrap();
    value
}
```

**چرا این راهِ‌حل درست است:** حالا گارد دقیقاً در همان خطی که `let value = ...;` تمام می‌شود پاک می‌شود — پیش از آنکه `counter` در انتهایِ بلوک از بین برود. مقدارِ `i32`ای که کپی شده (چون `i32` خودش `Copy` است) از آن گارد کاملاً مستقل زندگی می‌کند، پس بازگرداندنش دیگر به چیزی وابسته نیست که دارد همان لحظه پاک می‌شود.

### یک پنیکِ زمانِ اجرا — قفلِ مسموم، کورکورانه `unwrap`شده

```text
thread '<unnamed>' (28856) panicked at phase2-intermediate\08-concurrency\01-threads-mutex-arc\examples\08-unwrap-poisoned-lock-broken.rs:17:9:
simulated failure mid-update
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread 'main' (27372) panicked at phase2-intermediate\08-concurrency\01-threads-mutex-arc\examples\08-unwrap-poisoned-lock-broken.rs:21:32:
called `Result::unwrap()` on an `Err` value: PoisonError { .. }
```

(هر دو عددِ داخلِ پرانتز شناسه‌ی ریسمانند و هر اجرا فرق می‌کنند؛ بقیه‌ی متن ثابت است.)

**این حتی خطایِ کامپایلر نیست:** برنامه کامل کامپایل و اجرا می‌شود، و دقیقاً همان‌جا که خطِ ۲۱ (دومین `.lock().unwrap()`) اجرا می‌شود، پنیک می‌کند. ریسمانِ اول از قبل، عمداً، در حینِ نگه‌داشتنِ قفل پنیک کرده بود؛ آن `Mutex` از همان لحظه مسموم است. `.unwrap()` رویِ یک `Result::Err` — همان چیزی که `.lock()` حالا همیشه برمی‌گرداند — خودش هم پنیک می‌کند.

**راهِ‌حل:** رویِ `Err` صریح `match` بزن (یا `.into_inner()` را صدا بزن) به‌جایِ کورکورانه `unwrap` کردن — دقیقاً همان الگویِ `04-lock-poisoning-recovery.rs` که تویِ بخشِ مفهوم دیدی.

**چرا این راهِ‌حل درست است:** مسموم‌بودنِ قفل به‌تنهایی به این معنی نیست که داده‌ی پشتش واقعاً خراب است — فقط یعنی یک ریسمان درست در وسطِ کار پنیک کرده. کدی که خودش تصمیم می‌گیرد آن داده هنوز قابلِ‌اعتماد است یا نه، به‌جایِ کورکورانه پنیکِ ریسمانِ اول را به ریسمانِ خودش هم سرایت بدهد، دقیقاً همان انتخابی است که `.lock()` با برگرداندنِ `Result` به‌جایِ یک `MutexGuard`ِ خام، به تو می‌دهد.

---

## تمرین

### گرم‌کردن

<details>
<summary>این کد چه چاپ می‌کند؟</summary>

```rust
let handle = thread::spawn(|| 6 * 7);
let n = handle.join().unwrap();
println!("{n}");
```

</details>

<details>
<summary>جواب</summary>

```text
42
```

کلوژر چیزی قرض نمی‌گیرد، پس نیازی به `move` ندارد. `handle` از نوعِ `JoinHandle<i32>` است؛ `.join()` صبر می‌کند و `Ok(42)` می‌دهد؛ `.unwrap()` همان `42` را بیرون می‌کشد.

</details>

<details>
<summary>کامپایل می‌شود؟</summary>

```rust
let name = String::from("Frieren");
let handle = thread::spawn(|| {
    println!("{name}");
});
handle.join().unwrap();
```

</details>

<details>
<summary>جواب</summary>

نه — `E0373`. کلوژر `name` را فقط قرض می‌گیرد؛ امضایِ `spawn` کلوژرش را `'static` می‌خواهد. راهِ‌حل: `move ||`.

</details>

<details>
<summary>کامپایل می‌شود؟</summary>

```rust
let lock = Mutex::new(String::from("hi"));
thread::spawn(move || lock.lock().unwrap().push('!'));
thread::spawn(move || lock.lock().unwrap().push('?'));
```

</details>

<details>
<summary>جواب</summary>

نه — `E0382`. کلوژرِ اول با `move` مالکیتِ `lock` را کامل گرفت؛ کلوژرِ دوم می‌خواهد دوباره همان مالکیت را بگیرد و چیزی برایش نمانده. برایِ دو مالکِ هم‌زمان، `lock` باید تویِ یک `Arc` باشد.

</details>

<details>
<summary>درست یا غلط: دو ریسمان می‌توانند هم‌زمان رویِ همان <code>Mutex</code> قفل بگیرند، اگر هیچ‌کدام قرار نیست چیزی بنویسند.</summary>

قبل از دیدنِ جواب، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

غلط. برخلافِ `RefCell` که چند `Ref`ِ فقط‌خواندنیِ هم‌زمان را مجاز می‌داند، `.lock()` همیشه دسترسیِ انحصاری می‌دهد — حتی وقتی هیچ‌کدام قرار نیست بنویسند.

</details>

<details>
<summary>یک ریسمان درحینِ نگه‌داشتنِ قفل پنیک می‌کند. <code>.lock()</code>ِ بعدی رویِ یک ریسمانِ دیگر چه برمی‌گرداند؟</summary>

قبل از دیدنِ جواب، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

`Err` — قفل مسموم شده. نه بی‌نهایت صبر می‌کند، نه یک `Ok`ِ ساکت می‌دهد؛ هر `.lock()`ِ بعدی، رویِ هر ریسمانی، تا وقتی خودت صریح رسیدگی نکنی، همین `Err` را می‌دهد.

</details>

### تعمیر

هر چهار مثالِ خراب را درست کن:

1. `examples/05-borrow-without-move-broken.rs` را با اضافه‌کردنِ `move` به کلوژر درست کن.
2. `examples/06-mutex-without-arc-broken.rs` را با پیچیدنِ `counter` تویِ یک `Arc` و ساختنِ یک `Arc::clone` جدا برایِ هر کلوژر درست کن.
3. `examples/07-bare-tail-guard-broken.rs` را با ریختنِ مقدارِ بازشده تویِ یک `let` جداگانه، پیش از بازگرداندنش، درست کن.
4. `examples/08-unwrap-poisoned-lock-broken.rs` را طوری درست کن که دیگر پنیکِ دوم اتفاق نیفتد — رویِ `Err` صریح رسیدگی کن، به‌جایِ کورکورانه `unwrap`.

### پیاده‌سازی

دو تابع تویِ `src/lib.rs`، مشخصاتِ کاملِ هرکدام همان کامنتِ مستندسازِ بالایِ خودش است:

```sh
cargo test -p p2-08-01-threads-mutex-arc
```

- `sum_in_threads` — دیتا بینِ ریسمان‌ها تقسیم می‌شود، اما هیچ‌چیزی درحینِ اجرا مشترک نیست؛ هیچ `Mutex`ی لازم ندارد.
- `count_matching_in_threads` — ریسمان‌ها واقعاً یک شمارنده‌ی مشترک را هم‌زمان تغییر می‌دهند؛ دقیقاً همان چیزی که این درس برایش ساخته شده.

### بساز

طرحِ کوچکِ خودت را بساز: چند ریسمان که هرکدام نتیجه‌شان را تویِ یک `Vec` یا `HashMap`ِ مشترک، پشتِ `Arc<Mutex<...>>`، ثبت می‌کنند — مثلاً شمردنِ تکرارِ چند کلمه، یا جمع‌کردنِ نتیجه‌ی چند محاسبه‌ی مستقل. با یک تست یا `assert!` ثابت کن هر آپدیت دقیقاً یک‌بار اثر گذاشته، نه صفر بار و نه دوبار.

### چالش (اختیاری)

**بخشِ اول.** همین حالا `count_matching_in_threads` به ازایِ هر عضوِ `items` یک ریسمانِ تازه می‌سازد — برایِ چند تا آیتم خوب است، برایِ یک میلیون‌تایی اسراف است. یک نسخه‌ی دیگر بنویس که فقط `thread_count` ریسمانِ کارگر بسازد (دقیقاً همان‌طور که `sum_in_threads` ورودی‌اش را تکه‌تکه می‌کند)، و بازهم همان یک `Arc<Mutex<i32>>` را بینشان مشترک نگه دارد. مطمئن شو با همان ورودی، هر بار که تست را اجرا می‌کنی، دقیقاً همان جواب را می‌دهد.

**بخشِ دوم** (این یکی به جلو نگاه می‌کند). این آخرین‌باری است که یک شمارنده‌ی به همین سادگی را با دست، پشتِ `Arc<Mutex<...>>`، حمل می‌کنی — ۲.۸.۲، با یک نوعِ اتمیک، همین کار را بدونِ هیچ قفلی انجام می‌دهد؛ کنجکاو بمان، خودش نشانت می‌دهد چرا.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| ریسمان (thread) | ریسمانِ واقعیِ سطحِ سیستم‌عامل، قادر به اجرایِ واقعاً موازی | هر جا کاری باید همزمان و مستقل پیش برود |
| `JoinHandle<T>` | خروجیِ `thread::spawn`؛ `.join()` نتیجه یا پنیکِ ریسمان را پس می‌دهد | گرفتنِ نتیجه‌ی یک ریسمان، یا صبرکردن تا تمام شود |
| `Arc<T>` بینِ ریسمان‌ها | همان `Rc`، فقط با شمارنده‌ی اتمیک — امن برایِ عبور از مرزِ ریسمان | مالکیتِ اشتراکیِ یک مقدار بینِ چند ریسمان |
| `Mutex<T>` | دسترسیِ انحصاریِ یک ریسمان در هر لحظه، حتی برایِ خواندن | تغییرِ یک مقدارِ مشترک از چند ریسمان |
| `MutexGuard<T>` | گاردِ RAIIای که `.lock()` برمی‌گرداند؛ `Drop`اش قفل را آزاد می‌کند | خواندن/نوشتنِ مقدارِ پشتِ یک `Mutex` |
| مسموم‌شدنِ قفل (lock poisoning) | پنیکِ یک ریسمان در حینِ نگه‌داشتنِ قفل؛ از آن پس هر `.lock()` یک `Err` می‌دهد | تصمیم‌گیری درباره‌ی داده‌ای که ممکن است نیمه‌آپدیت مانده باشد |

### الان می‌دانی

- یک ریسمانِ واقعیِ سیستم‌عامل را با `thread::spawn` اجرا کردی و نتیجه‌اش را با `JoinHandle::join` پس گرفتی.
- چرا کلوژرِ `spawn` باید `move` باشد و `'static` بماند، و خطایِ `E0373` را از رو خواندی و رفع کردی.
- `Arc` همان `Rc`ِ ۲.۶.۳ است، فقط با شمارنده‌ی اتمیک — امن برایِ عبور از مرزِ ریسمان؛ `Mutex` همان کارِ `RefCell`ِ ۲.۶.۵ را می‌کند، فقط با قفلِ سطحِ سیستم‌عامل به‌جایِ شمارنده‌ی قرضِ زمانِ‌اجرا.
- `.lock()` یک گاردِ RAII برمی‌گرداند — دقیقاً همان الگویِ `Ref`/`RefMut` — و چون خودش یک قرض است، همان قاعده‌های دامنه‌ی قرض رویش اجرا می‌شوند (`E0597`).
- مسموم‌شدنِ قفل یعنی چه، چرا اتفاق می‌افتد، و چطور با `match` یا `.into_inner()` از آن برگردی به‌جایِ کورکورانه `unwrap` کردن.

### بعداً کامل‌تر می‌بینی

- **`RwLock`، چند خواننده یا یک نویسنده، و نوع‌هایِ اتمیک بدونِ هیچ قفلی** — [۲.۸.۲ — `RwLock`، `Semaphore`، `OnceLock`/`LazyLock`، اتمیک‌ها](../02-rwlock-semaphore-oncelock-atomics/README.fa.md)
- **کانال‌ها: به‌جایِ اشتراکِ حافظه، پیام بفرست** — [۲.۸.۳ — کانال‌ها و پیام‌رسانی](../03-channels-message-passing/README.fa.md)
- **مکانیزمِ دقیقِ اینکه چرا `Arc`/`Mutex` امن‌اند و `Rc`/`RefCell` نه: صفت‌هایِ `Send` و `Sync`** — [۲.۸.۴ — `Send` و `Sync`](../04-send-and-sync/README.fa.md)
- **`async`، برایِ کارهایِ گلوگاه-ورودی/خروجی به‌جایِ گلوگاه-پردازنده** — [۲.۸.۵ — فیوچرها و رانتایم‌ها](../05-futures-and-runtimes/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا کلوژرِ `spawn` باید `'static` باشد، و `move` دقیقاً چه کاری برایش می‌کند؟
- `Arc` و `Mutex` هرکدام دقیقاً چه مشکلی را حل می‌کنند، و چرا اشتراکِ وضعیتِ تغییرپذیر بینِ ریسمان‌ها به هردو با هم نیاز دارد؟
- چرا `Rc`/`RefCell` نمی‌توانند جایِ `Arc`/`Mutex` را بگیرند، همین‌که پایِ بیش از یک ریسمان وسط باشد — قاعده‌ی عملی‌اش چیست، و کدام درس مکانیزمِ دقیقش را تمام می‌کند؟
- `MutexGuard` چیست، و کدام الگویِ آشنایِ ۲.۶.۵ را تکرار می‌کند؟
- مسموم‌شدنِ قفل یعنی چه، و بعد از آن هر `.lock()` چه برمی‌گرداند؟

---

## بیشتر

- [کتابِ Rust — همروندیِ نترس](https://doc.rust-lang.org/book/ch16-00-concurrency.html) — همین موضوع، از زبانِ خودِ تیمِ Rust.
- [مستنداتِ `std::thread`](https://doc.rust-lang.org/std/thread/index.html) — کاملِ ماژول، همراه با `Builder` برایِ کنترلِ نامِ ریسمان و اندازه‌ی استک.
- [مستنداتِ `std::sync::Mutex`](https://doc.rust-lang.org/std/sync/struct.Mutex.html) — همراه با جزئیاتِ کاملِ مسموم‌شدن.
- [مستنداتِ `std::sync::Arc`](https://doc.rust-lang.org/std/sync/struct.Arc.html) — همان API، این‌بار با شمارنده‌ی اتمیک.

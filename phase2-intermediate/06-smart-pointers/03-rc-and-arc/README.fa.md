# ۲.۶.۳ — `Rc` و `Arc`

## در یک نگاه

بعد از این درس می‌توانی:

- توضیح بدهی چرا `Rc<T>` اجازه می‌دهد یک مقدارِ رویِ هیپ بیش از یک مالکِ واقعی و هم‌زمان داشته باشد — چیزی که فاز ۱ اصلاً امکانش را نمی‌داد — و دقیقاً بگویی آن مقدار کِی واقعاً آزاد می‌شود.
- فرقِ واقعیِ `.clone()` رویِ یک `Rc<T>` را با `.clone()` رویِ خودِ دیتایِ داخلش، با یک اثباتِ ملموس نشان بدهی — نه فقط با یک ادعا.
- برایِ یک تکه‌کدِ واقعی بینِ `Rc<T>` و `Arc<T>` یکی را انتخاب کنی، و در یک جمله بگویی چرا `Rc` اصلاً اجازه‌ی رد شدن از مرزِ یک ریسه را ندارد.

**زمان:** حدود ۶۰ دقیقه · **پیش‌نیاز:**
[۲.۶.۱ — `Box` و تخصیصِ هیپ](../01-box-and-heap-allocation/README.fa.md)،
[۲.۶.۲ — نوع‌های بازگشتی و شیء‌های صفتِ جعبه‌ای](../02-recursive-types-and-trait-objects/README.fa.md)

---

## چرا اهمیت دارد

۲.۳.۷ با یک وعده تمام شد، نه یک جمع‌بندیِ ساده: «`Rc`/`Arc` برایِ مالکیتِ
اشتراکی — قدمِ بعدی، وقتی حتی یک مالکِ تنها (مثلِ `Box<dyn Trait>`) هم کافی
نیست.» آن درس یک `Vec<Box<dyn Summarize>>` ساخت که دو ساختارِ کاملاً متفاوت
را کنارِ هم نگه می‌داشت — ولی هر `Box`، هرچقدر هم هوشمند، هنوز دقیقاً یک
مالک داشت: خودِ آن `Vec`. امروز آن وعده را ادا می‌کنیم — یک مقدارِ رویِ هیپ که
واقعاً بیش از یک مالک دارد، نه به‌عنوانِ استعاره، به‌عنوانِ یک واقعیتِ کاملاً
قابلِ‌اندازه‌گیری.

فاز ۱ یک قانون را بارها تکرار کرد: هر مقداری دقیقاً یک مالک دارد، همیشه.
[۲.۶.۱](../01-box-and-heap-allocation/README.fa.md) به‌ات `Box<T>` را داد —
ساده‌ترین اشاره‌گرِ هوشمند — و آن قانون را دست‌نخورده گذاشت؛ `Box` فقط یک
لایه‌ی غیرمستقیم‌سازی اضافه کرد، مالک همچنان دقیقاً یکی بود.
[۲.۶.۲](../02-recursive-types-and-trait-objects/README.fa.md) یک قدم جلوتر
رفت، سراغِ نوع‌هایِ بازگشتی، و باز هم همان قانون دست‌نخورده ماند: هر گره‌ی
درخت دقیقاً یک مالک داشت، فقط این‌بار آن مالک، خودش، پشتِ یک `Box` بود.

حالا این را در نظر بگیر: یک تنظیماتِ اپلیکیشن — یک `AppConfig` — که فقط
یک‌بار، موقعِ بالا آمدنِ برنامه، لود می‌شود؛ و بعد سه بخشِ کاملاً مستقلِ برنامه
لازمش دارند: یک هندلرِ درخواست، یک جابِ پس‌زمینه، و یک لاگر. کدام‌یک «مالکِ»
این تنظیمات است؟ جوابِ صادقانه: هیچ‌کدام، به‌تنهایی. اگر تنظیمات را به هندلر
بدهی، دیگر برایِ لاگر نمانده — دقیقاً همان خطایِ آشنایِ فاز ۱ که یک مقدار را
دوبار جابه‌جا می‌کنی. `Box` هم کمکی نمی‌کند؛ `Box` فقط محلِ مالکیت را عوض
می‌کند، تعدادش را نه.

این‌جاست که این درس یک استثنایِ واقعی معرفی می‌کند، نه یک نسخه‌ی کوچک‌ترِ
مالکیت. `Rc<T>` به‌ات **مالکیتِ اشتراکی (shared ownership)** می‌دهد: اجازه
می‌دهد بیش از یک متغیر، هم‌زمان، مالکِ واقعیِ همان یک مقدارِ رویِ هیپ باشند —
بدونِ `unsafe`، بدونِ دور زدنِ قانون، فقط با تغییرِ خودِ قانون. به‌جایِ «صفر یا
یک مالک»، Rust حالا می‌شمارد «چند مالکِ زنده الان وجود دارد»، و مقدار را
فقط وقتی آزاد می‌کند که آن شمارنده به صفر برسد.

---

## مفهوم

### `Rc<T>` — چند مالکِ واقعی، هم‌زمان

```rust
use std::rc::Rc;

struct AppConfig {
    app_name: String,
    max_connections: u32,
}

let config = Rc::new(AppConfig {
    app_name: "senpai-api".to_string(),
    max_connections: 100,
});
let for_handler = Rc::clone(&config);
let for_logger = Rc::clone(&config);
```

```rust
println!("handler sees:           {}", for_handler.app_name);
println!("logger sees:            {}", for_logger.app_name);
println!("original still usable:  {}", config.app_name);
println!("owners right now:       {}", Rc::strong_count(&config));
println!("max_connections too:    {}", for_handler.max_connections);
```

```text
handler sees:           senpai-api
logger sees:            senpai-api
original still usable:  senpai-api
owners right now:       3
max_connections too:    100
```

سه متغیر — `config`، `for_handler`، `for_logger` — هر سه، هم‌زمان، مالکِ
واقعیِ همان یک `AppConfig`ِ رویِ هیپ‌اند. هیچ‌کدام «حرکت» نکرده؛ اگر این را با
`Box<AppConfig>` امتحان می‌کردی، همان دومین `let` مقدار را از دستِ اولی
بیرون می‌کشید و کامپایلر با `E0382` جوابت را می‌داد — دقیقاً همان چیزی که
فاز ۱ به‌ات یاد داد. اینجا برعکس است: `Rc::clone(&config)` یک مالکِ **تازه**
می‌سازد، بدونِ اینکه مالکِ قبلی را بی‌اعتبار کند. و هیچ‌کدام از این سه هم
«اصلی‌تر» از بقیه نیست — از نگاهِ `Rc`، `config` و `for_handler` و
`for_logger` دقیقاً هم‌ارزند.

```senpai-visual
{"kind":"ownership","labels":["AppConfig on heap","Rc: config","Rc: for_handler","Rc: for_logger"]}
```

### شمارنده: دقیقاً کِی مقدار واقعاً آزاد می‌شود

هر `Rc<T>` کنارِ داده‌اش یک **شمارنده‌ی ارجاع (reference count)** نگه
می‌دارد — عددی که می‌گوید همین الان چند دستگیره به این تخصیصِ هیپ زنده‌اند. هر
`Rc::clone` این عدد را یکی بالا می‌برد؛ هر `drop` یکی پایین. برایِ اینکه
دقیقاً ببینی مقدار کِی واقعاً از حافظه پاک می‌شود، یک `Drop` بهش اضافه کن —
همان چیزی که [۱.۲.۵](../../../phase1-fundamentals/02-ownership-and-memory/05-drop-and-raii/README.fa.md)
یادت داد: `Drop` دقیقاً همان لحظه‌ای اجرا می‌شود که اسکوپِ مالکش تمام شود.

```rust
struct AppConfig {
    app_name: String,
}

impl Drop for AppConfig {
    fn drop(&mut self) {
        println!("  {} actually freed now", self.app_name);
    }
}

let a = Rc::new(AppConfig {
    app_name: "senpai-api".to_string(),
});
```

```rust
println!("after Rc::new:      count = {}", Rc::strong_count(&a));
let b = Rc::clone(&a);
println!("after first clone:  count = {}", Rc::strong_count(&a));
let c = Rc::clone(&a);
println!("after second clone: count = {}", Rc::strong_count(&a));
drop(b);
println!("after dropping one: count = {}", Rc::strong_count(&a));
drop(c);
println!("after dropping two: count = {}", Rc::strong_count(&a));
println!("dropping the last owner:");
drop(a);
```

```text
after Rc::new:      count = 1
after first clone:  count = 2
after second clone: count = 3
after dropping one: count = 2
after dropping two: count = 1
dropping the last owner:
  senpai-api actually freed now
```

پیامِ `actually freed now` دقیقاً یک‌بار چاپ می‌شود، و دقیقاً همان لحظه‌ای که
شمارنده به صفر می‌رسد — نه زودتر، وقتی `b` یا `c` رفتند، چون آن‌ها آخرین
دستگیره نبودند. هیچ‌کدام از مالک‌ها «ویژه» نیست؛ آنچه واقعاً تعیین می‌کند
دیتا کِی آزاد شود، فقط خودِ شمارنده است.

```senpai-visual
{"kind":"concept","labels":["count = 1","count = 2","count = 3","count = 2","count = 1","freed at 0"]}
```

### `.clone()` رویِ `Rc` ارزان است — نه یک کپیِ دیتا

اینجا یک تله‌ی واقعی هست: `some_rc.clone()` و `some_string.clone()` از نظرِ
نوشتار عینِ هم‌اند، ولی کارهایی که می‌کنند زمین تا آسمون فرق دارد. بیا به‌جایِ
اینکه فقط ادعا کنیم، ثابتش کنیم — با مقایسه‌ی آدرس‌ها:

```rust
#[derive(Clone)]
struct AppConfig {
    app_name: String,
}

let original = Rc::new(AppConfig {
    app_name: "senpai-api".to_string(),
});
let rc_clone = Rc::clone(&original);
let data_clone: AppConfig = (*original).clone();
```

```rust
println!(
    "Rc::as_ptr(&original) == Rc::as_ptr(&rc_clone):             {}",
    Rc::as_ptr(&original) == Rc::as_ptr(&rc_clone)
);
println!(
    "original.app_name.as_ptr() == data_clone.app_name.as_ptr(): {}",
    original.app_name.as_ptr() == data_clone.app_name.as_ptr()
);
```

```text
Rc::as_ptr(&original) == Rc::as_ptr(&rc_clone):             true
original.app_name.as_ptr() == data_clone.app_name.as_ptr(): false
```

`Rc::as_ptr` نشانیِ همان `AppConfig`ِ رویِ هیپ را برمی‌گرداند که `Rc` بهش
اشاره می‌کند. برایِ `original` و `rc_clone` این نشانی **یکی** است — چون
`Rc::clone` هیچ تخصیصِ تازه‌ای نساخت، فقط شمارنده را بالا برد و یک پوینترِ
دومِ ارزان به همان جا داد. اما `(*original).clone()` واقعاً `AppConfig` را
کپی کرد — یک `String` تازه برایِ `app_name` ساخته شد، با بایت‌هایِ خودش، رویِ
حافظه‌ای کاملاً جدا — و همین است که `false` را می‌بینی. به همین دلیل، عرفِ
این کد این است که به‌جایِ `x.clone()` بنویسی `Rc::clone(&x)`: همان‌جا، تویِ
محلِ فراخوانی، می‌گوید «این فقط یک کلونِ ارزانِ پوینتر است»، بدونِ اینکه
خواننده مجبور شود اول برود ببیند نوعِ `x` اصلاً چیست. و برایِ همان چکِ
`Rc::as_ptr(&a) == Rc::as_ptr(&b)` بالا، یک کوته‌نویسیِ آماده هم هست که
مستقیم‌تر همین را می‌گوید: `Rc::ptr_eq(&a, &b)`.

### چرا `Rc` فقط برایِ تک‌ریسمانی است

شمارنده‌ی داخلِ `Rc<T>` یک عددِ صحیحِ کاملاً معمولی است — نه چیزِ خاصی.
«یکی‌بالابردن» رویِ یک عددِ معمولی، تویِ سطحِ پردازنده، یک عملیاتِ تکی و
غیرقابلِ‌تقسیم نیست: باید اول عدد را بخوانی، بعد یکی بهش اضافه کنی، بعد
نتیجه را جایِ قبلی بنویسی. اگر دو ریسه دقیقاً همین سه قدم را «هم‌زمان» رویِ
همان شمارنده انجام بدهند، ممکن است هردو همان عددِ قدیمی را بخوانند، هردو
همان قدیمی‌بعلاوه‌یک را بنویسند، و یک افزایش کاملاً گم شود. نتیجه یک
مسابقه‌ی داده است: شمارنده زودتر از موعد به صفر می‌رسد، مقدار آزاد می‌شود،
درحالی‌که یک دستگیره به آن هنوز جایی زنده است — دقیقاً همان دسته از باگ که
دلیلِ اصلیِ وجودِ قاعده‌ی هم‌نامی است.

Rust این را در زمانِ اجرا رها نمی‌کند تا شانسی حل شود؛ کامپایلر همین امروز
جلویش را می‌گیرد: نوعِ `Rc<T>` صفتِ نشانه‌گذارِ `Send` را پیاده‌سازی نکرده،
پس هر تلاشی برایِ فرستادنِ آن به یک ریسه‌ی دیگر، همان لحظه‌ی کامپایل شکست
می‌خورد — نه ماه‌ها بعد، تویِ پروداکشن. مکانیزمِ دقیقِ عبور از مرزِ ریسه‌ها و
خودِ `Send` مالِ ماژول ۸ است؛ امروز فقط این واقعیت را لازم داری: `Rc` عمداً
تویِ همان یک ریسه می‌ماند.

### `Arc<T>` — همان API، شمارنده‌ای اتمی

`Arc<T>` ("**a**tomically **r**eference **c**ounted") دقیقاً همان مشکل را
حل می‌کند — با یک شمارنده‌ای که با دستورالعمل‌هایِ **اتمیِ** پردازنده بالا و
پایین می‌رود، نه با یک افزایشِ معمولی. یک عملیاتِ اتمی، برخلافِ افزایشِ
معمولیِ بالا، واقعاً تکی و غیرقابلِ‌تقسیم است، حتی وقتی چند هسته هم‌زمان به
همان حافظه دست می‌زنند — و دقیقاً همین است که `Arc<T>` را ایمن برایِ اشتراک
بینِ ریسه‌ها می‌کند. این ایمنی رایگان نیست: یک عملیاتِ اتمی باید با بقیه‌ی
هسته‌ها هماهنگ شود، پس به‌طورِ قابلِ‌اندازه‌گیری کندتر است از یک افزایشِ
معمولیِ `Rc`. API اما حرف‌به‌حرف همان است:

```rust
use std::sync::Arc;

struct AppConfig {
    app_name: String,
    max_connections: u32,
}

let config = Arc::new(AppConfig {
    app_name: "senpai-api".to_string(),
    max_connections: 100,
});
let for_handler = Arc::clone(&config);
```

```rust
println!("handler sees:     {}", for_handler.app_name);
println!("owners right now: {}", Arc::strong_count(&config));
println!("max_connections:  {}", config.max_connections);
```

```text
handler sees:     senpai-api
owners right now: 2
max_connections:  100
```

حرف‌به‌حرف همان کدِ `Rc` بالا، فقط با `Arc` جایِ `Rc`. هیچ‌جایِ این کد واقعاً
از مرزِ یک ریسه رد نمی‌شود — آن قسمت مالِ ماژول ۸ است. قاعده‌ی سرانگشتی همین
حالا هم روشن است: پیش‌فرض `Rc`، برایِ کدِ تک‌ریسمانی؛ همان لحظه‌ای که یک مقدار
واقعاً باید دستِ ریسه‌ی دیگری برسد، سراغِ `Arc` برو.

### کِی واقعاً به مالکیتِ اشتراکی نیاز داری — و کِی نه

پیش‌فرض همچنان همان چیزی است که فاز ۱ به‌ات یاد داد: مالکیتِ تکی و
قرض‌گیریِ معمولی (`&T`/`&mut T`). ساده‌تر است، کاملاً در زمانِ کامپایل چک
می‌شود، و هزینه‌ی اجرایی‌اش صفر است. `Rc`/`Arc` برایِ آن استثنایِ واقعی‌اند،
نه یک عادت — دو شکلِ واقعی که واقعاً به‌شان برمی‌خوری:

- **دیتایی که یک‌بار لود می‌شود و چند مالکِ مستقل لازمش دارند** — همان
  `AppConfig` بالا، بینِ یک هندلر، یک جابِ پس‌زمینه، و یک لاگر.
- **یک گره‌ی درخت یا گراف با بیش از یک والد** — مثلاً یک برچسبِ ژانر که دو
  آنیمه‌ی مختلف هردو بهش ارجاع می‌دهند، بدونِ اینکه متنِ ژانر دوبار تویِ
  حافظه کپی شود.

| نوع | تعدادِ مالک | شمارنده | برایِ چه |
|---|---|---|---|
| `Box<T>` | دقیقاً یکی | ندارد | یک مقدار که باید رویِ هیپ باشد؛ هنوز دقیقاً یک مالک |
| `Rc<T>` | چند تا، هم‌زمان | معمولی، غیراتمی | همان مقدار، چند مالکِ واقعی، همه تویِ یک ریسه |
| `Arc<T>` | چند تا، هم‌زمان | اتمی | همان چیزِ `Rc`، وقتی مالک‌ها ممکن است در ریسه‌هایِ مختلف باشند |

یک محدودیتِ مهم را هم صادقانه بگو: `Rc`/`Arc` به‌تنهایی فقط اجازه‌ی
**اشتراک** می‌دهند، نه تغییر. هر مالکی که از `Rc<T>` بگیری، فقط `&T` به‌ات
می‌دهد — چون اجازه دادنِ `&mut T` به بیش از یک مالکِ هم‌زمان دقیقاً همان
قاعده‌ی هم‌نامی را می‌شکند که فاز ۱ به‌ات یاد داد. اگر واقعاً لازم است مقدارِ
اشتراکی را هم تغییر بدهی، به یک ابزارِ دیگر نیاز داری —
[۲.۶.۵](../05-refcell-and-interior-mutability/README.fa.md) دقیقاً همین را
حل می‌کند. و خودِ مالکیتِ اشتراکی هم یک شکستِ واقعی دارد — وقتی دو مقدار،
هرکدام، صاحبِ یک `Rc` به آن یکی می‌شوند — که
[۲.۶.۴](../04-weak-and-reference-cycles/README.fa.md) کاملش را نشانت
می‌دهد.

---

## دست‌به‌کد

```sh
cargo run -p p2-06-03-rc-and-arc --example 01-multiple-owners
cargo run -p p2-06-03-rc-and-arc --example 02-count-and-drop-order
cargo run -p p2-06-03-rc-and-arc --example 03-clone-cost-comparison
cargo run -p p2-06-03-rc-and-arc --example 04-arc-same-api
```

بعد سه‌تای خراب:

```sh
cargo run -p p2-06-03-rc-and-arc --example 05-forgot-to-clone-broken --features broken
cargo run -p p2-06-03-rc-and-arc --example 06-cannot-mutate-through-rc-broken --features broken
cargo run -p p2-06-03-rc-and-arc --example 07-rc-arc-mismatch-broken --features broken
```

بعد این‌ها را امتحان کن:

۱. تویِ `01-multiple-owners.rs`، یک مالکِ سوم اضافه کن (مثلاً
   `for_background_job`) و `Rc::strong_count` را دوباره چاپ کن — الان چند
   است؟
۲. تویِ `02-count-and-drop-order.rs`، ترتیبِ `drop(b)` و `drop(c)` را
   برعکس کن — پیامِ `actually freed now` بازهم فقط یک‌بار، و فقط در آخر،
   چاپ می‌شود؟
۳. تویِ `04-arc-same-api.rs`، یک `Arc::clone` دومی هم بگیر و
   `Arc::strong_count` را دوباره چاپ کن — همان‌طور که با نسخه‌ی `Rc` بالاتر
   دیدی رفتار می‌کند؟

---

## خطاهایی که خواهی دید

### `E0382` — جابه‌جا کردنِ خودِ `Rc`، نه دیتایِ داخلش

```text
error[E0382]: the type `Rc` does not implement `Copy`
  --> phase2-intermediate\06-smart-pointers\03-rc-and-arc\examples\05-forgot-to-clone-broken.rs:29:17
   |
24 |     let config = Rc::new(AppConfig {
   |         ------ this move could be avoided by cloning the original `Rc`, which is inexpensive
...
28 |     announce(config);
   |              ------ value moved here
29 |     log_startup(config);
   |                 ^^^^^^ value used here after move
   |
   = note: consider using `Rc::clone`
note: consider changing this parameter type in function `announce` to borrow instead if owning the value isn't necessary
  --> phase2-intermediate\06-smart-pointers\03-rc-and-arc\examples\05-forgot-to-clone-broken.rs:15:21
   |
15 | fn announce(config: Rc<AppConfig>) {
   |    --------         ^^^^^^^^^^^^^ this parameter takes ownership of the value
   |    |
   |    in this function
help: clone the value to increment its reference count
   |
28 |     announce(config.clone());
   |                    ++++++++

For more information about this error, try `rustc --explain E0382`.
```

**کامپایلر به چه اعتراض دارد:** یک `Rc<AppConfig>` هنوز یک مقدارِ کاملاً
معمولی و مالک‌دار است. دادنِ همان `config` به دو تابعی که هرکدام مالکیت
می‌گیرند، دقیقاً مثلِ هر مقدارِ دیگری که `Copy` نیست، بارِ اول جابه‌جایش می‌کند —
`Rc` استثنا نیست، فقط چیزی که به‌اش اشاره می‌کند ارزان است.

**راه‌حل:** به‌جایِ دادنِ خودِ `config`، یک کلونِ تازه بساز — یکی برایِ هر
فراخوانی:

```rust
announce(Rc::clone(&config));
log_startup(Rc::clone(&config));
```

**چرا این راه‌حل است:** خودِ کامپایلر این را می‌گوید — «this move could be
avoided by cloning the original `Rc`, which is inexpensive.» `Rc::clone`
فقط شمارنده را یکی بالا می‌برد و یک دستگیره‌ی تازه می‌دهد؛ حالا هر دو تابع
مالکِ خودشان را دارند، و `config` هم بعدِ هر دو فراخوانی هنوز معتبر است.

### `E0596` — نمی‌شود از پشتِ یک `Rc` تغییرش داد

```text
error[E0596]: cannot borrow data in an `Rc` as mutable
  --> phase2-intermediate\06-smart-pointers\03-rc-and-arc\examples\06-cannot-mutate-through-rc-broken.rs:26:5
   |
26 |     config.raise_limit(50);
   |     ^^^^^^ cannot borrow as mutable
   |
   = help: trait `DerefMut` is required to modify through a dereference, but it is not implemented for `Rc<AppConfig>`

For more information about this error, try `rustc --explain E0596`.
```

**کامپایلر به چه اعتراض دارد:** `Rc<T>` صفتِ `Deref` را پیاده کرده — همان
چیزی که اجازه می‌دهد `config.app_name` یا `config.raise_limit(...)` بدونِ
نشان‌زداییِ دستی کار کند — ولی `DerefMut` را نه. هیچ راهی نیست که از پشتِ
یک `Rc` به `&mut T` برسی، حتی وقتی شمارنده الان دقیقاً `1` است، چون
کامپایلر باید همین حالا، بدونِ دانستنِ شمارنده در زمانِ اجرا، تصمیم بگیرد
که این کد ایمن است یا نه.

**راه‌حل:** با ابزارهایِ همین درس، هیچ راهِ درستی برایِ این نیست — و همین
خودش نکته‌ی درس است. مقداری که قرار است از پشتِ یک مرجعِ اشتراکی تغییر کند،
به یک نوعِ دیگر نیاز دارد، نه به تلاشِ بیشتر برایِ نشان‌زدایی از `Rc`.

**چرا این «راه‌حل» است:** چون این محدودیت تصادفی نیست. اگر `Rc<T>` اجازه‌ی
`&mut T` می‌داد، دو مالک می‌توانستند هم‌زمان دو `&mut` به همان دیتا بگیرند —
دقیقاً همان قاعده‌ی هم‌نامی که فاز ۱ گفت هرگز نباید بشکند. `Rc`/`Arc` عمداً
فقط `&T` می‌دهند؛ ابزاری که این محدودیت را با یک قاعده‌ی زمانِ‌اجرا (نه
زمانِ‌کامپایل) دور می‌زند مالِ [۲.۶.۵](../05-refcell-and-interior-mutability/README.fa.md) است.

### `E0308` — `Rc<T>` و `Arc<T>` قابلِ‌جابه‌جایی نیستند

```text
error[E0308]: mismatched types
  --> phase2-intermediate\06-smart-pointers\03-rc-and-arc\examples\07-rc-arc-mismatch-broken.rs:25:18
   |
25 |     spawn_worker(config);
   |     ------------ ^^^^^^ expected `Arc<AppConfig>`, found `Rc<AppConfig>`
   |     |
   |     arguments to this function are incorrect
   |
   = note: expected struct `Arc<AppConfig>`
              found struct `Rc<AppConfig>`
note: function defined here
  --> phase2-intermediate\06-smart-pointers\03-rc-and-arc\examples\07-rc-arc-mismatch-broken.rs:16:4
   |
16 | fn spawn_worker(config: Arc<AppConfig>) {
   |    ^^^^^^^^^^^^ ----------------------

For more information about this error, try `rustc --explain E0308`.
```

**کامپایلر به چه اعتراض دارد:** `Rc<AppConfig>` و `Arc<AppConfig>` دو
`struct` کاملاً جدا و بی‌ربط‌اند، فقط با متدهایِ هم‌نام. داشتنِ یک API
یکسان — `::new`، `::clone`، `::strong_count` — آن‌ها را یک نوع نمی‌کند، و
Rust هیچ تبدیلِ ضمنی‌ای بین‌شان انجام نمی‌دهد.

**راه‌حل:** اگر مقدار قرار است دستِ چیزی برسد که `Arc` می‌خواهد، از همان اول
با `Arc::new` بسازش:

```rust
let config = Arc::new(AppConfig {
    app_name: "senpai-api".to_string(),
});
spawn_worker(config);
```

**چرا این راه‌حل است:** بینِ `Rc` و `Arc` هیچ راهِ ارزانی برایِ رفتن از یکی به
آن یکی نیست — تبدیل‌شان یعنی ساختنِ یک تخصیصِ کاملاً تازه با نوعِ شمارنده‌ی
درست. ساده‌ترین راه این است که همان اول تصمیم بگیری این مقدار قرار است
تک‌ریسمانی بماند یا نه، و نوعش را از همان‌جا انتخاب کنی.

---

## تمرین

### گرم‌کردن

<details>
<summary>بعد از این کد، <code>Rc::strong_count(&a)</code> چند برمی‌گرداند؟</summary>

```rust
let a = Rc::new(5);
let b = Rc::clone(&a);
let c = Rc::clone(&a);
drop(b);
```

</details>

<details>
<summary>پاسخ</summary>

```text
2
```

`a` و `c` هنوز زنده‌اند؛ فقط `b` رفته. شمارنده با هر `Rc::clone` یکی بالا و
با هر `drop` یکی پایین می‌رود — نه بیشتر، نه کمتر.

</details>

<details>
<summary>این کامپایل می‌شود؟</summary>

```rust
struct AppConfig { app_name: String }
fn announce(config: Rc<AppConfig>) {}
fn log_startup(config: Rc<AppConfig>) {}

let config = Rc::new(AppConfig { app_name: "x".to_string() });
announce(config);
log_startup(config);
```

</details>

<details>
<summary>پاسخ</summary>

نه — `E0382`. `config` یک مقدارِ مالک‌دارِ معمولی است؛ دادنش به `announce`
جابه‌جایش می‌کند، و برایِ `log_startup` چیزی نمانده. باید `Rc::clone(&config)`
به هرکدام بدهی.

</details>

<details>
<summary>این کامپایل می‌شود؟</summary>

```rust
struct AppConfig { max_connections: u32 }
impl AppConfig {
    fn raise_limit(&mut self, by: u32) { self.max_connections += by; }
}

let config = Rc::new(AppConfig { max_connections: 100 });
config.raise_limit(50);
```

</details>

<details>
<summary>پاسخ</summary>

نه — `E0596`. `Rc<T>` فقط `Deref` دارد، نه `DerefMut`؛ از پشتِ آن هرگز
نمی‌توانی به `&mut T` برسی، حتی وقتی شمارنده `1` است.

</details>

<details>
<summary>درست یا غلط: <code>some_rc.clone()</code> رویِ یک <code>Rc&lt;String&gt;</code>، بایت‌هایِ آن <code>String</code> را کپی می‌کند.</summary>

فکرت را قبل از دیدنِ پاسخ بنویس.

</details>

<details>
<summary>پاسخ</summary>

غلط. `Rc<T>: Clone` صفتِ `Clone` را رویِ خودِ `Rc` پیاده کرده، پس
`some_rc.clone()` همیشه به همان پیاده‌سازی می‌رود — فقط شمارنده را بالا
می‌برد، بدونِ لمسِ دیتایِ داخلش.

</details>

### تعمیر

هر سه مثالِ خراب را درست کن:

۱. `examples/05-forgot-to-clone-broken.rs` را با کلون‌کردنِ `Rc` برایِ هر
   فراخوانی درست کن.
۲. `examples/06-cannot-mutate-through-rc-broken.rs` را طوری درست کن که
   دیگر تلاش نکند از پشتِ `Rc` تغییرش بدهد — مثلاً مقدارِ نهایی را همان
   لحظه‌ی ساختنِ `AppConfig` بگذار، پیش از اینکه بپیچی‌اش تویِ `Rc`.
۳. `examples/07-rc-arc-mismatch-broken.rs` را با ساختنِ `config` از رویِ
   `Arc::new` به‌جایِ `Rc::new` درست کن.

### پیاده‌سازی

پنج تابع در `src/lib.rs`، رویِ همان `AppConfig` که از قبل کاملاً نوشته شده:

```sh
cargo test -p p2-06-03-rc-and-arc
```

- `share_config` — یک `AppConfig` را تویِ یک `Rc` بپیچ.
- `add_owner` — یک دستگیره‌ی تازه به همان تخصیص بده، بدونِ کپیِ دیتا.
- `owner_count` — همین الان چند دستگیره زنده‌اند.
- `same_allocation` — آیا دو `Rc` به همان تخصیصِ هیپ اشاره می‌کنند، نه فقط
  دیتایِ برابر.
- `share_config_across_threads` — همان `share_config`، با `Arc`.

مشخصاتِ دقیقِ هرکدام — از جمله مثال‌هایِ عینی — در کامنتِ مستنداتِ بالایِ
همان تابع است.

### بساز

یک شکلِ واقعی برایِ مالکیتِ اشتراکی، مالِ خودت، انتخاب کن — یک دیتایِ
یک‌باربارگذاری‌شده که چند بخشِ مستقل لازمش دارند، یا یک گره‌ی کوچک (مثلِ یک
ژانر یا یک نویسنده) که دو ساختارِ دیگر هردو بهش ارجاع می‌دهند. با `Rc`
بسازش، و با `owner_count` یا `same_allocation` ثابت کن که واقعاً یک تخصیص
است، نه دو تا. در یک کامنت بنویس چرا مالکیتِ تکی برایِ همین سناریو کار
نمی‌کرد.

### چالش (اختیاری)

`Rc::get_mut(&mut self) -> Option<&mut T>` را تویِ مستنداتِ استاندارد
جست‌وجو کن — فقط وقتی `Some` برمی‌گرداند که شمارنده دقیقاً `1` باشد. یک
`Rc<i32>` بساز، `Rc::get_mut` را صدا بزن (باید `Some` بدهد)، یک کلون بگیر
و دوباره صدا بزن (این‌بار باید `None` بدهد)، کلون را `drop` کن و یک‌بارِ
دیگر صدا بزن (باید دوباره `Some` بدهد). در یک یا دو جمله بگو چرا این دقیقاً
همان قاعده‌ی هم‌نامیِ فاز ۱ است — فقط این‌بار کامپایلر با شمردنِ مالک‌ها
تصمیم می‌گیرد، نه با دنبال‌کردنِ یک قرض تا آخرِ اسکوپش.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| مالکیتِ اشتراکی (shared ownership) | بیش از یک متغیر، هم‌زمان، مالکِ واقعیِ یک مقدار | وقتی هیچ مالکِ تکیِ منطقی وجود ندارد |
| `Rc<T>` | اشاره‌گرِ هوشمندِ شمارشی، فقط تک‌ریسمانی | دیتایِ اشتراکیِ فقط‌خواندنی، در یک ریسه |
| `Arc<T>` | همان `Rc`، با شمارنده‌ای اتمی | همان دیتا، وقتی مالک‌ها ممکن است در ریسه‌هایِ مختلف باشند |
| شمارنده‌ی ارجاع (`strong_count`) | چند دستگیره همین الان زنده‌اند | دانستنِ اینکه مقدار کِی واقعاً آزاد می‌شود |
| `Rc::ptr_eq` | آیا دو دستگیره به همان تخصیص اشاره می‌کنند؟ | جدا کردنِ «همان مقدار» از «مقدارِ برابر» |

### الان می‌دانی

- `Rc<T>` اجازه می‌دهد بیش از یک متغیر، هم‌زمان، مالکِ واقعیِ یک مقدارِ
  رویِ هیپ باشند — یک استثنایِ رسمی رویِ قانونِ «دقیقاً یک مالک»ِ فاز ۱، نه
  یک نسخه‌ی کوچک‌ترش.
- `Rc::clone` هیچ‌وقت دیتا را کپی نمی‌کند — فقط شمارنده را یکی بالا می‌برد
  و یک پوینترِ دومِ ارزان به همان تخصیص می‌دهد؛ مقایسه‌ی `Rc::as_ptr` این
  را با آدرس ثابت کرد.
- مقدارِ زیربنایی فقط وقتی واقعاً از حافظه پاک می‌شود که شمارنده به صفر
  برسد — یعنی دقیقاً وقتی آخرین `Rc` هم `drop` شود.
- شمارنده‌ی `Rc` یک عددِ معمولیِ غیراتمی است، پس `Rc<T>` عمداً `Send`
  نیست و نمی‌تواند از مرزِ یک ریسه رد شود؛ `Arc<T>` همان API را با شمارنده‌ای
  اتمی می‌دهد، به قیمتِ کندترشدنِ هر کلون.
- `Rc<T>` و `Arc<T>` هرچند API یکسان دارند، دو نوعِ کاملاً جدا هستند — هیچ
  تبدیلِ ضمنی‌ای بین‌شان نیست.
- مالکیتِ تکی هنوز پیش‌فرضِ درست است؛ `Rc`/`Arc` برایِ آن استثنایِ واقعی‌اند —
  دیتایی که چند مالکِ مستقل لازم دارد، یا یک گره با بیش از یک والد — و
  هردو فقط `&T` می‌دهند، نه `&mut T`.

### بعداً کامل‌تر می‌بینی

- **`RefCell` و تغییرپذیریِ درونی، برایِ وقتی واقعاً باید از پشتِ یک مرجعِ
  اشتراکی تغییر بدهی** — [۲.۶.۵ — `RefCell`، `Cell` و معامله‌ی پنیکِ زمانِ اجرا](../05-refcell-and-interior-mutability/README.fa.md)
- **`Weak` و چرخه‌ی مرجع — شکستِ واقعیِ مالکیتِ اشتراکی** — [۲.۶.۴ — `Weak` و چرخه‌های ارجاع](../04-weak-and-reference-cycles/README.fa.md)
- **`thread::spawn` و عبورِ واقعی از مرزِ یک ریسه** — [۲.۸.۱ — ریسه‌ها، `Mutex`، `Arc`](../../08-concurrency/01-threads-mutex-arc/README.fa.md)
- **صفتِ `Send` و اینکه دقیقاً چرا `Rc<T>` آن را ندارد** — [۲.۸.۴ — `Send` و `Sync`](../../08-concurrency/04-send-and-sync/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا دو `Rc<T>` می‌توانند هم‌زمان مالکِ واقعیِ یک مقدار باشند، وقتی فاز ۱
  گفت هر مقدار دقیقاً یک مالک دارد؟
- `Rc::clone` دقیقاً با دیتایِ زیربنایی چه‌کار می‌کند، و مقایسه‌ی آدرس‌ها
  چطور این را ثابت کرد؟
- چرا `Rc<T>` فقط برایِ یک ریسه است — دقیقاً چه چیزی خراب می‌شد اگر نبود؟
- `Rc<T>` و `Arc<T>` یک API دارند ولی یک نوع نیستند — کدام خطایِ کامپایلر
  این را ثابت کرد؟
- یک شکلِ واقعی از مالکیتِ اشتراکی نام ببر، و بگو چرا آن‌جا یک مالکِ تکی
  کافی نبود.
- `Rc`/`Arc` به‌تنهایی فقط `&T` می‌دهند — اگر مقدارِ اشتراکی واقعاً باید
  تغییر کند، چه چیزی هنوز کم است؟

---

## بیشتر

- [کتابِ Rust — `Rc<T>`, the Reference Counted Smart Pointer](https://doc.rust-lang.org/book/ch15-04-rc.html) — همین موضوع، از زبانِ خودِ تیمِ Rust.
- [مستنداتِ `std::rc::Rc`](https://doc.rust-lang.org/std/rc/struct.Rc.html) — فهرستِ کاملِ متدهایش، از جمله `get_mut` و `ptr_eq` که امروز دیدی.
- [مستنداتِ `std::sync::Arc`](https://doc.rust-lang.org/std/sync/struct.Arc.html) — همان API، این‌بار با شمارنده‌ی اتمی.
- [مستنداتِ `std::sync::atomic`](https://doc.rust-lang.org/std/sync/atomic/index.html) — برایِ وقتی کنجکاو شدی خودِ عملیاتِ اتمی دقیقاً چطور کار می‌کند.

# ۲.۸.۴ — `Send` و `Sync`: واقعاً چه هستند، و چرا نوعِ تو `Send` نیست

## در یک نگاه

بعد از این درس می‌توانی:

- برایِ هر نوعی — چه مالِ کتابخانه‌ی استاندارد، چه نوشته‌ی خودت — با نگاه‌کردن به فیلدهایش بگویی `Send` هست یا نه، `Sync` هست یا نه.
- رابطه‌ی دقیقِ بینِ `Send` و `Sync` را در یک جمله بگویی، و با آن توضیح بدهی چرا `RefCell<T>` را می‌شود به یک ریسمانِ دیگر داد ولی نمی‌شود بینِ چند ریسمان به اشتراک گذاشت.
- خطایِ `E0277` («cannot be sent between threads safely») را رویِ کدِ خودت بخوانی، علتِ دقیقش را بگویی، و با انتخابِ نوعِ درست — نه حدس — رفعش کنی.

**زمان:** حدود ۴۵ دقیقه · **پیش‌نیاز:**
[۲.۸.۳ — کانال‌ها و پیام‌رسانی](../03-channels-message-passing/README.fa.md)

---

## چرا اهمیت دارد

از ۲.۸.۱ تا همین‌جا، هر بار پایِ وضعیتِ مشترک بینِ چند ریسمان به میان آمده، دستِ تو رفته سراغِ `Arc<Mutex<T>>` — نه `Rc<RefCell<T>>`. ۲.۸.۳، آن‌جا که کانال‌ها را با همین جفت مقایسه کرد، دوباره همان انتخاب را جلویت گذاشت. تا این‌جا این را به‌عنوانِ یک قانون پذیرفته‌ای: «وقتی ریسمان‌ها وسط‌اند، `Arc`/`Mutex`؛ `Rc`/`RefCell` را کنار بگذار.» ولی هیچ‌جا هنوز نگفته چرا — فقط گفته این کار را بکن.

۲.۶.۳ حتی از این هم جلوتر رفت. همان‌جا که `Rc<T>` را معرفی کرد، صریح نوشت نوعِ `Rc<T>` صفتِ نشانه‌گذارِ `Send` را پیاده‌سازی نکرده، و «مکانیزمِ دقیقِ عبور از مرزِ ریسمان‌ها و خودِ `Send` مالِ ماژول ۸ است.» و ۲.۶.۵ یک قدم عملی‌تر هم برداشت: تویِ چالشِ اختیاری‌اش از تو خواست یک `Rc<RefCell<i32>>` را داخلِ `thread::spawn` امتحان کنی. کامپایل نشد. کامپایلر دقیقاً همین را گفت: «cannot be sent between threads safely» — و همان‌جا نوشت همین الگو، رویِ چند ریسمان به‌جایِ یکی، موضوعِ همین ماژول است.

امروز آن دو وعده ادا می‌شود. `Send` و `Sync` دو صفتِ رسمی‌اند که خودِ Rust برایِ دقیقاً همین سؤال دارد. تفاوتشان با هر صفتی که تا این‌جا نوشته‌ای این است: نه تو جایی `impl`شان می‌کنی، نه کامپایلر منتظرِ یک `#[derive]` می‌ماند — کامپایلر با نگاه‌کردن به فیلدهایِ خودِ نوع، حسابش را می‌کند. بعد از این درس دیگر لازم نیست «`Arc`/`Mutex`، نه `Rc`/`RefCell`» را حفظ باشی — می‌توانی رویِ هر نوعِ تازه‌ای که می‌نویسی، از اول خودت اثباتش کنی.

---

## مفهوم

### `Send` — می‌شود مالکیت را به یک ریسمانِ دیگر داد؟

بیشترِ نوع‌هایی که تا این‌جایِ دوره نوشته‌ای، همین الان، بدونِ هیچ کارِ اضافه‌ای، می‌توانند مالکیتشان کاملاً بروند آن‌طرفِ مرزِ یک ریسمان:

```rust
use std::thread;

let name = String::from("senpai");
let numbers = vec![1, 2, 3, 4, 5];

let handle = thread::spawn(move || {
    let total: i32 = numbers.iter().sum();
    format!("{name} counted a total of {total}")
});
```

```rust
println!("{}", handle.join().unwrap());
```

```text
senpai counted a total of 15
```

هیچ‌جایِ این کد چیزِ خاصی نوشته نشده — نه یک `impl`، نه یک نشانه، نه حتی یک درخواست. `String` و `Vec<i32>` هردو همین‌طوری اجازه دارند مالکیتشان کاملاً منتقل شود. Rust به این ویژگی می‌گوید **`Send`**: نوعی `Send` است اگر بشود مالکیتِ یک مقدار از آن نوع را با خیالِ راحت به یک ریسمانِ دیگر داد. تقریباً هر نوعی که تا این‌جای دوره ساخته‌ای یا استفاده کرده‌ای — انواعِ عددی، `String`، `Vec<T>`، `HashMap`، ساختارها و شمارشی‌هایِ خودت — `Send` است. استثناها کم‌اند، ولی امروز دقیقاً یکی‌شان را می‌شناسی، به‌همراهِ دلیلش.

### `Sync` — می‌شود `&T` را هم‌زمان از چند ریسمان لمس کرد؟

`Send` می‌گوید مالکیت می‌تواند برود. یک سؤالِ کاملاً جدا این است: اگر مالکیت همین‌جا بماند، ولی چند ریسمان هم‌زمان یک ارجاعِ اشتراکی (`&T`) به همان مقدار داشته باشند — این هم امن است؟ به این ویژگی Rust می‌گوید **`Sync`**: نوعی `Sync` است اگر بشود یک `&T` از آن را با خیالِ راحت بینِ چند ریسمان به اشتراک گذاشت.

این دو صفت آن‌قدر به‌هم گره خورده‌اند که رابطه‌شان یک قاعده‌ی دقیق و قابلِ‌نقل‌قول دارد، همانی که از همین امروز تویِ ذهنت می‌ماند:

> `T` وقتی `Sync` است که `&T` خودش `Send` باشد — نه یک‌سره بیشتر، نه یک‌سره کمتر.

این رابطه چیزی نیست که فقط رویِ کاغذ درست باشد؛ خودِ کامپایلر رویش حساب می‌کند، و زیربخشِ بعدی دقیقاً همین را نشانت می‌دهد.

### یک دسته‌ی کاملاً تازه از صفت: صفتِ خودکار (auto trait)

هر صفتی که تا این‌جایِ دوره نوشته‌ای، یا خودت `impl`ش کرده‌ای (`impl Display for ...`) یا لااقل از کامپایلر با `#[derive(...)]` خواسته‌ای رویش بنویسدش. `Send` و `Sync` هیچ‌کدام این‌طور نیستند — نه `impl Send for Ticket` نوشتنی است، نه `#[derive(Send)]` معنایی دارد؛ هیچ‌کدام حتی نحوِ مجاز نیست. کامپایلر خودش، فقط با نگاه‌کردن به فیلدهایِ یک نوع، حساب می‌کند: **یک ساختار دقیقاً وقتی `Send` است که تک‌تکِ فیلدهایش `Send` باشند؛ دقیقاً همین قاعده برایِ `Sync` هم برقرار است.** به این دسته از صفت — بدونِ متد، بدونِ `impl` دستی، محاسبه‌شده از رویِ ترکیب — Rust می‌گوید **صفتِ خودکار (auto trait)**. (این‌ها را **صفتِ نشانه‌گذار (marker trait)** هم می‌گویند، همان اصطلاحی که ۲.۶.۳ برایِ `Send` به‌کار برد — نشانه‌گذارند چون هیچ متدی حمل نمی‌کنند، فقط یک واقعیت را رویِ نوع علامت می‌زنند.)

```rust
fn assert_send<T: Send>() {}
fn assert_sync<T: Sync>() {}

// For any `T`: if `T` is `Sync`, `&T` is `Send`.
fn shared_ref_is_send_when_sync<T: Sync>() {
    assert_send::<&T>();
}

struct Ticket {
    id: u32,
    title: String,
}
```

```rust
let ticket = Ticket { id: 7, title: "senpai-api outage".to_string() };
assert_send::<Ticket>();
assert_sync::<Ticket>();
shared_ref_is_send_when_sync::<Ticket>();
println!("Ticket #{} ({}) is Send and Sync; so is &Ticket", ticket.id, ticket.title);
```

```text
Ticket #7 (senpai-api outage) is Send and Sync; so is &Ticket
```

هیچ خطی از این کد نگفته `Ticket` از نظرِ `Send`/`Sync` چیست. `id: u32` و `title: String` هردو `Send` و `Sync`اند، پس `Ticket` هم — بدونِ هیچ کاری از طرفِ تو — همین دو صفت را دارد؛ و چون `Ticket` خودش `Sync` است، `shared_ref_is_send_when_sync` هم دقیقاً همان رابطه‌ای را که بالا خواندی رویِ آن ثابت کرد. این دقیقاً همان چیزی است که «خودکار» تویِ اسمش یعنی.

### چرا `Rc<T>` نه `Send` است، نه `Sync`

۲.۶.۳ این را نشانت داد: شمارنده‌ی داخلِ `Rc<T>` یک عددِ صحیحِ کاملاً معمولی است. «یکی بالا بردن» رویِ آن سه قدمِ جداست — بخوان، یکی اضافه کن، بنویس — و اگر دو ریسمان هم‌زمان دقیقاً همین سه قدم را رویِ همان شمارنده انجام بدهند، یک افزایش می‌تواند کاملاً گم شود: مسابقه‌ی داده، رویِ خودِ شمارنده.

حالا می‌توانی این را با زبانِ امروز بگویی. اگر `Rc<T>` اجازه داشت `Send` باشد، هیچ‌چیز جلویِ همان مسابقه را نمی‌گرفت — دو ریسمان می‌توانستند هرکدام یک `Rc` بگیرند و هم‌زمان کلون کنند. Rust این احتمال را همان لحظه‌ی کامپایل می‌بندد: `Rc<T>` نه `Send` است نه `Sync` — نه مالکیتش را می‌شود به ریسمانِ دیگر داد، نه `&Rc<T>`اش را می‌شود به اشتراک گذاشت، چون هردو در نهایت به همان شمارنده‌ی ناامن ختم می‌شوند.

`Arc<T>` دقیقاً همان مشکل را حل می‌کند، با همان راهی که ۲.۶.۳ نشانت داد: شمارنده‌اش با دستورالعمل‌هایِ **اتمیِ** پردازنده بالا و پایین می‌رود — یک عملیاتِ تکی و غیرقابلِ‌تقسیم، حتی وقتی چند هسته هم‌زمان به همان حافظه دست می‌زنند. هیچ مسابقه‌ای رویِ شمارنده ممکن نیست، پس هیچ دلیلی هم نیست `Arc<T>` از `Send`/`Sync` محروم بماند — و نیست: `Arc<T>` هم `Send` است هم `Sync`، به شرطِ اینکه چیزی که داخلش نگه می‌دارد خودش هم `Send` باشد هم `Sync`.

```senpai-visual
{"kind":"ownership","labels":["Rc: plain counter","thread boundary","compile error","Arc: atomic counter","crosses safely"]}
```

### `RefCell<T>`: می‌شود جابه‌جا کرد، نمی‌شود به اشتراک گذاشت

۲.۶.۵ نشانت داد `RefCell<T>` چطور قاعده‌ی هم‌نامی را از زمانِ کامپایل به زمانِ اجرا می‌برد: `.borrow()`/`.borrow_mut()` گاردهایی برمی‌گردانند، پشتِ صحنه از رویِ یک شمارنده که می‌گوید همین الان چند قرضِ خواندنی و کدام قرضِ نوشتنی زنده است. آن شمارنده هم — درست مثلِ شمارنده‌ی `Rc<T>` — یک عددِ کاملاً معمولی و غیراتمی است، نه چیزِ خاصی.

این‌جا اما فرق با `Rc<T>` شروع می‌شود. اگر یک `RefCell<T>` کامل — با هرچه داخلش هست — به یک ریسمانِ دیگر منتقل شود، فقط یک ریسمان در هر لحظه بهش دسترسی خواهد داشت؛ آن شمارنده هیچ‌وقت هم‌زمان از دو جا لمس نمی‌شود. برایِ همین `RefCell<T>` — تا وقتی خودِ `T` هم `Send` باشد — `Send` است. ولی اگر بخواهی یک `&RefCell<T>` را بینِ چند ریسمان به اشتراک بگذاری، آن‌وقت چند ریسمان می‌توانند هم‌زمان `.borrow_mut()` را صدا بزنند — و هرکدام، بدونِ خبر از آن یکی، همان شمارنده‌ی غیراتمی را می‌خواند و می‌نویسد. دقیقاً همان مسابقه‌ای که رویِ شمارنده‌ی `Rc<T>` دیدی، این‌بار رویِ شمارنده‌ی قرض. برایِ همین `RefCell<T>` عمداً `Sync` نیست — فرقی نمی‌کند `T` داخلش چه باشد.

### چرا `Mutex<T>` فرق دارد

اگر نتیجه‌ی بالا را با یک قانونِ خشک جمع‌بندی کنی — «هرچه از پشتِ یک ارجاعِ اشتراکی نوشته می‌شود، `Sync` نیست» — جواب غلط می‌گیری، و `Mutex<T>` دقیقاً همان استثنایی است که نشان می‌دهد چرا. `Mutex<T>` هم، مثلِ `RefCell<T>`، قاعده‌ی هم‌نامی را به زمانِ اجرا می‌برد: `.lock()` به‌جایِ `.borrow_mut()` می‌نشیند. ولی به‌جایِ یک شمارنده‌ی معمولی، از یک قفلِ واقعی و ایمن‌برایِ‌ریسمان استفاده می‌کند — همان چیزی که ۲.۸.۱ به‌کارت برد. آن قفل تضمین می‌کند در هر لحظه دقیقاً یک ریسمان اجازه‌ی دسترسی دارد؛ بقیه منتظر می‌مانند، نه اینکه هم‌زمان دست‌درازی کنند.

نتیجه یک تفاوتِ واقعاً ظریف است: `Mutex<T>` هم `Send` است، هم `Sync` — فقط به شرطِ `T: Send`، نه `T: Sync`! چون قفل از اساس جلویِ دسترسیِ هم‌زمان را می‌گیرد، دیگر مهم نیست خودِ `T` می‌توانست از پسِ دسترسیِ هم‌زمان بربیاید یا نه؛ هیچ‌وقت قرار نیست دو ریسمان هم‌زمان بهش برسند. برایِ اثباتش لازم نیست فقط حرفم را باور کنی: حتی `Mutex<RefCell<i32>>`، با اینکه `RefCell<i32>` خودش `Sync` نیست، باز هم `Sync` است — دقیقاً به همین دلیل.

پس این دقیقاً همان الگویِ `Rc` در برابرِ `Arc` است، فقط این‌بار رویِ تغییرپذیریِ درونی به‌جایِ رویِ شمارنده‌ی مرجع: `RefCell<T>` قاعده را با یک ابزارِ ساده و غیراتمی به زمانِ اجرا می‌برد — خوب برایِ یک ریسمان، ناامن برایِ چند تا؛ `Mutex<T>` همان کار را با ابزاری ایمن‌برایِ‌ریسمان انجام می‌دهد. `Arc<Mutex<T>>`، آن ترکیبی که از ۲.۸.۱ به بعد بدونِ توضیحِ کامل به‌کار برده‌ای، حالا از اول تا آخر روشن است: `Arc` مالکیتِ اشتراکی را امن می‌کند (شمارنده‌ی اتمی)، `Mutex` تغییرپذیریِ مشترک را امن می‌کند (قفلِ ایمن‌برایِ‌ریسمان). هرکدام را با `Rc`/`RefCell` عوض کنی، دقیقاً همان قطعه‌ی ناامن را برمی‌گردانی.

```senpai-visual
{"kind":"concurrency","labels":["RefCell: plain borrow flag","shared across threads","compile error","Mutex: real lock","safe to share"]}
```

### اشاره‌گرهایِ خام، و دادنِ دستیِ این دو صفت — به‌طورِ خلاصه

دو نکته‌ی کوتاه، فقط برایِ اینکه غافلگیر نشوی. اشاره‌گرهایِ خام — `*const T` و `*mut T` — هم به‌طورِ پیش‌فرض نه `Send`اند نه `Sync`، دقیقاً به همان دلیلِ کلی: کامپایلر هیچ راهی ندارد بداند چیزی که به آن اشاره می‌کنند از پشتِ چند ریسمان دست‌نخورده می‌ماند یا نه، پس محتاطانه‌ترین فرض را می‌گیرد. کارِ واقعی با اشاره‌گرهایِ خام کارِ ۲.۱۰ است.

و همین‌طور: دیدی هیچ‌جا خودمان `impl Send for X` ننوشتیم — این تصادفی نیست، ولی کاملاً هم بن‌بست نیست. نویسنده‌ی یک نوع می‌تواند با `unsafe impl Send for X {}` (یا `Sync`) دستی این صفت را به نوعش بدهد، وقتی خودش، با دست، تضمین کرده که واقعاً امن است. این یک قولِ `unsafe` واقعی است — دقیقاً همان نوع تعهدی که ۲.۱۰ رسمی بازش می‌کند؛ امروز فقط باید بدانی این در چنته هست، نه اینکه چطور نوشته می‌شود.

---

## دست‌به‌کد

```sh
cargo run -p p2-08-04-send-and-sync --example 01-ordinary-types-are-send
cargo run -p p2-08-04-send-and-sync --example 02-auto-trait-from-fields
```

بعد سه‌تای خراب:

```sh
cargo run -p p2-08-04-send-and-sync --example 03-struct-with-rc-not-send-broken --features broken
cargo run -p p2-08-04-send-and-sync --example 04-rc-refcell-thread-spawn-broken --features broken
cargo run -p p2-08-04-send-and-sync --example 05-arc-refcell-not-sync-broken --features broken
```

و حالا راه‌حلِ درست:

```sh
cargo run -p p2-08-04-send-and-sync --example 06-arc-mutex-fix
```

بعد این‌ها را امتحان کن:

۱. تویِ `02-auto-trait-from-fields.rs`، یک فیلدِ `count: Rc<i32>` به `Ticket` اضافه کن. سه جایِ صداکردنِ جداگانه تویِ `main` دیگر کامپایل نمی‌شوند — کدام سه خط، و هرکدام چرا؟
۲. تویِ `06-arc-mutex-fix.rs`، تعدادِ ریسمان‌ها را از `4` به `8` ببر و شمارشِ داخلیِ هر ریسمان را از `1000` به `500` نصف کن. عددِ نهایی هنوز دقیقاً همان `4000` می‌ماند؟
۳. تویِ `04-rc-refcell-thread-spawn-broken.rs`، فقط `Rc::new` را با `Arc::new` عوض کن — `RefCell` را دست‌نخورده بگذار. پیش‌بینی کن پیامِ خطا عوض می‌شود یا نه، بعد اجرا کن و ببین. (راهنمایی: این دقیقاً مثالِ ۰۵ است.)

---

## خطاهایی که خواهی دید

### `E0277` (۱) — یک فیلد کافی است تا کلِ ساختار `Send` نباشد

```text
error[E0277]: `Rc<i32>` cannot be sent between threads safely
  --> phase2-intermediate\08-concurrency\04-send-and-sync\examples\03-struct-with-rc-not-send-broken.rs:19:19
   |
19 |     assert_send::<HoldsRc>();
   |                   ^^^^^^^ `Rc<i32>` cannot be sent between threads safely
   |
   = help: within `HoldsRc`, the trait `Send` is not implemented for `Rc<i32>`
note: required because it appears within the type `HoldsRc`
  --> phase2-intermediate\08-concurrency\04-send-and-sync\examples\03-struct-with-rc-not-send-broken.rs:11:8
   |
11 | struct HoldsRc {
   |        ^^^^^^^
note: required by a bound in `assert_send`
  --> phase2-intermediate\08-concurrency\04-send-and-sync\examples\03-struct-with-rc-not-send-broken.rs:16:19
   |
16 | fn assert_send<T: Send>() {}
   |                   ^^^^ required by this bound in `assert_send`

For more information about this error, try `rustc --explain E0277`.
```

**کامپایلر به چه اعتراض دارد:** `assert_send` از `HoldsRc` می‌خواهد `Send` باشد. `HoldsRc` دو فیلد دارد: `label: String` (`Send`) و `count: Rc<i32>` (نه `Send`). یک ساختار دقیقاً وقتی `Send` است که **تک‌تکِ** فیلدهایش `Send` باشند — یکی که نباشد کافی است کلِ ساختار را از رده خارج کند، و کامپایلر همین را می‌گوید.

**راه‌حل:** اگر `HoldsRc` واقعاً باید بینِ ریسمان‌ها جابه‌جا شود، فیلدش باید `Arc<i32>` باشد، نه `Rc<i32>`:

```rust
struct HoldsRc {
    label: String,
    count: Arc<i32>,
}
```

**چرا این راه‌حل است:** `Arc<i32>` — برخلافِ `Rc<i32>` — خودش `Send` است، چون شمارنده‌اش اتمی است. وقتی هر فیلدِ `HoldsRc` `Send` باشد، خودِ `HoldsRc` هم — بدونِ هیچ کارِ اضافه‌ای — دوباره `Send` می‌شود.

### `E0277` (۲) — `Rc<RefCell<i32>>` داخلِ `thread::spawn`

```text
error[E0277]: `Rc<RefCell<i32>>` cannot be sent between threads safely
   --> phase2-intermediate\08-concurrency\04-send-and-sync\examples\04-rc-refcell-thread-spawn-broken.rs:15:32
    |
 15 |       let handle = thread::spawn(move || {
    |                    ------------- ^------
    |                    |             |
    |  __________________|_____________within this `{closure@04-rc-refcell-thread-spawn-broken.rs:15:32}`
    | |                  |
    | |                  required by a bound introduced by this call
 16 | |         *shared.borrow_mut() += 1;
 17 | |     });
    | |_____^ `Rc<RefCell<i32>>` cannot be sent between threads safely
    |
    = help: within `{closure@04-rc-refcell-thread-spawn-broken.rs:15:32}`, the trait `Send` is not implemented for `Rc<RefCell<i32>>`
note: required because it's used within this closure
   --> phase2-intermediate\08-concurrency\04-send-and-sync\examples\04-rc-refcell-thread-spawn-broken.rs:15:32
    |
 15 |     let handle = thread::spawn(move || {
    |                                ^^^^^^^
note: required by a bound in `spawn`
   --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\std\src\thread\functions.rs:128:8
    |
125 | pub fn spawn<F, T>(f: F) -> JoinHandle<T>
    |        ----- required by a bound in this function
...
128 |     F: Send + 'static,
    |        ^^^^ required by this bound in `spawn`
    = note: the full name for the type has been written to 'M:\SenPai-Rust-Journey\target\debug\examples\04_rc_refcell_thread_spawn_broken.long-type-11410446554853862890.txt'
    = note: consider using `--verbose` to print the full type name to the console

For more information about this error, try `rustc --explain E0277`.
```

دو خط از این خروجی مخصوصِ همین اجرا و همین دستگاه‌اند، نه بخشی از خودِ درس: مسیرِ کتابخانه‌ی استاندارد (خطِ `required by a bound in spawn`) چون به پوشه‌ی نصبِ Rust رویِ دستگاهِ خودت بستگی دارد، و آن هشِ داخلِ نامِ فایلِ `long-type-...txt` چون هر بار که همین مثال را کامپایل کنی عوض می‌شود — حتی رویِ همین دستگاه، بدونِ اینکه کدت عوض شده باشد. بقیه‌ی پیام — کدِ خطا، پیامِ اصلی، شماره‌خط و ستون‌هایِ فایلِ خودِ درس — دقیقاً همینی می‌ماند که الان می‌بینی.

**کامپایلر به چه اعتراض دارد:** کلوژرِ دادنی به `thread::spawn` با `move` مالکیتِ `shared: Rc<RefCell<i32>>` را کامل می‌گیرد، و همان کلوژر خودش باید `Send` باشد — این را خودِ امضایِ `spawn` می‌خواهد: `F: Send + 'static`. چون `Rc<RefCell<i32>>` نه `Send` است نه `Sync` (زیربخشِ «چرا `Rc<T>` نه `Send` است، نه `Sync`» را ببین)، کلوژری که آن را در خودش نگه می‌دارد هم نمی‌تواند `Send` باشد.

**راه‌حل:** دقیقاً همان چیزی که از ۲.۸.۱ به بعد استفاده کرده‌ای — `Arc<Mutex<T>>`:

```rust
let shared = Arc::new(Mutex::new(0));
let handle = thread::spawn(move || {
    *shared.lock().unwrap() += 1;
});
```

**چرا این راه‌حل است:** `Arc<Mutex<i32>>` هم `Send` است (شمارنده‌ی `Arc` اتمی است، و `Mutex<i32>` خودش `Send` است چون `i32: Send`) هم `Sync` (چون همین `Mutex<i32>: Send` کافی است — زیربخشِ «چرا `Mutex<T>` فرق دارد» را ببین). کلوژری که فقط این نوع را نگه می‌دارد، خودش هم `Send` می‌شود، و `thread::spawn` راضی است.

### `E0277` (۳) — عوض‌کردنِ `Rc` به `Arc` به‌تنهایی کافی نیست

```text
error[E0277]: `RefCell<i32>` cannot be shared between threads safely
   --> phase2-intermediate\08-concurrency\04-send-and-sync\examples\05-arc-refcell-not-sync-broken.rs:17:32
    |
 17 |       let handle = thread::spawn(move || {
    |  __________________-------------_^
    | |                  |
    | |                  required by a bound introduced by this call
 18 | |         *clone_for_thread.borrow_mut() += 1;
 19 | |     });
    | |_____^ `RefCell<i32>` cannot be shared between threads safely
    |
    = help: the trait `Sync` is not implemented for `RefCell<i32>`
    = note: if you want to do aliasing and mutation between multiple threads, use `std::sync::RwLock` instead
    = note: required for `Arc<RefCell<i32>>` to implement `Send`
note: required because it's used within this closure
   --> phase2-intermediate\08-concurrency\04-send-and-sync\examples\05-arc-refcell-not-sync-broken.rs:17:32
    |
 17 |     let handle = thread::spawn(move || {
    |                                ^^^^^^^
note: required by a bound in `spawn`
   --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\std\src\thread\functions.rs:128:8
    |
125 | pub fn spawn<F, T>(f: F) -> JoinHandle<T>
    |        ----- required by a bound in this function
...
128 |     F: Send + 'static,
    |        ^^^^ required by this bound in `spawn`

For more information about this error, try `rustc --explain E0277`.
```

(همان نکته‌ی مسیرِ محلی که در خطایِ قبلی دیدی، رویِ خطِ `required by a bound in spawn` این‌جا هم صادق است.)

**کامپایلر به چه اعتراض دارد:** این‌بار پیام فرق دارد — «cannot be **shared** between threads safely»، نه «sent». چون این‌بار `Arc::clone` داخلِ کلوژر می‌رود، نه خودِ `Rc`، کامپایلر مستقیم سراغِ `Sync` می‌رود: `Arc<T>` تنها وقتی `Send` است که `T` هم `Send` باشد هم `Sync` — و `RefCell<i32>` دقیقاً همان‌جایی است که می‌شکند: `Send` هست، `Sync` نیست. پیام حتی خودش یک راهِ جایگزین هم پیشنهاد می‌دهد — `RwLock` (موضوعِ ۲.۸.۲) — ولی این درس رویِ همان `Mutex`ی می‌ماند که تا این‌جا دیده‌ای.

**راه‌حل:** به‌جایِ `RefCell`، `Mutex` بگذار — همان راه‌حلِ خطایِ قبلی، این‌بار با `Arc`ای که از اول درست بود:

```rust
let shared = Arc::new(Mutex::new(0));
let clone_for_thread = Arc::clone(&shared);
let handle = thread::spawn(move || {
    *clone_for_thread.lock().unwrap() += 1;
});
```

**چرا این راه‌حل است:** `Mutex<i32>` — برخلافِ `RefCell<i32>` — هم `Send` است هم `Sync` (زیربخشِ «چرا `Mutex<T>` فرق دارد» را ببین)، پس `Arc<Mutex<i32>>` هر دو شرطی را که `Arc` می‌خواهد دارد: `T: Send + Sync`.

---

## تمرین

### گرم‌کردن

<details>
<summary>این کامپایل می‌شود؟</summary>

```rust
let numbers = vec![1, 2, 3];
let handle = std::thread::spawn(move || numbers.iter().sum::<i32>());
handle.join().unwrap();
```

</details>

<details>
<summary>پاسخ</summary>

بله. `Vec<i32>` مثلِ تقریباً هر نوعِ دیگری که تا این‌جا نوشته‌ای `Send` است — هیچ فیلدی درونش نیست که مانع شود.

</details>

<details>
<summary>این کامپایل می‌شود؟</summary>

```rust
use std::rc::Rc;

struct Session {
    user: String,
    hits: Rc<u32>,
}

fn assert_send<T: Send>() {}
assert_send::<Session>();
```

</details>

<details>
<summary>پاسخ</summary>

نه — `E0277`. `Session` دو فیلد دارد؛ `hits: Rc<u32>` `Send` نیست، و همین یکی کافی است کلِ `Session` را از `Send` بودن بیندازد.

</details>

<details>
<summary>درست یا غلط: <code>RefCell&lt;i32&gt;</code> را می‌شود با <code>move</code> به یک ریسمانِ دیگر داد.</summary>

فکرت را قبل از دیدنِ پاسخ بنویس.

</details>

<details>
<summary>پاسخ</summary>

درست. `RefCell<i32>` خودش `Send` است — فقط یک ریسمان در هر لحظه بهش دسترسی خواهد داشت، پس شمارنده‌ی قرضِ غیراتمی‌اش هرگز هم‌زمان از دو جا لمس نمی‌شود. آنچه نمی‌شود این است که یک `&RefCell<i32>` را بینِ چند ریسمان به اشتراک بگذاری.

</details>

<details>
<summary>درست یا غلط: <code>Mutex&lt;RefCell&lt;i32&gt;&gt;</code> ترکیبی است که واقعاً باید بنویسی وقتی <code>RefCell</code> کافی نیست.</summary>

فکرت را قبل از دیدنِ پاسخ بنویس.

</details>

<details>
<summary>پاسخ</summary>

غلط، و اصلاً لازم هم نیست. `Mutex<T>` خودش دقیقاً همان کارِ `RefCell<T>` را می‌کند — تغییرپذیری از پشتِ ارجاعِ اشتراکی — فقط با ابزاری ایمن‌برایِ‌ریسمان. کافی است `RefCell` را با `Mutex` عوض کنی، نه اینکه یکی را تویِ دیگری بگذاری.

</details>

### تعمیر

هر سه مثالِ خراب را درست کن:

۱. `examples/03-struct-with-rc-not-send-broken.rs` را با عوض‌کردنِ فیلدِ `count` از `Rc<i32>` به `Arc<i32>` درست کن.
۲. `examples/04-rc-refcell-thread-spawn-broken.rs` را با عوض‌کردنِ `Rc<RefCell<i32>>` به `Arc<Mutex<i32>>` درست کن (و `.borrow_mut()` را به `.lock().unwrap()`).
۳. `examples/05-arc-refcell-not-sync-broken.rs` را با عوض‌کردنِ `RefCell` به `Mutex` درست کن — `Arc` را دست‌نخورده بگذار، چون از اول درست بود.

### پیاده‌سازی

هر سه چیز را تویِ `src/lib.rs` پیاده‌سازی کن:

- تابعِ `label_from_thread`
- ساختارِ `SharedCounter` و سه متدش (`new`، `increment`، `value`)
- تابعِ `fan_out_increments`

مشخصاتِ دقیقِ هرکدام همان کامنتِ مستندسازِ بالایِ خودشان است.

### بساز

یک نوعِ کوچک از دامنه‌ای که خودت انتخاب می‌کنی بساز که قرار است بینِ چند ریسمان به اشتراک برود — یک کشِ کوچک، یک صفِ رویدادها، یک شمارنده‌ی آماری. با `Arc<Mutex<T>>` بسازش. بعد، تویِ یک کامنت، بگو اگر به‌جایش `Rc<RefCell<T>>` انتخاب می‌کردی، دقیقاً کدام فیلد اولین‌بار خطا می‌داد و چرا.

### چالش (اختیاری)

پیش‌بینی کن — بعد چک کن: `Mutex<Rc<i32>>` (نه `Arc<Rc<i32>>`، خودِ `Mutex`) `Send` است؟ `Sync` است؟ با `assert_send`/`assert_sync` (مثلِ مثالِ ۰۲) خودت بسازش و ببین حدست درست بود یا نه. یک جمله بنویس که چرا فقط پیچیدنِ یک نوع در `Mutex` تضمینی رویِ `Send`‌بودنِ خودِ `T` نمی‌دهد.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| `Send` | نوعی که مالکیتِ یک مقدارش را می‌شود امن به ریسمانِ دیگر داد | هرچیزی که با `move` به `thread::spawn` می‌رود |
| `Sync` | نوعی که `&T`اش را می‌شود امن بینِ چند ریسمان به اشتراک گذاشت | هرچیزی که پشتِ یک `Arc` بینِ ریسمان‌ها می‌چرخد |
| صفتِ خودکار (auto trait) | صفتی بدونِ متد که کامپایلر خودش از رویِ فیلدها حسابش می‌کند | `Send`، `Sync` — بدونِ `impl` و بدونِ `derive` |
| صفتِ نشانه‌گذار (marker trait) | صفتی که فقط یک واقعیت را رویِ نوع علامت می‌زند، هیچ متدی حمل نمی‌کند | همان `Send`/`Sync` |
| `T: Sync` ⇔ `&T: Send` | رابطه‌ی دقیقِ بینِ دو صفت | فهمیدنِ اینکه چرا `RefCell` را می‌شود جابه‌جا کرد ولی نه به‌اشتراک گذاشت |

### الان می‌دانی

- `Send` یعنی مالکیت می‌تواند امن به یک ریسمانِ دیگر برود؛ `Sync` یعنی `&T` می‌تواند امن بینِ چند ریسمان به اشتراک برود — و رابطه‌شان دقیق است: `T` وقتی `Sync` است که `&T` خودش `Send` باشد.
- `Send`/`Sync` صفتِ خودکارند: نه `impl` می‌نویسی، نه `derive` می‌خواهی — کامپایلر از رویِ فیلدهایِ نوعت حسابش می‌کند؛ یک فیلدِ غیرِ`Send` کافی است کلِ ساختار غیرِ`Send` بشود.
- `Rc<T>` نه `Send` است نه `Sync`، چون شمارنده‌اش یک عددِ معمولی و غیراتمی است — دو ریسمانی که هم‌زمان رویش کار کنند، مسابقه‌ی داده می‌سازند. `Arc<T>` همان مشکل را با شمارنده‌ای اتمی حل می‌کند.
- `RefCell<T>` را می‌شود کامل به یک ریسمانِ دیگر داد (`Send`، اگر `T` هم باشد) ولی نمی‌شود `&RefCell<T>` را بینِ چند ریسمان به اشتراک گذاشت (نه `Sync`) — شمارنده‌ی قرضش هم غیراتمی است.
- `Mutex<T>` هم `Send` است هم `Sync`، فقط به شرطِ `T: Send` — چون قفلش، نه شمارنده‌اش، ایمنیِ هم‌زمانی را تضمین می‌کند. برایِ همین `Arc<Mutex<T>>` جواب می‌دهد و `Rc<RefCell<T>>` نه.
- اشاره‌گرهایِ خام به‌طورِ پیش‌فرض نه `Send`اند نه `Sync`؛ و دادنِ دستیِ این دو صفت به یک نوع، با `unsafe impl`، ممکن است ولی مالِ درسِ دیگری است.

### بعداً کامل‌تر می‌بینی

- **کارِ واقعی با اشاره‌گرهایِ خام، و بقیه‌ی عملیاتِ `unsafe`ای که این درس ازشان دوری کرد** — [۲.۱۰.۴ — بررسیِ Rustِ ناایمن](../../10-rust-toolbox/04-unsafe-for-real/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا `Send` و `Sync` دو صفتِ جدا هستند، نه یکی؟ چه نوعی می‌شناسی که یکی را دارد و دیگری را نه؟
- چرا کامپایلر اجازه نمی‌دهد خودت `impl Send for MyType {}` بنویسی، همان‌طور که برایِ `Display` یا `Clone` می‌نویسی؟
- رابطه‌ی `T: Sync` ⇔ `&T: Send` را با کلماتِ خودت، بدونِ نمادِ ریاضی، توضیح بده.
- چرا `Rc<T>` نه `Send` است نه `Sync`، ولی `RefCell<T>` فقط یکی‌شان را ندارد؟ فرقِ دقیقشان چیست؟
- `Mutex<T>` چطور به `T`ای که خودش `Sync` نیست اجازه می‌دهد `Sync` باشد؟ اگر بخواهی این را به یک هم‌کلاسی توضیح بدهی، چه می‌گویی؟

---

## بیشتر

- [Rustonomicon — Send and Sync](https://doc.rust-lang.org/nomicon/send-and-sync.html) — همین موضوع، با جزئیاتِ رسمی‌تر، شاملِ اینکه دستی `unsafe impl` چطور نوشته می‌شود.
- [مستنداتِ `std::marker::Send`](https://doc.rust-lang.org/std/marker/trait.Send.html)
- [مستنداتِ `std::marker::Sync`](https://doc.rust-lang.org/std/marker/trait.Sync.html)

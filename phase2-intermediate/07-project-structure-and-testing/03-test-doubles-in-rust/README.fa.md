# ۲.۷.۳ — بدل‌های تست در Rust، و چرا به‌ندرت به فریم‌ورکِ mock احتیاج داری

## در یک نگاه

بعد از این درس می‌توانی:

- چهار شکلِ رایجِ بدلِ تست — stub، fake، spy و mock — را از هم جدا کنی و اسمِ درست را رویِ هرکدام بگذاری.
- برایِ یک وابستگیِ واقعی (یک `Notifier`، یک ساعت، یک مخزنِ داده) یک صفت بنویسی، یک پیاده‌سازیِ واقعی و یک fakeِ دستی برایش بسازی، و بینِ کراندِ جنریک و شیءِ صفتی برایِ تزریقش انتخاب کنی.
- توضیح بدهی چرا کامپایلر خودش جلویِ یک fakeِ جامانده از امضایِ واقعیِ صفت را می‌گیرد — بدونِ هیچ فریم‌ورکِ اضافه‌ای — و کجا این الگو دیگر کافی نیست.

**زمان:** حدود ۵۵ دقیقه · **پیش‌نیاز:**
[۲.۷.۲ — تست‌هایِ واحد، یکپارچگی و مستندات](../02-unit-integration-doc-tests/README.fa.md)،
[۲.۳.۷ — ارسالِ ایستا در برابرِ پویا، و ایمنیِ شیء](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.fa.md)

---

## چرا اهمیت دارد

۲.۷.۲ سه‌جور تست به‌ات یاد داد — واحد، یکپارچگی، مستندات — و هر سه‌شان رویِ یک تابعِ کاملاً خالص تمرین شدند: `celsius_to_fahrenheit` نه فایلی باز می‌کرد، نه به شبکه سر می‌زد، نه به ساعتِ سیستم نگاه می‌کرد. تستش کردن، هرچقدر هم که صدایش بزنی، همیشه یک‌جور بود.

کدِ واقعی این‌طور نیست. یک سرویسِ ارسالِ سفارش باید مشتری را خبر کند؛ یک هندلر باید از دیتابیس بخواند؛ یک جاب باید یک تایم‌اوتِ واقعی را بسنجد. اگر تست‌هایت مجبور باشند از میانِ همین وابستگی‌هایِ واقعی رد شوند، یا کند و شکننده می‌شوند (یک ایمیلِ واقعی، یک اتصالِ واقعیِ دیتابیس)، یا اصلاً نمی‌توانی مسیرهایِ خطا را امتحان کنی — چطور یک سرویسِ ایمیلِ واقعی را وادار کنی *دقیقاً همین الان* شکست بخورد؟

در پایتون یا جاوااسکریپت، جوابِ رایج یک فریم‌ورکِ mock است: چیزی که رفتارِ یک متد را در زمانِ اجرا، رویِ یک شیءِ واقعی، جایگزین می‌کند. Rust یک راهِ دیگر دارد — راهی که از همان صفت‌ها و همان انتخابِ ایستا/پویایِ ۲.۳.۷ می‌آید، نه از یک کتابخانه‌ی جداگانه. این درس همان راه را نشانت می‌دهد: کِی کافی است، و کجا واقعاً کم می‌آورد.

---

## مفهوم

### چهار نوع بدلِ تست: stub، fake، spy، mock

وقتی کدی که می‌خواهی تست کنی به یک چیزِ کند، غیرقابل‌پیش‌بینی یا بیرون از کنترلت وابسته است، به‌جایِ آن چیزِ واقعی یک جایگزینِ کوچک‌تر و قابلِ‌کنترل می‌گذاری. این چتر «بدلِ تست» (test double) نام دارد، و زیرِ همین یک چتر چهار شکلِ رایج هست که ارزش دارد از هم جدایشان کنی:

- **استاب (stub)** — فقط یک جوابِ از پیش‌تعیین‌شده برمی‌گرداند. نه منطقی دارد، نه چیزی ثبت می‌کند.
- **فیک (fake)** — رفتارِ واقعی و کاری دارد، فقط ساده‌شده — مثلِ یک مخزنِ درون‌حافظه‌ای به‌جایِ یک پایگاه‌داده‌ی واقعی.
- **جاسوس (spy)** — یادش می‌ماند چه چیزی، با چه آرگومان‌هایی، چند بار صدا زده شده — تا تست بعداً رویش assert کند.
- **mock** — از پیش با یک سری انتظار بارگذاری شده — کدام فراخوانی‌ها، چند بار، به چه ترتیبی — و اگر آن انتظارها برآورده نشوند، خودش تست را شکست می‌دهد.

خط بینِ این چهارتا همیشه تیز نیست — خیلی از دوبل‌هایِ واقعی که می‌نویسی چند نقش را هم‌زمان بازی می‌کنند (همین امروز یکی از همین‌ها را می‌سازی). آنچه واقعاً مهم است این نیست که دقیق‌ترین اسم را رویِ یک دوبلِ خاص بگذاری؛ مهم این است که وقتی یکی از همکارهایت می‌گوید «رویِ این تست، `Notifier` را mock کردم»، دقیقاً بدانی منظورش چیست.

### صفتِ `Notifier`: مرزی که در تست عوضش می‌کنیم

برایِ اینکه این ایده‌ها دستت بیاید، یک مثالِ واحد را تا آخرِ درس دنبال می‌کنیم: یک سرویسِ ارسالِ سفارش که، وقتی یک سفارش می‌رود، باید مشتری را خبر کند. آن «خبر کردن» دقیقاً همان چیزی است که در تست نمی‌خواهی واقعی باشد — یک ایمیلِ واقعی نه سریع است، نه رایگان، نه چیزی که بشود رویش یک شکستِ عمدی را قابلِ‌تکرار امتحان کرد.

جوابِ Rust این نیست که یک فریم‌ورک بیاوری تا رفتارِ یک تابع را در زمانِ اجرا عوض کند — دقیقاً همان‌طور که ۲.۳.۷ نشانت داد، مرزِ بینِ «کدام پیاده‌سازی» و «چه‌کاری» یک صفت است:

```rust
trait Notifier {
    fn notify(&mut self, to: &str, message: &str) -> Result<(), NotifyError>;
}

struct EmailNotifier;

impl Notifier for EmailNotifier {
    fn notify(&mut self, to: &str, message: &str) -> Result<(), NotifyError> {
        println!("email to {to}: {message}");
        Ok(())
    }
}
```

هیچ‌چیزِ تازه‌ای در این کد نیست — دقیقاً همان صفت و همان `impl` که ۲.۳.۱ به‌ات داد. `EmailNotifier` همان چیزی است که در پروداکشن واقعاً اجرا می‌شود؛ به‌جایِ فراخوانیِ واقعیِ یک API، همین‌جا فقط چاپ می‌کند تا رفتارش قابلِ‌مشاهده بماند.

```rust
let mut notifier = EmailNotifier;
match notifier.notify("ren@example.com", "your order has shipped") {
    Ok(()) => println!("sent"),
    Err(NotifyError(reason)) => println!("failed: {reason}"),
}
```

```text
email to ren@example.com: your order has shipped
sent
```

### `FakeNotifier`: یک پیاده‌سازیِ دومِ دستی، فقط برایِ تست

`Notifier` فقط یک صفت است — هر نوعی که این یک متد را داشته باشد، یک `Notifier` معتبر است. پس یک نوعِ دومِ دستی بنویس، فقط برایِ تست، که هیچ ایمیلی نمی‌فرستد:

```rust
#[derive(Default)]
struct FakeNotifier {
    calls: Vec<(String, String)>,
    fail_with: Option<String>,
}

impl Notifier for FakeNotifier {
    fn notify(&mut self, to: &str, message: &str) -> Result<(), NotifyError> {
        if let Some(reason) = &self.fail_with {
            return Err(NotifyError(reason.clone()));
        }
        self.calls.push((to.to_string(), message.to_string()));
        Ok(())
    }
}
```

همین یک نوع هر دو نقشِ بالا را بازی می‌کند. وقتی `fail_with` چیزی نیست، `FakeNotifier` یک **جاسوس (spy)** است — هر فراخوانی را دقیقاً همان‌طور که آمده در `calls` ثبت می‌کند:

```rust
let mut spy = FakeNotifier::default();
spy.notify("ren@example.com", "shipped").unwrap();
spy.notify("aoi@example.com", "shipped").unwrap();
println!("spy.calls: {:?}", spy.calls);
```

```text
spy.calls: [("ren@example.com", "shipped"), ("aoi@example.com", "shipped")]
```

و وقتی `fail_with` مقدار دارد، همان نوع یک **استاب (stub)** می‌شود — یک شکستِ از پیش‌تعیین‌شده برمی‌گرداند، دقیقاً همان چیزی که ساختنش با `EmailNotifier` واقعی سخت یا غیرِممکن است:

```rust
let mut stub = FakeNotifier {
    fail_with: Some("simulated provider outage".to_string()),
    ..Default::default()
};
match stub.notify("ren@example.com", "shipped") {
    Ok(()) => println!("unexpected success"),
    Err(NotifyError(reason)) => println!("stub failed with: {reason}"),
}
```

```text
stub failed with: simulated provider outage
```

یک نکته‌ی سازمانی: `FakeNotifier` اینجا یک نوعِ معمولی است، بدونِ هیچ محدودیتی، تا همین مثال‌ها بتوانند مستقیم اجرایش کنند و خروجی‌اش را نشانت بدهند. تویِ یک کریتِ واقعی که فقط تست‌هایِ خودش این نوع را می‌سازند، جایِ طبیعی‌اش این است که کاملاً داخلِ همان بلوکِ `#[cfg(test)] mod tests { ... }`ای زندگی کند که assertionهایت هم آن‌جایند — همان‌جا که تمرینِ امروز هم می‌گذاردش. آن‌وقت هرگز داخلِ باینریِ نهایی کامپایل نمی‌شود، و لازم هم نیست `pub` باشد.

### تزریق با کراندِ جنریک: یک `ShippingService`، دو `Notifier`

حالا چیزی بساز که واقعاً از `Notifier` استفاده کند — یک سرویسِ ارسالِ سفارش که وقتی سفارشی می‌رود مشتری را خبر می‌کند، و از ارسالِ دوباره‌ی همان سفارش جلوگیری می‌کند:

```rust
enum ShipError {
    AlreadyShipped,
    NotifyFailed(NotifyError),
}

struct ShippingService<N> {
    notifier: N,
    shipped: HashSet<String>,
}
```

`ShippingService` رویِ نوعِ نوتیفایرش جنریک است — همین یک نکته، کلِ درس است. حالا متدهایش:

```rust
impl<N: Notifier> ShippingService<N> {
    fn new(notifier: N) -> Self {
        Self {
            notifier,
            shipped: HashSet::new(),
        }
    }
}
```

```rust
impl<N: Notifier> ShippingService<N> {
    fn ship_order(&mut self, order_id: &str, customer_email: &str) -> Result<(), ShipError> {
        if self.shipped.contains(order_id) {
            return Err(ShipError::AlreadyShipped);
        }
        let message = format!("Your order {order_id} has shipped!");
        self.notifier
            .notify(customer_email, &message)
            .map_err(ShipError::NotifyFailed)?;
        self.shipped.insert(order_id.to_string());
        Ok(())
    }
}
```

نه `ShippingService` و نه `ship_order`، هیچ‌کدام هیچ‌جا اسمِ `EmailNotifier` یا `FakeNotifier` را نمی‌آورند — فقط `N: Notifier` را می‌شناسند. **تزریق (injection)** دقیقاً همین‌جا اتفاق می‌افتد: نه در یک فایلِ کانفیگ، نه در یک فریم‌ورکِ DI، بلکه در همان لحظه‌ای که `::new(...)` را صدا می‌زنی و تصمیم می‌گیری کدام نوعِ ملموس را بدهی.

```rust
let mut live = ShippingService::new(EmailNotifier);
match live.ship_order("A1", "ren@example.com") {
    Ok(()) => println!("live.ship_order: sent"),
    Err(_) => println!("live.ship_order: failed"),
}
```

```text
email to ren@example.com: Your order A1 has shipped!
live.ship_order: sent
```

همان `ShippingService`، همان `ship_order` — این‌بار با `N = FakeNotifier`:

```rust
let mut test_service = ShippingService::new(FakeNotifier::default());
test_service.ship_order("A1", "ren@example.com").unwrap();
match test_service.ship_order("A1", "ren@example.com") {
    Ok(()) => println!("unexpected: shipped A1 twice"),
    Err(ShipError::AlreadyShipped) => println!("second ship_order: already shipped"),
    Err(ShipError::NotifyFailed(NotifyError(reason))) => {
        println!("second ship_order: notify failed: {reason}")
    }
}
println!("notifier.calls: {:?}", test_service.notifier.calls);
```

```text
second ship_order: already shipped
notifier.calls: [("ren@example.com", "Your order A1 has shipped!")]
```

بارِ دومِ `ship_order`، پیش از آنکه اصلاً به `notifier` سر بزند، با `AlreadyShipped` برگشت — و `notifier.calls` هم دقیقاً همین را ثابت می‌کند: فقط یک فراخوانی ثبت شده، نه دو تا.

```senpai-visual
{"kind":"concept","labels":["ShippingService::new(notifier)","تولید: N = EmailNotifier","تست: N = FakeNotifier","ship_order() — یک متد، دو نمونه‌سازی","کامپایلر impl را انتخاب می‌کند، نه یک پرچمِ زمانِ‌اجرا"]}
```

### تزریق با شیءِ صفتی: وقتی به ناهمگونی نیاز داری

کراندِ جنریکِ بالا وقتی کافی است که یک `ShippingService` دقیقاً یک نوعِ نوتیفایرِ ثابت دارد. ولی گاهی باید چند `Notifier`ِ *متفاوت* را در یک مجموعه‌ی واحد نگه داری — دقیقاً همان مسئله‌ی ناهمگونی که ۲.۳.۷ با `Vec<Box<dyn Summarize>>` نشانت داد. همان حرکت، اینجا هم کار می‌کند:

```rust
fn notify_all(notifiers: &mut [Box<dyn Notifier>], to: &str, message: &str) -> usize {
    let mut succeeded = 0;
    for notifier in notifiers.iter_mut() {
        if notifier.notify(to, message).is_ok() {
            succeeded += 1;
        }
    }
    succeeded
}
```

```rust
let mut always_fails = FakeNotifier {
    fail_with: Some("simulated outage".to_string()),
    ..Default::default()
};
match always_fails.notify("ren@example.com", "shipped") {
    Ok(()) => println!("unexpected success"),
    Err(NotifyError(reason)) => println!("standalone failure: {reason}"),
}
```

```text
standalone failure: simulated outage
```

حالا همین `always_fails` را کنارِ یک `EmailNotifier` واقعی و یک `FakeNotifier` سالم، تویِ یک `Vec` واحد بگذار:

```rust
let mut fleet: Vec<Box<dyn Notifier>> = vec![
    Box::new(EmailNotifier),
    Box::new(FakeNotifier::default()),
    Box::new(always_fails),
];

let succeeded = notify_all(&mut fleet, "ren@example.com", "shipped");
println!("succeeded: {succeeded} of {}", fleet.len());
```

```text
email to ren@example.com: shipped
succeeded: 2 of 3
```

یک `Vec`، سه نوعِ واقعاً متفاوتِ زیرش — چیزی که هیچ `ShippingService<N>`ِ جنریک با یک `N`ِ ثابت نمی‌توانست نگه دارد. قاعده‌اش همان قاعده‌ی ۲.۳.۷ است: پیش‌فرض، کراندِ جنریک؛ سراغِ `dyn Trait` برو وقتی واقعاً به ترکیبی از چند نوع، تویِ یک متغیر یا کالکشن، نیاز داری.

### چرا این جایِ بیشترِ کاری را می‌گیرد که یک فریم‌ورکِ mock انجام می‌دهد

هیچ‌جایِ کدِ بالا یک کتابخانه‌ی جداگانه، یک دکوریتور، یا یک مرحله‌ی «حالا این متد را patch کن» نبود. دو چیزِ ساده اتفاق افتاد:

۱. **کدامیک اجرا می‌شود، با نوشتنِ کدِ معمولی تصمیم گرفته شد.** `ShippingService::new(EmailNotifier)` در برابرِ `ShippingService::new(FakeNotifier::default())` — همین. هیچ متغیرِ محیطی، هیچ رجیستریِ سراسری، هیچ چیزِ پنهانی در کار نبود.
۲. **کامپایلر، نه یک لایه‌ی رهگیریِ زمانِ‌اجرا، تصمیم می‌گیرد کدام `notify` واقعاً صدا زده می‌شود.** `FakeNotifier` باید دقیقاً همان صفتی را پیاده کند که `EmailNotifier` پیاده کرده — همان امضا، همان تعدادِ آرگومان، همان نوعِ خروجی.

همین نکته‌ی دوم چیزی به‌ات می‌دهد که یک mockِ زمانِ‌اجرا در یک زبانِ پویا نمی‌تواند قول بدهد: اگر یک روز امضایِ `Notifier::notify` عوض شود — یک پارامتر اضافه شود، نوعِ خروجی عوض شود — و `FakeNotifier` را به‌روز نکنی، کدت اصلاً کامپایل نمی‌شود. نه یک تستِ کم‌پوشش که این مسیر را هیچ‌وقت اجرا نمی‌کند، نه یک شکستِ غافلگیرکننده در پروداکشن شش ماه بعد — همین الان، سرِ `cargo build`. نمونه‌ی واقعی‌اش را در «خطاهایی که خواهی دید» می‌بینی.

### صداقت درباره‌ی محدودیت‌ها: این الگو کجا کم می‌آورد

این الگو برایِ یک‌دو وابستگیِ قابلِ‌تعویض — یک `Notifier`، شاید یک ساعت، شاید یک مخزنِ داده — عالی کار می‌کند. ولی جایگزینِ *همه‌ی* کاری که یک فریم‌ورکِ mock در زبان‌هایِ دیگر انجام می‌دهد نیست:

- **تأییدِ ترتیب یا تعدادِ دقیقِ فراخوانی، وقتی چند وابستگیِ بی‌ربط درگیرند، دستی می‌شود.** `FakeNotifier.calls` به‌راحتی جواب می‌دهد «دقیقاً همین یک بار، با همین آرگومان‌ها صدا زده شدم» — همان چیزی که تمرین‌هایِ امروز با `assert_eq!` رویِ همین `Vec` چک می‌کنند. ولی «Notifier باید *بعدِ* ذخیره‌شدنِ سفارش در دیتابیس صدا زده شود» یک ترتیب بینِ دو بدلِ جداست، و Rust هیچ ابزارِ آماده‌ای برایِ این نمی‌دهد — باید خودت یک لاگِ رویدادِ مشترک بسازی. کریت‌هایی مثلِ `mockall` دقیقاً برایِ همین نیازِ سنگین‌تر وجود دارند؛ این درس سراغشان نمی‌رود، چون بیشترِ کدی که می‌نویسی هیچ‌وقت به آن سطح نیاز ندارد.
- **برایِ چیزی که واقعاً به‌اندازه‌ی کافی ساده است که اجرایش کنی، اغلب یک تستِ یکپارچگیِ واقعی (۲.۷.۲) بهتر از یک fakeِ پیچیده است.** یک fake همیشه یک مدلِ ساده‌شده از واقعیت است، و می‌تواند از رفتارِ واقعیِ چیزی که جایگزینش شده عقب بیفتد؛ یک پایگاه‌داده‌ی محلی یا صفِ درون‌حافظه‌ای که واقعاً اجرایش می‌کنی، این خطر را کلاً کنار می‌زند.

و دو نکته‌ی کوچکِ دیگر، فقط برایِ اینکه بدانی این الگو کجاها ادامه پیدا می‌کند: اگر یک بدل باید بینِ چند ریسه مشترک باشد، همین‌جا `Mutex`/`Arc` می‌آید. و اگر `Notifier`ِ واقعی خودش باید async باشد (یک فراخوانیِ واقعیِ شبکه)، متدِ async داخلِ یک صفت دقیقاً همان دیوارِ ایمنیِ شیء را پیش می‌کشد که ۲.۳.۷ نامش را گذاشت.

---

## دست‌به‌کد

```sh
cargo run -p p2-07-03-test-doubles-in-rust --example 01-notifier-trait-and-real-impl
cargo run -p p2-07-03-test-doubles-in-rust --example 02-fake-notifier-spy-and-stub
cargo run -p p2-07-03-test-doubles-in-rust --example 03-generic-injection
cargo run -p p2-07-03-test-doubles-in-rust --example 04-trait-object-injection
```

بعد سه‌تای خراب:

```sh
cargo run -p p2-07-03-test-doubles-in-rust --example 05-signature-drift-broken --features broken
cargo run -p p2-07-03-test-doubles-in-rust --example 06-missing-trait-impl-broken --features broken
cargo run -p p2-07-03-test-doubles-in-rust --example 07-use-after-move-broken --features broken
```

بعد این‌ها را امتحان کن:

۱. در `02-fake-notifier-spy-and-stub.rs`، یک سومین `notify` رویِ `spy` صدا بزن. `calls` چند عضو دارد؟
۲. در `03-generic-injection.rs`، `ship_order` را برایِ یک `order_id` دیگر (مثلاً `"A2"`) هم صدا بزن. `notifier.calls` چه چیزی نشان می‌دهد؟
۳. در `04-trait-object-injection.rs`، یک `FakeNotifier` سالمِ دیگر به `fleet` اضافه کن. `succeeded` الان چند است؟

---

## خطاهایی که خواهی دید

### `E0050` — یک fakeِ جامانده از امضایِ واقعیِ صفت

```text
error[E0050]: method `notify` has 2 parameters but the declaration in trait `Notifier::notify` has 3
  --> phase2-intermediate\07-project-structure-and-testing\03-test-doubles-in-rust\examples\05-signature-drift-broken.rs:21:15
   |
12 |     fn notify(&mut self, to: &str, message: &str) -> Result<(), NotifyError>;
   |               ---------------------------------- trait requires 3 parameters
...
21 |     fn notify(&mut self, to: &str) -> Result<(), NotifyError> {
   |               ^^^^^^^^^^^^^^^^^^^ expected 3 parameters, found 2

For more information about this error, try `rustc --explain E0050`.
```

**کامپایلر به چه اعتراض دارد:** `Notifier::notify` سه پارامتر می‌خواهد (`self`، `to`، `message`). `StaleFake` امضایِ قدیمی‌تری دارد — انگار پیش از آنکه `message` به صفت اضافه شود نوشته شده، و کسی به‌روزش نکرده.

**راه‌حل:** پارامترِ جامانده را اضافه کن تا امضا دقیقاً با صفت جور دربیاید:

```rust
impl Notifier for StaleFake {
    fn notify(&mut self, to: &str, message: &str) -> Result<(), NotifyError> {
        println!("to: {to}, message: {message}");
        Ok(())
    }
}
```

**چرا این راه‌حل است:** این دقیقاً همان چیزی است که «چرا این جایِ فریم‌ورکِ mock را می‌گیرد» ادعا کرد: `StaleFake` یک پیاده‌سازیِ واقعیِ صفت است، نه یک شیءِ کاملاً آزادِ زمانِ‌اجرا — کامپایلر دقیقاً همان قانونی را رویش اعمال می‌کند که رویِ `EmailNotifier` اعمال کرد. اگر این fake یک mockِ پایتونی بود، این drift تا اولین باری که تستی دقیقاً همین متد را با همین آرگومان‌ها صدا می‌زد، پنهان می‌ماند.

### `E0277` — تزریقِ نوعی که اصلاً `Notifier` نیست

```text
error[E0277]: the trait bound `SilentLogger: Notifier` is not satisfied
  --> phase2-intermediate\07-project-structure-and-testing\03-test-doubles-in-rust\examples\06-missing-trait-impl-broken.rs:35:41
   |
35 |     let _service = ShippingService::new(SilentLogger);
   |                    -------------------- ^^^^^^^^^^^^ unsatisfied trait bound
   |                    |
   |                    required by a bound introduced by this call
   |
help: the trait `Notifier` is not implemented for `SilentLogger`
  --> phase2-intermediate\07-project-structure-and-testing\03-test-doubles-in-rust\examples\06-missing-trait-impl-broken.rs:32:1
   |
32 | struct SilentLogger;
   | ^^^^^^^^^^^^^^^^^^^
help: this trait has no implementations, consider adding one
  --> phase2-intermediate\07-project-structure-and-testing\03-test-doubles-in-rust\examples\06-missing-trait-impl-broken.rs:11:1
   |
11 | trait Notifier {
   | ^^^^^^^^^^^^^^
note: required by a bound in `ShippingService::<N>::new`
  --> phase2-intermediate\07-project-structure-and-testing\03-test-doubles-in-rust\examples\06-missing-trait-impl-broken.rs:23:9
   |
23 | impl<N: Notifier> ShippingService<N> {
   |         ^^^^^^^^ required by this bound in `ShippingService::<N>::new`
24 |     fn new(notifier: N) -> Self {
   |        --- required by a bound in this associated function

For more information about this error, try `rustc --explain E0277`.
```

**کامپایلر به چه اعتراض دارد:** `ShippingService::new` هر `N: Notifier`ای را قبول می‌کند. `SilentLogger` هیچ‌وقت `impl Notifier for SilentLogger` نگرفته — شاید چون شکلش شبیهِ یک نوتیفایر به‌نظر می‌رسید، ولی هیچ‌کس واقعاً `impl` را ننوشت.

**راه‌حل:** یا `impl Notifier for SilentLogger` را بنویس، یا یک نوعی بده که واقعاً `Notifier` را پیاده کرده — `EmailNotifier` یا `FakeNotifier`.

**چرا این راه‌حل است:** این‌جا هیچ تزریقی اتفاق نمی‌افتد، چون هیچ substitutionای برایِ کامپایلر معنی ندارد — `SilentLogger` از نظرِ نوع، به همان اندازه نامربوط به `Notifier` است که یک `bool` به یک `String`. برخلافِ یک mockِ زمانِ‌اجرا که تا لحظه‌ی صدا زدنِ یک متدِ ناموجود چیزی نمی‌گوید، اینجا حتی یک خط کد هم اجرا نشد.

### `E0382` — استفاده از یک fake بعد از اینکه مالکیتش رفته

```text
error[E0382]: borrow of moved value: `fake`
  --> phase2-intermediate\07-project-structure-and-testing\03-test-doubles-in-rust\examples\07-use-after-move-broken.rs:60:38
   |
54 |     let fake = FakeNotifier::default();
   |         ---- move occurs because `fake` has type `FakeNotifier`, which does not implement the `Copy` trait
55 |     let mut service = ShippingService::new(fake);
   |                                            ---- value moved here
...
60 |     println!("calls recorded: {:?}", fake.calls);
   |                                      ^^^^^^^^^^ value borrowed here after move
   |
note: consider changing this parameter type in method `new` to borrow instead if owning the value isn't necessary
  --> phase2-intermediate\07-project-structure-and-testing\03-test-doubles-in-rust\examples\07-use-after-move-broken.rs:38:22
   |
38 |     fn new(notifier: N) -> Self {
   |        ---           ^ this parameter takes ownership of the value
   |        |
   |        in this method
note: if `FakeNotifier` implemented `Clone`, you could clone the value
  --> phase2-intermediate\07-project-structure-and-testing\03-test-doubles-in-rust\examples\07-use-after-move-broken.rs:21:1
   |
21 | struct FakeNotifier {
   | ^^^^^^^^^^^^^^^^^^^ consider implementing `Clone` for this type
...
55 |     let mut service = ShippingService::new(fake);
   |                                            ---- you could clone this value

For more information about this error, try `rustc --explain E0382`.
```

**کامپایلر به چه اعتراض دارد:** `ShippingService::new(fake)` مالکیتِ `fake` را گرفت. متغیرِ محلیِ `fake` بعدِ آن خط دیگر معتبر نیست — دقیقاً همان قانونِ انتقالی که فازِ ۱.۲ یادت داد، اینجا هم هیچ استثنایی برایِ «ولی این دارد برایِ تست استفاده می‌شود» قائل نمی‌شود.

**راه‌حل:** به‌جایِ نگه‌داشتنِ `fake`، از راهی که `ShippingService` خودش برایِ این کار گذاشته استفاده کن — متدِ `notifier()`:

```rust
println!("calls recorded: {:?}", service.notifier().calls);
```

**چرا این راه‌حل است:** در پایتون، یک `Mock()` که پاسش می‌دهی همان شیءای است که اسمش را نگه داشته‌ای — چک کردنش بعداً کارِ رایگانی است، چون همه‌چیز از رویِ ارجاع کار می‌کند. Rust این میان‌بر را ندارد: `ShippingService::new` مالکیتِ `notifier` را می‌گیرد، پس نامِ محلی‌ات از دست می‌رود. `notifier()` دقیقاً برایِ همین لحظه نوشته شده — یک راهِ رسمی برایِ اینکه از خودِ سرویس بپرسی «الان چه چیزی داخلت است؟» به‌جایِ اینکه به یک دستگیره‌ی جداگانه تکیه کنی.

---

## تمرین

### گرم‌کردن

<details>
<summary>بعدِ این کد، <code>fake.calls</code> چه چیزی دارد؟</summary>

```rust
let mut fake = FakeNotifier::default();
fake.notify("ren@example.com", "shipped").unwrap();
fake.notify("aoi@example.com", "shipped").unwrap();
```

</details>

<details>
<summary>پاسخ</summary>

```text
[("ren@example.com", "shipped"), ("aoi@example.com", "shipped")]
```

هیچ‌کدام از این دو فراخوانی شکست نخورد (`fail_with` مقدار ندارد)، پس هر دو تویِ `calls` به همان ترتیبی که آمدند ثبت شدند.

</details>

<details>
<summary>این <code>notify</code> چه چیزی برمی‌گرداند؟</summary>

```rust
let mut fake = FakeNotifier {
    fail_with: Some("outage".to_string()),
    ..Default::default()
};
fake.notify("ren@example.com", "shipped")
```

</details>

<details>
<summary>پاسخ</summary>

```text
Err(NotifyError("outage".to_string()))
```

`fail_with` مقدار دارد، پس `notify` بدونِ اینکه چیزی به `calls` اضافه کند، همان دلیل را در یک `NotifyError` برمی‌گرداند.

</details>

<details>
<summary>این کامپایل می‌شود؟</summary>

```rust
struct SilentLogger;

let service = ShippingService::new(SilentLogger);
```

</details>

<details>
<summary>پاسخ</summary>

نه — `E0277`. `SilentLogger` هیچ‌وقت `impl Notifier for SilentLogger` نگرفته، و `ShippingService::new` هر `N`ای که می‌گیرد باید `Notifier` باشد.

</details>

<details>
<summary>این کامپایل می‌شود؟</summary>

```rust
let fake = FakeNotifier::default();
let mut service = ShippingService::new(fake);
println!("{:?}", fake.calls);
```

</details>

<details>
<summary>پاسخ</summary>

نه — `E0382`. `ShippingService::new(fake)` مالکیتِ `fake` را گرفت؛ خطِ بعدی می‌خواهد از یک متغیرِ حرکت‌کرده استفاده کند.

</details>

<details>
<summary>اگر اولین نوتیفایرِ داخلِ <code>notifiers</code> شکست بخورد، آیا <code>notify_all</code> بقیه را هم صدا می‌زند؟</summary>

</details>

<details>
<summary>پاسخ</summary>

بله. `notify_all` رویِ همه‌ی عناصر پیمایش می‌کند و فقط تعدادِ موفق‌ها را می‌شمارد؛ یک فراخوانیِ شکست‌خورده پیمایش را متوقف نمی‌کند.

</details>

### تعمیر

۱. `examples/05-signature-drift-broken.rs` را طوری درست کن که امضایِ `StaleFake::notify` دقیقاً با صفت جور دربیاید.
۲. `examples/06-missing-trait-impl-broken.rs` را طوری درست کن که `SilentLogger` واقعاً یک `Notifier` معتبر باشد — یا `impl Notifier for SilentLogger` بنویس، یا یک نوعِ دیگر به `ShippingService::new` بده.
۳. `examples/07-use-after-move-broken.rs` را با استفاده از متدِ `notifier()` (به‌جایِ نگه‌داشتنِ متغیرِ `fake`) درست کن.

### پیاده‌سازی

سه تکه در `src/lib.rs`:

```sh
cargo test -p p2-07-03-test-doubles-in-rust
```

- `impl Notifier for FakeNotifier` (داخلِ `mod tests`) — خودِ بدل.
- `ShippingService::ship_order` — منطقِ idempotency + تزریق.
- `notify_all` — تزریق با شیءِ صفتی.

مشخصاتِ دقیقِ هرکدام در کامنتِ مستندات، درست بالایِ خودِ تابع است.

### بساز

یک وابستگیِ دیگر، مالِ خودت، انتخاب کن — یک `Clock` (`fn now(&self) -> u64`، یا هرچه)، یک `Repository` سادهِ درون‌حافظه‌ای، هرچه دوست داری. یک صفت برایش بنویس، یک پیاده‌سازیِ «واقعی»، و یک fakeِ دستی که چیزی ثبت می‌کند یا جوابِ از پیش‌آماده برمی‌گرداند. یک تابعِ کوچک بنویس که این وابستگی را — با کراندِ جنریک یا شیءِ صفتی، خودت انتخاب کن — تزریق می‌کند، و یک تست برایش. در یک کامنت بنویس چرا generic یا `dyn` را انتخاب کردی.

### چالش (اختیاری)

«صداقت درباره‌ی محدودیت‌ها» گفت تأییدِ ترتیب بینِ چند بدلِ جدا دستی می‌شود. ثابتش کن: دو `FakeNotifier` بساز که هر دو یک `Rc<Cell<usize>>` مشترک را برایِ شماره‌گذاری به اشتراک می‌گذارند — هر فراخوانی، مقدارِ فعلیِ شمارنده را هم کنارِ `to`/`message` در `calls` ثبت می‌کند و شمارنده را یکی زیاد می‌کند. با این دو نوتیفایر یک `ShippingService` و یک تماسِ جداگانه بساز، و با یک `assert!` ثابت کن کدام‌یک واقعاً زودتر صدا زده شده. هیچ تستِ آماده‌ای برایِ این بخش نیست — خودت باید بنویسی‌اش.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| بدلِ تست (test double) | جایگزینِ یک وابستگیِ واقعی در تست، از همان مرزِ صفتی | هر جا وابستگی کند، غیرقابلِ‌پیش‌بینی یا بیرون از کنترلت باشد |
| استاب (stub) | فقط یک جوابِ از پیش‌تعیین‌شده برمی‌گرداند | آزمایشِ مسیرهایِ خطا که با چیزِ واقعی سخت‌اند |
| فیک (fake) | رفتارِ واقعی و کاری، فقط ساده‌شده | یک مخزنِ درون‌حافظه‌ای به‌جایِ یک پایگاه‌داده |
| جاسوس (spy) | ثبتِ اینکه چه چیزی، با چه آرگومان‌هایی صدا زده شد | assertion بعدِ اجرا |
| mock | از پیش با انتظار بارگذاری شده، خودش تأییدشان می‌کند | نیازهایِ سنگین‌ترِ ترتیب/تعدادِ فراخوانی |

### الان می‌دانی

- چهار شکلِ رایجِ بدلِ تست — stub، fake، spy، mock — و اینکه خیلی از بدل‌هایِ واقعی چند نقش را هم‌زمان بازی می‌کنند.
- چطور یک صفت برایِ یک وابستگی بنویسی، یک پیاده‌سازیِ واقعی و یک fakeِ دستی برایش بسازی، و fake را کجا (`#[cfg(test)]`) نگه داری.
- چطور بینِ تزریق با کراندِ جنریک و تزریق با شیءِ صفتی انتخاب کنی — همان معامله‌ی ۲.۳.۷، این‌بار رویِ یک وابستگیِ واقعی.
- چرا کامپایلر جلویِ یک fakeِ جامانده از امضایِ واقعیِ صفت را می‌گیرد، بدونِ هیچ فریم‌ورکِ اضافه‌ای.
- کجا این الگو کم می‌آورد — تأییدِ ترتیب بینِ چند بدل، و کجا یک تستِ یکپارچگیِ واقعی بهتر از یک fakeِ پیچیده است.

### بعداً کامل‌تر می‌بینی

- **تستِ خاصیت‌محور با `proptest`، تستِ عکس‌برداری با `insta`** — [۲.۷.۴](../04-property-and-snapshot-testing/README.fa.md)
- **نخ‌ها، Mutex، Arc — وقتی یک بدل باید بینِ چند ریسه مشترک باشد** — [۲.۸.۱](../../08-concurrency/01-threads-mutex-arc/README.fa.md)
- **متدهایِ `async` داخلِ یک صفت** — [۲.۹.۴](../../09-async-in-practice/04-async-traits-and-blocking/README.fa.md)

### می‌توانی توضیح بدهی؟

- استاب، فیک، جاسوس و mock را با کلماتِ خودت، هرکدام با یک مثالِ متفاوت از این درس، از هم جدا کن.
- چرا `FakeNotifier` هم استاب است هم جاسوس؟ کدام فیلد این را تصمیم می‌گیرد؟
- چرا `ShippingService::new(fake)` نگه‌داشتنِ متغیرِ `fake` را غیرِممکن می‌کند، و `notifier()` چطور این را دور می‌زند؟
- بینِ کراندِ جنریک و شیءِ صفتی برایِ تزریقِ یک `Notifier`، کِی هرکدام را انتخاب می‌کنی؟
- یک مثالِ واقعی بزن که در آن یک fakeِ دستی کافی نیست و باید سراغِ یک فریم‌ورکِ mock یا یک تستِ یکپارچگی بروی.

---

## بیشتر

- [Martin Fowler — Mocks Aren't Stubs](https://martinfowler.com/articles/mocksArentStubs.html) — همان مقاله‌ای که این تاکسونومی از آن می‌آید.
- [`mockall` روی docs.rs](https://docs.rs/mockall) — یک فریم‌ورکِ mockِ واقعی برایِ Rust، برایِ وقتی این الگو واقعاً کم می‌آورد.
- [کتابِ Rust — Traits: Defining Shared Behavior](https://doc.rust-lang.org/book/ch10-02-traits.html) — یادآوریِ خودِ مکانیزمِ صفت، همان چیزی که کلِ این درس رویش سوار است.

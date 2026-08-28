# راه‌حل — ۲.۷.۳ بدل‌های تست در Rust

```rust
impl<N: Notifier> ShippingService<N> {
    pub fn ship_order(&mut self, order_id: &str, customer_email: &str) -> Result<(), ShipError> {
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

impl Notifier for FakeNotifier {
    fn notify(&mut self, to: &str, message: &str) -> Result<(), NotifyError> {
        if let Some(reason) = &self.fail_with {
            return Err(NotifyError(reason.clone()));
        }
        self.calls.push((to.to_string(), message.to_string()));
        Ok(())
    }
}

pub fn notify_all(notifiers: &mut [Box<dyn Notifier>], to: &str, message: &str) -> usize {
    let mut succeeded = 0;
    for notifier in notifiers.iter_mut() {
        if notifier.notify(to, message).is_ok() {
            succeeded += 1;
        }
    }
    succeeded
}
```

## `ShippingService::ship_order` — ترتیب همان چیزی است که idempotency را درست نگه می‌دارد

```rust
if self.shipped.contains(order_id) {
    return Err(ShipError::AlreadyShipped);
}
let message = format!("Your order {order_id} has shipped!");
self.notifier
    .notify(customer_email, &message)
    .map_err(ShipError::NotifyFailed)?;
self.shipped.insert(order_id.to_string());
```

سه خط، به همین ترتیب: اول فقط *بخوان* (`contains`)، بعد `notify` را صدا بزن، و فقط اگر آن با `?` واقعاً موفق شد، `order_id` را به `shipped` اضافه کن. اگر ترتیب را عوض می‌کردی — اول `insert`، بعد `notify` — یک شکستِ واقعیِ نوتیفای‌کردن، سفارش را همچنان به‌عنوانِ «شده» ثبت می‌کرد، و هیچ تلاشِ دومی هرگز رخ نمی‌داد. تستِ `ship_order_is_retried_after_a_previous_notify_failure` دقیقاً همین را چک می‌کند: دو بارِ پشتِ‌سرِهم صدا زدنِ `ship_order` رویِ یک نوتیفایرِ همیشه‌شکست‌خورده، باید **دوبار** `NotifyFailed` بدهد، نه یک‌بار `NotifyFailed` و یک‌بار `AlreadyShipped`.

`.map_err(ShipError::NotifyFailed)` هم یک نکته‌ی کوچک دارد: `ShipError::NotifyFailed` یک تابعِ ساخت‌وسازِ خودکار است — هر گونه‌ی تاپلیِ یک شمارشی، رایگان، یک تابعِ `NotifyError -> ShipError` هم هست. همان ترکیبِ `?` و `.map_err(...)`ای که ۱.۶.۵ به‌ات داد، فقط این‌بار سازنده‌اش یک بستنِ دستی (closure) نیست، خودِ گونه است.

## `impl Notifier for FakeNotifier` — یک `if let`، دو نقش

```rust
if let Some(reason) = &self.fail_with {
    return Err(NotifyError(reason.clone()));
}
self.calls.push((to.to_string(), message.to_string()));
Ok(())
```

همین یک شرط تصمیم می‌گیرد `FakeNotifier` این‌بار استاب است یا جاسوس — نه دو نوعِ جدا، نه یک فلگِ اضافه‌ی «حالتِ تست». وقتی `fail_with` چیزی ندارد، مسیرِ اول رد می‌شود و کد به مسیرِ ثبت‌کردن می‌رسد؛ وقتی دارد، برمی‌گردد و اصلاً به `calls` دست نمی‌زند. همین ترتیب است که تستِ `fake_notifier_returns_canned_error_when_configured_to_fail` را با `assert!(fake.calls.is_empty())` قانع می‌کند.

## `notify_all` — همان فراخوانی، پشتِ یک `dyn Notifier`

```rust
for notifier in notifiers.iter_mut() {
    if notifier.notify(to, message).is_ok() {
        succeeded += 1;
    }
}
```

یک وسوسه‌ی طبیعی این بود که به‌جایِ حلقه، از `.iter_mut().filter(|n| n.notify(...).is_ok()).count()` استفاده کنی — کوتاه‌تر، و دقیقاً همان چیزی که ۲.۲ به‌ات یاد داد. ولی امتحانش کن: کامپایل نمی‌شود. کلوژرِ `.filter()` همیشه یک `&Item` می‌گیرد، ولی اینجا خودِ `Item` از قبل `&mut Box<dyn Notifier>` است — همان چیزی که `.iter_mut()` می‌دهد — پس پارامترِ کلوژر می‌شود `n: &&mut Box<dyn Notifier>`؛ یک ارجاعِ اشتراکی روی سرِ همان `&mut`. `.notify(...)` باید به یک `&mut Box<dyn Notifier>` برسد، و یک `&` را — از پشتِ هر چند لایه که باشد — نمی‌شود به `&mut` تبدیل کرد. یک حلقه‌ی ساده این مشکل را اصلاً پیش نمی‌آورد، چون `notifier` را مستقیماً به همان `&mut Box<dyn Notifier>`ای می‌بندد که `notifiers.iter_mut()` می‌دهد، بدون هیچ لایه‌ی ارجاعِ اضافه‌ای.

نکته‌ی اصلی جایِ دیگری است: بدنه‌ی حلقه هیچ فرقی با چیزی که رویِ یک `N: Notifier`ِ جنریک می‌نوشتی ندارد — همان `.notify(to, message)`. تنها چیزی که عوض شد نوعِ `notifiers` بود؛ `&mut [Box<dyn Notifier>]` به‌جایِ یک `N`ِ ثابت. ارسالِ پویا دقیقاً همین‌جا هزینه‌اش را می‌گیرد — یک پرشِ vtable سرِ هر فراخوانی — و دقیقاً همین‌جا هم چیزی که در ازایش می‌خری: یک `Vec` که `EmailNotifier` و چند `FakeNotifier` را واقعاً کنارِ هم نگه می‌دارد.

## این درس واقعاً درباره‌ی چه بود

- **مرزِ تعویض همیشه صفت بود، نه هیچ لایه‌ی جداگانه.** `ShippingService<N>` و `notify_all` هیچ‌کدام هیچ‌جا نمی‌دانستند `N` یا عنصرِ `Box<dyn Notifier>` واقعاً چه چیزی است — فقط `Notifier` را می‌شناختند.
- **`FakeNotifier` باید همان قانونِ کامپایلری را رعایت کند که `EmailNotifier` رعایت کرد.** این همان چیزی است که «چرا این جایِ فریم‌ورکِ mock را می‌گیرد» ادعا کرد، و `E0050` در «خطاهایی که خواهی دید» ثابتش کرد.
- **مالکیت، نه یک قراردادِ تست، تعیین می‌کند بعدِ `ship_order` چطور به `FakeNotifier` نگاه می‌کنی.** `ShippingService::new` مالکیتِ نوتیفایر را می‌گیرد، پس `notifier()` — نه متغیرِ محلی‌ای که پاسش دادی — راهِ رسمیِ رسیدن به آن است.
- **کراندِ جنریک و `dyn Trait` دو ابزارِ جداگانه‌اند، نه یک انتخابِ سلیقه‌ای.** `ShippingService<N>` یک نوعِ ثابت می‌خواست؛ `notify_all` به یک مخلوطِ واقعی نیاز داشت. هرکدام دقیقاً همان‌جایی به‌کار رفت که ۲.۳.۷ گفته بود.

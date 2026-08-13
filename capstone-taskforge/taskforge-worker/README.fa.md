# کریت `taskforge-worker`

استخرِ کارگرهای async: متدِ `(WorkerPool::new(store` تعدادِ `concurrency` حلقه‌ی تکرارِ مستقل (به طور پیش‌فرض 4 تا) بالا میاره، که هرکدوم با `JobStore::claim_next` یه جاب رو برمی‌دارن، اونو روی همون `JobHandler` ثبت‌شده‌ای که با `job_type` اون مچ باشه ران می‌کنن، و نتیجه رو گزارش می‌دن — موفقیت، تلاش مجدد با تاخیر تصاعدی به همراه لرزش (exponential-backoff-plus-jitter)، یا انتقال به کارهای مرده (dead-letter) وقتی که `max_attempts` تمام بشه. دلایل کاملِ معماری رو تو فایلِ [`../docs/adr/0004-worker-failure-handling.fa.md`](../docs/adr/0004-worker-failure-handling.fa.md) بخون.

```rust,ignore
let store: Arc<dyn JobStore> = Arc::new(PostgresJobStore::connect(&database_url).await?);
let pool = WorkerPool::new(store)
    .with_concurrency(8)
    .register(Arc::new(SendEmailHandler::new()))
    .register(Arc::new(GenerateReportHandler::new()));

let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
tokio::spawn(pool.run(shutdown_rx));
// ... موقع SIGTERM: اول shutdown_tx.send(true)؟ بعد هم await روی فیوچرِ ()pool.run
```

## چه چیزایی اینجا تست می‌شن، و چطوری، اونم با صفر زیرساخت

تک‌تکِ تست‌های تو فایلِ `src/lib.rs` یه `WorkerPool` واقعی رو در برابرِ یه `InMemoryJobStore` واقعی (از کریت `taskforge-storage` که کلاً به عنوان یه `dev-dependency` آورده شده — کُدِ واقعی این کریت هیچ‌وقت به هیچ پیاده‌سازیِ فیزیکیِ ذخیره‌سازی وابسته نیست، فقط به Traitِ `JobStore` وابسته‌ست) و چند تا `JobHandler`ِ مینیاتوری و مخصوصِ تست (`AlwaysSucceeds`، `AlwaysFails`، `AlwaysPanics`) ران می‌کنه. این قضیه رفتار واقعیِ مدیریتِ خطاهای سیستم رو به صورت صفر تا صد اثبات می‌کنه:

- یه جابِ موفق علامتِ `Succeeded` می‌خوره
- جابی که هندلراش `Err` برگردونه واسه تلاشِ مجدد (`Retrying`) زمان‌بندی می‌شه
- **جابی که هندلراش پنیک (panic) کنه باعث کِرش کردن استخر نمی‌شه** — استخر به کارش ادامه میده، و اون جاب بازم واسه تلاش مجدد زمان‌بندی می‌شه، دقیقاً شبیه یه `Err` عادی
- جابی که هیچ هندلرِ ثبت‌شده‌ای واسه `job_type` خودش نداشته باشه درجا به dead-letter منتقل می‌شه (چون کلاً دلیلی واسه تلاش مجددش وجود نداره)
- تموم شدنِ `max_attempts` به جای تلاش مجددِ بی‌نهایت، جاب رو منتقل می‌کنه به dead-letter

```sh
cargo test -p taskforge-worker
```

## باگ/شکافِ شناخته‌شده (مستند شده، نه از روی بی‌دقتی)

اگه کلاً *پروسه‌یِ* یه کارگر کشته بشه (نه اینکه فقط یه هندلر پنیک کنه — بلکه کلِ پروسه بمیره)، هر جابی که اون کارگر قبلش برداشته بود (claim) واسه همیشه تو وضعیتِ `Running` باقی می‌مونه بدون اینکه چیزی باشه تا تمومش کنه — تو این نسخه کلاً هیچ مکانیزمِ تایم‌اوتِ پینگ/اجاره (heartbeat/lease-timeout) وجود نداره که اتوماتیک اونو دوباره آزاد کنه. بخش "پیامدها" تو فایل ADR-0004 رو ببین. یه کار توسعه‌ایِ خوب: اضافه کردن ستون `claimed_at`/پینگ و تغییر متد `claim_next` تا جاب‌های `Running`ای که پینگشون کهنه شده رو هم اتوماتیک دوباره آزاد کنه.

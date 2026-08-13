# کریت `taskforge-scheduler`

کارهای زمان‌بندی‌شده و دوره‌ای (Recurring jobs): یه ساختار `ScheduledJob` (شامل یه `job_type`، یه بارِ داده‌ی ثابت یا همون `payload`، و یه بازه‌ی زمانی به اسم `interval`) رو ثبت کن، تا متدِ `Scheduler::run` اونو — از طریقِ همون Traitِ `JobStore` که همه‌ی کریت‌های دیگه استفاده می‌کنن — دقیقا به ازای هر بازه‌ی زمانی، واسه همیشه تا زمانِ خاموش شدن سیستم، ثبت کنه (enqueue).

```rust,ignore
let scheduler = Scheduler::new(store)
    .with_schedule(ScheduledJob::new("cleanup_temp_files", json!({}), Duration::from_secs(3600)))
    .with_schedule(ScheduledJob::new("send_daily_digest", json!({}), Duration::from_secs(86_400)));

let (tx, rx) = tokio::sync::watch::channel(false);
tokio::spawn(scheduler.run(rx));
```

اجرای واقعیِ یه جابِ زمان‌بندی‌شده به محض اینکه تو صف ثبت بشه، کلاً و تماماً به عهده‌ی کریتِ `taskforge-worker` هستش — این اسکیجولر فقط و فقط متدِ `JobStore::enqueue` رو صدا می‌زنه، دقیقاً مثل هر فراخواننده‌ی دیگه‌ای (از جمله مسیرِ `POST /jobs` تو کریتِ `taskforge-api`). این کریت کلاً صفر آگاهی و دانش نسبت به کارگرها (workers)، هندلرها، یا چطوری پردازش‌شدنِ نهاییِ جاب‌ها داره — که اینم یه نمونه‌ی واقعی و خفنِ دیگه از کارکردِ مرزِ Traitِ `JobStore` بر اساسِ سندِ [`../docs/adr/0001-architecture-overview.fa.md`](../docs/adr/0001-architecture-overview.fa.md) هستش.

## محدودیتِ شناخته‌شده (مستندشده، نه از روی بی‌دقتی)

بازه‌های زمانی به صورتِ مقادیرِ ثابتِ `Duration` تعریف شدن، نه عبارت‌های واقعیِ cron — در نتیجه پشتیبانی از چیزایی مثل "هر روز ساعت ۹ صبح" یا "هر دوشنبه" تو این نسخه وجود نداره. یه توسعه‌ی عالی برای رفتن به فاز بالاتر: جایگزین کردنِ فیلدِ `ScheduledJob::interval: Duration` با یه کریتِ کوچیکِ پارسرِ cron (مثل کریت `cron`) و محاسبه‌ی تایمِ شلیکِ بعدی از روی فرمولِ کرون به جای بازه‌ی زمانیِ ثابت — کلاً شکل و ساختارِ بقیه‌ی متدِ `Scheduler::run` دست‌نخورده باقی می‌مونه.

## ران کردن تست‌ها

```sh
cargo test -p taskforge-scheduler
```

بدون نیاز به هیچ زیرساختی — تست‌ها در برابر `InMemoryJobStore` ران می‌شن (که فقط یه `dev-dependency` هست؛ کد واقعی این کریت هیچ‌وقت به پیاده‌سازیِ فیزیکی وابستگی نداره) و واسه اینکه تست‌ها سریع تموم شن از بازه‌های زمانی خیلی کوتاه (۱۰ تا ۱۵ میلی‌ثانیه) استفاده شده.

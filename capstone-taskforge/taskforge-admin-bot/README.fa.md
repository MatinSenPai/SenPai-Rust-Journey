# کریت `taskforge-admin-bot`

یه رباتِ تلگرام به عنوانِ کلاینتِ مدیریتِ ChatOps برای `taskforge-api` — همون مهارت‌های کار با `teloxide` از مأموریتِ جانبیِ
[`side-quests/sq-02-telegram-quiz-bot/README.fa.md`](../../side-quests/sq-02-telegram-quiz-bot/README.fa.md)، که حالا به جای یه کوییز متصل شده به زیرساختِ واقعی. دستورات:

- `/status` — جاب‌های اخیر، دسته‌بندی و شمارش‌شده بر اساس وضعیت
- `/cancel <id>` — کنسل کردن یه جاب با آیدی

## چرا فقط این دو تا دستور، در حالی که تو ADR اسم از `/retry` و `/pause_queue` برده شده بود؟

فایلِ `../docs/adr/0001-architecture-overview.fa.md` دستوراتِ `/retry <id>` و `/pause_queue` رو به عنوان دستوراتِ آینده ترسیم کرده بود. در واقعیت فقط `/status` و `/cancel` پیاده‌سازی شدن، چون `taskforge-api` هنوز اندپوینت‌های `POST /jobs/{id}/retry` یا `POST /queue/pause` رو نداره — این ربات فقط اندپوینت‌های موجود و واقعی رو صدا می‌زنه. اضافه کردنِ این دو تا دستور یه توسعه‌ی فوق‌العاده و خوش‌ساخت برای قدم‌های بعدیه:
1. اضافه کردنِ `POST /jobs/{id}/retry` به `taskforge-api` (ثبت مجدد یه جابِ `DeadLetter` شده به عنوان `Pending`) و متدِ معادلِ `retry_job` روی Traitِ `JobStore` تو کریت `taskforge-core`.
2. اضافه کردنِ متد `retry_job`/`pause_queue` به `ApiClient` (تو `src/client.rs`).
3. اضافه کردنِ مقدار معادلِ `Command` تو فایل `src/main.rs`.

## ساختار کد (همون تفکیکِ مأموریت جانبی ۲)

- `src/format.rs` — منطقِ محضِ فرمت‌دهی (`format_status`، `format_cancel_result`)، صفر I/O، کاملاً یونیت‌تست‌شده.
- `src/client.rs` — یه کلاینت HTTPِ سبک مبتنی بر `reqwest` برای `taskforge-api`.
- `src/main.rs` — اتصالاتِ اصلیِ ربات با `teloxide`. برای اجرا نیاز به توکن واقعیِ ربات (متغیر محیطی `TELOXIDE_TOKEN` — که از [@BotFather](https://t.me/BotFather) می‌گیری) و دسترسی به اینترنت داره؛ با `cargo test` اجرا **نمی‌شه**.

## ران کردن تست‌ها

```sh
cargo test -p taskforge-admin-bot
```

## ران کردن واقعی

```sh
export TELOXIDE_TOKEN="your-bot-token-from-botfather"
export TASKFORGE_API_URL="http://localhost:8080"
export TASKFORGE_API_TOKEN="whatever-token-taskforge-api-was-started-with"
cargo run -p taskforge-admin-bot
```

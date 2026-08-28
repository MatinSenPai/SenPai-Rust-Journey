# ۰۸ — هم‌روندی

این ماژول همان چیزی است که قاعده‌ی هم‌نامی برایش وضع شده بود. هر تضمینی که
قرض گرفتن از فاز ۱ به تو داده — هرگز دو دسترسیِ تغییرپذیر هم‌زمان به یک داده —
دقیقاً همان چیزی است که Rustِ هم‌روند را «اشتباه کردنش سخت» می‌کند، نه فقط
«درست کردنش سخت». این ماژول جایی تمام می‌شود که async شروع می‌شود.

۱. [ریسه‌ها، `Mutex`، `Arc`](01-threads-mutex-arc/README.fa.md)
۲. [`RwLock`، `Semaphore`، `OnceLock`/`LazyLock`، اتمیک‌ها](02-rwlock-semaphore-oncelock-atomics/README.fa.md)
۳. [کانال‌ها و پیام‌رسانی](03-channels-message-passing/README.fa.md)
۴. [`Send` و `Sync`: چه هستند و چرا نوعِ تو `Send` نیست](04-send-and-sync/README.fa.md)
۵. [فیوچرها و رانتایم‌ها: `async fn` به چه تبدیل می‌شود](05-futures-and-runtimes/README.fa.md)
۶. [مقدماتِ `tokio`](06-tokio-basics/README.fa.md)

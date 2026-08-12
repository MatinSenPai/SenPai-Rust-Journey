# ایست بازرسی

۱. اگر پس از spawn همه‌ی producerها `drop(tx)` را فراموش کنی، تست چرا hang می‌کند؟
۲. برای یک نتیجه می‌شد از `JoinHandle<i32>` استفاده کرد؛ برای چند progress update چه تفاوتی ایجاد می‌شود؟
۳. چرا `Sender<T>`، `Clone` است ولی `Receiver<T>` نیست؟
۴. جمع‌کردن پیام‌های مستقل با channel چه تفاوتی با تغییر counter واحد زیر `Arc<Mutex<_>>` دارد؟

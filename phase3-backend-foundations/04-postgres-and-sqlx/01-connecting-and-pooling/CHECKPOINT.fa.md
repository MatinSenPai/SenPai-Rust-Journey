# ایست بازرسی

۱. در test بیست query هم‌زمان و pool پنج‌تایی، queryهای ۶ تا ۲۰ چه می‌شوند: reject، drop یا await در صف؟
۲. فایده‌ی عملی `.connect(database_url).await` eager چیست و کدام test درس نشانش می‌دهد؟
۳. بالا بردن `max_connections` از ۵ به ۵۰۰ چه ریسک واقعی دارد و سقف واقعی از کجا می‌آید؟
۴. با حذف `acquire_timeout` کدام test و چگونه تغییر می‌کند؟
۵. چرا `tokio::spawn` هر task را مجبور به `pool.clone()` می‌کند و این الگوی `Arc` قبلاً کجا آموزش داده شد؟

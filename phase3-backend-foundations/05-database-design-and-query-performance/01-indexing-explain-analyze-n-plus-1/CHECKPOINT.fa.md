# ایست بازرسی

۱. B-tree چه داده‌ای دارد که table خام ندارد و چگونه `Seq Scan` را به `Index Scan` تبدیل می‌کند؟
۲. index برای هر write چه هزینه و چرا «همه columnها را index کن» چه anti-patternی دارد؟
۳. برای پنج post، کدام lineهای naive و batched query countهای ۶ و ۱ می‌سازند؟
۴. چرا regroupکردن JOIN می‌تواند function خالص بدون Tokio و database باشد؟
۵. `query!` macroها در CI/sandbox بدون database چه چیزی را در `cargo check` می‌شکنند؟
۶. کدام method همتای `prefetch_related` و کدام همتای loop lazy `post.comments.all()` است؟
۷. چرا حذف `#[serial]` از فقط یک test DB همچنان race reset table را باز می‌کند؟

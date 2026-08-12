# ایست بازرسی

۱. اگر crate دیگری پس از افزودن این package به dependencies بنویسد `band.internal_rating` چه می‌شود و در compile time یا runtime؟
۲. چرا module تست می‌تواند `catalog::normalize_rating(...)` خصوصی را ببیند؟ قانون دسترسی parent و child را توضیح بده.
۳. `pub use catalog::Anime;` نوع را جابه‌جا نمی‌کند؛ دقیقاً چه می‌کند و چرا از publicکردن کل `catalog` بهتر است؟
۴. برگرداندن `low/medium/high` به‌جای عدد داخلی چه آزادی‌ای برای تغییر implementation آینده نگه می‌دارد؟
۵. یک مزیت build-time workspace مشترک نسبت به crateهای کاملاً جدا نام ببر.

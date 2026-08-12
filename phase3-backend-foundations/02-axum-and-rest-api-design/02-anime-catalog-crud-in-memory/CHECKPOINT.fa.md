# ایست بازرسی

۱. چرا `tests/store_test.rs` بدون import `axum` همه‌ی validation و not-foundها را پوشش می‌دهد؟ اگر منطق `create` داخل handler بود، strategy تست چه تغییری می‌کرد؟
۲. `UpdateAnime` برای `rating: Option<u8>` فقط «تغییر نده» را با `None` دارد. برای بیان «پاک‌کردن صریح rating» در کنار «دست نزن» چه typeی می‌سازی؟ نکته‌ی serde برای تفاوت absent و JSON null چیست؟
۳. چرا response موفق create باید `StatusCode::CREATED` داشته باشد؟ `Json<Anime>` تنها چه statusی می‌دهد؟
۴. برای envelopکردن همه‌ی successها مانند `{"data": ...}` چه تغییرهایی لازم است و چرا از envelope error بزرگ‌تر است؟
۵. چرا یک `Mutex<StoreInner>` مشترک برای `next_id` و `items` از دو Mutex جدا درست‌تر است؟ یک race lost-write را شرح بده.

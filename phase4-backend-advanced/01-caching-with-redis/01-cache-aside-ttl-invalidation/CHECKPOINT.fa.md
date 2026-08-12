# ایست بازرسی

1. چرا `compute: F` در `get_or_set` باید lazy باشد؟ اگر caller نتیجه را پیشاپیش بسازد، cache hit چه فایده‌ای دارد؟
2. چرا `InMemoryCache::get` با `&self` می‌تواند entry منقضی را حذف نکند؟ این انتخاب بین bookkeeping و memory چه بهایی دارد؟
3. اگر دو request هم‌زمان برای یک key miss بخورند، cache stampede چیست و lock یا single-flight برای هر key چه کمکی می‌کند؟
4. چرا `FakeClock` از `Cell<Instant>` استفاده می‌کند، با اینکه `Clock::now(&self)` فقط reference مشترک می‌گیرد؟
5. یک تجربهٔ بد کاربر مثال بزن که TTL بدون invalidation صریح پس از write ایجاد می‌کند.

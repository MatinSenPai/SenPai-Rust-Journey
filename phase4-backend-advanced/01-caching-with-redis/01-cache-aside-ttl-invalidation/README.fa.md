# ۰۴.۱.۱ — Cache-aside، TTL و invalidation

به فاز چهار خوش آمدی. از اینجا هر درس یک ایدهٔ طراحی سیستم را به کد اجرایی وصل می‌کند. cache احتمالاً پُربازده‌ترین راه برای سریع و ارزان‌کردن backend است.

فرض کن صفحهٔ خانهٔ Django در هر request این query را اجرا می‌کند:

```python
Post.objects.filter(published=True).order_by("-created_at")[:20]
```

اگر query چهل میلی‌ثانیه طول بکشد و صفحه ۵۰۰ request در ثانیه داشته باشد، database بارها پاسخ یکسان را محاسبه می‌کند. cache نتیجه را یک‌بار محاسبه، در جایی بسیار سریع (معمولاً Redis) ذخیره و تا وقتی معتبر است برمی‌گرداند. معامله روشن است: گاهی دادهٔ کمی stale را در برابر سرعت و فشار کمتر روی source of truth می‌پذیری.

```senpai-visual
{"kind":"database","labels":["request","cache hit یا miss","database/source of truth","cache با TTL"]}
```

## الگوی cache-aside

```
مسیر read:
  1. cache را برای key بپرس
  2. HIT  -> مقدار cache را برگردان
  3. MISS -> مقدار واقعی را محاسبه کن
          -> در cache بگذار
          -> برگردان

مسیر write:
  1. source of truth را بنویس
  2. key مربوطه را invalidate (delete) کن
```

application این هماهنگی را انجام می‌دهد، نه خود cache. cache-aside انتخاب پیش‌فرض خوبی است چون منطق محاسبهٔ دادهٔ واقعی از قبل در application وجود دارد.

## TTL؛ تور ایمنی، نه جایگزین invalidation

invalidation فراموش‌شده دادهٔ قدیمی را تا ابد سرو می‌کند. **TTL (time-to-live)** برای هر entry زمان انقضا می‌گذارد؛ پس حتی اگر invalidation را جا انداختی، مثلاً حداکثر پس از ۶۰ ثانیه خودبه‌خود تازه می‌شود. در writeهای درست هنوز باید همان لحظه invalidate کنی. TTL کوتاه فایدهٔ cache را کم می‌کند؛ TTL بلند خطای invalidation را طولانی‌تر نشان می‌دهد.

## key بخشی از طراحی است

cache فضایی تخت از `string -> string` است. keyهای بی‌نظم می‌توانند دو feature نامرتبط را روی هم بیندازند. قالب این درس `namespace:identifier` است: `"user-profile:42"` یا `"homepage-feed:page-1"`. namespace کمک می‌کند بدانی هنگام تغییر پروفایل کدام keyها را باید پاک کنی.

## چرا traitِ `Cache`؟

`get_or_set` بر ضد trait نوشته می‌شود، نه crateِ `redis`: همین dependency inversion دو دستاورد دارد. با `InMemoryCache` مبتنی بر `HashMap` منطق cache-aside و TTL را بدون سرویس بیرونی و با testهای deterministic می‌سنجی؛ سپس `RedisCache` واقعی همان trait را در production پیاده می‌کند. تست‌های Redis واقعی `#[ignore]` هستند:

```sh
redis-server --daemonize yes
cargo test -p p4-01-01-cache-aside-ttl-invalidation -- --ignored
```

در `src/lib.rs`، `cache_key`، سه متد `InMemoryCache` و orchestrationِ `get_or_set` را کامل کن. `FakeClock` عمداً sleep ندارد تا TTL testها فوری و قابل‌پیش‌بینی باشند.

## Checkpoint

`cargo test -p p4-01-01-cache-aside-ttl-invalidation`، سپس `CHECKPOINT.fa.md` و `solution/SOLUTION.fa.md`.

# مأموریت جانبی ۴ — API تجمیع‌کننده انیمه/مانگا

دست‌گرمی قبل پروژه‌ی پایانی: سه فکربکر از فاز ۴ — کشینگِ cache-aside، محدودکننده نرخ الگوی token-bucket، و نظارت‌پذیری مبتنی بر `tracing` — ترکیب‌شده تو یه پروژه مینیاتوری و جذاب.

## چی کار می‌کنه

یه API برای تجمیع اطلاعات عناوین انیمه/مانگا (عنوان، خلاصه، تعداد قسمت/چپتر).

```
GET /titles       تمام عناوین شناخته‌شده، بدون کشینگ
GET /titles/{id}  یک عنوان — مجهز به cache-aside، لاگ‌کننده hit یا miss
```

## سه بخش اصلی

1. **Cache-aside** (`TtlCache` + `get_or_fetch`) — اول کش رو چک کن؛ اگه hit شد درجا برگردون. اگه miss شد کوئری رو بزن، نتیجه رو کش کن، برگردون.
2. **Rate limiting** (`TokenBucket` + `rate_limit_middleware`) — هر درخواست باید یه توکن از باکت مشترک بگیره یا با `429` رد می‌شه.
3. **Observability** (لاگ‌های `tracing` متصل‌شده) — هر درخواست متد، مسیر، hit/miss کش و تاخیرش رو به عنوان فیلد ساخت‌یافته لاگ می‌کنه.

## وظیفه‌ی تو

فایل `src/lib.rs` رو باز کن. توابع `TtlCache::get`، `TtlCache::set`، `get_or_fetch` و `TokenBucket::refill` رو پیاده‌سازی کن.

## چک‌پوینت

`cargo test -p sq-04-anime-manga-aggregator-api`. بعد `CHECKPOINT.md` و در نهایت `solution/SOLUTION.md`.

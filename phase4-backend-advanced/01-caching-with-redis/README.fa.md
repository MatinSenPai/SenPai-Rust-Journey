# ۰۱ — Caching با Redis

هر backend دیر یا زود به دیوار یک query، API call یا محاسبهٔ گران می‌رسد که بسیار بیشتر از تغییر داده درخواست می‌شود. cache یک کپی سریع و ارزان از نتیجه را نزدیک نگه می‌دارد و فقط وقتی لازم باشد هزینهٔ اصلی را دوباره می‌پردازد. این ماژول الگوی رایج production را می‌سازد: cache-aside با TTL و invalidation صریح.

1. [Cache-aside، TTL و invalidation](01-cache-aside-ttl-invalidation/README.fa.md)

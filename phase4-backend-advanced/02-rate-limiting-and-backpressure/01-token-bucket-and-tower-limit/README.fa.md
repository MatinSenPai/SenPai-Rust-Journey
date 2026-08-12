# ۰۴.۲.۱ — Token bucket و `tower::limit`

cache خواندن را ارزان کرد؛ این درس جلوی درخواستِ کار بیش از حد را می‌گیرد. rate limiting وقتی client، bug یا attacker نرخ توافق‌شده را رد کرد request را رد یا عقب می‌اندازد، به‌جای آنکه database، API بیرونی یا CPU را تا مرز شکست ببرد.

## الگوریتم token bucket

یک سطل با حداکثر `capacity` token تصور کن. ابتدا پر است. هر request برای عبور یک token برمی‌دارد؛ اگر خالی بود رد یا منتظر می‌شود. هم‌زمان tokenها با `refill_rate` در ثانیه و تا سقف capacity برمی‌گردند.

```senpai-visual
{"kind":"queue","labels":["سطل پر","burst تا capacity","refill پیوسته","رد با 429"]}
```

```
capacity = 5, refill_rate = 1 token/sec

t=0.0s  bucket: [#####] (5/5)  try_acquire -> true   (4/5)
t=0.0s  ... چهار درخواست دیگر ...                    (0/5)
t=0.0s  bucket: [.....]        try_acquire -> false
t=2.5s  2.5 token پر شده       try_acquire -> true    (1.5/5)
```

نتیجهٔ عمدی: burst تا `capacity` مجاز است؛ مثلاً page load می‌تواند پنج API call هم‌زمان بزند. اما نرخ پایدار هرگز از `refill_rate` بیشتر نمی‌شود. این الگو یا خویشاوند نزدیکش در AWS API Gateway، Stripe، nginx و `tower::limit::RateLimitLayer` دیده می‌شود.

## backpressure وقتی سطل خالی است

1. **رد فوری**؛ `try_acquire` همین کار را می‌کند و API معمولاً `429 Too Many Requests` می‌دهد.
2. **queue با سقف**؛ request را تا token بعدی نگه می‌داری، با محدودیت زمان/عمق queue. latency بیشتر و resource بیشتر مصرف می‌شود.
3. **queue نامحدود**؛ حالت شکستی که باید از آن دوری کنی: memory و latency بی‌حد رشد می‌کنند و سرویس دیرتر اما بدتر می‌افتد.

تمرین اینجا crate نیست، algorithm است. `TokenBucket` بدون dependency و sync است؛ `now: Instant` را صریح می‌گیرد تا test بدون `sleep` زمان را کنترل کند. در service واقعی `tower::limit::RateLimitLayer` را روی router می‌گذاری؛ `poll_ready` آن برای اشباع `Pending` می‌دهد و به strategy دوم نزدیک است.

در `src/lib.rs`، `new` را با bucket پر، `refill` را با elapsed time و cap، و `try_acquire` را با refill پیش از تصمیم کامل کن.

## Checkpoint

`cargo test -p p4-02-01-token-bucket-and-tower-limit`، بعد `CHECKPOINT.fa.md` و `solution/SOLUTION.fa.md`.

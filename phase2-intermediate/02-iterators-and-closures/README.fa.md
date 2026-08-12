# ۰۲ — iterator و closure

اینجا Rust زبان و idiomهای خودش را نشان می‌دهد. closure تابع بی‌نامی است که environment را capture می‌کند. adapterهای trait `Iterator` مانند `.map()`، `.filter()` و `.fold()` زنجیره‌های lazy و composable می‌سازند که کامپایلر معمولاً آن‌ها را به loop فشرده تبدیل می‌کند.

این رویکرد جای list comprehensionهای eager پایتون را در بسیاری از مسیر‌ها می‌گیرد، اما iterator همیشه بهتر نیست؛ وقتی loop صریح وضعیت و branching پیچیده را روشن‌تر می‌کند، خوانایی را قربانی زنجیره‌ی نمایشی نکن.

1. [closure و traitهای `Fn`](01-closures-and-fn-traits/README.md)
2. [adapterهای iterator](02-iterator-adapters/README.md)

```senpai-visual
{"kind":"concept","labels":["source","map/filter","collect"]}
```

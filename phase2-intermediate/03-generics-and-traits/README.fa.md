# ۰۳ — generic و trait

در Python، روش Duck Typing معمولاً هنگام اجرا روشن می‌کند یک شیء از عملیات موردنیاز پشتیبانی می‌کند یا نه. در Rust، نوع عمومی (Generic) اجازه می‌دهد یک تابع یا struct را برای چند نوع بنویسی و trait قرارداد رفتارهای لازم را هنگام کامپایل بیان می‌کند. در ارسال ایستا (Static Dispatch)، کامپایلر برای نوع‌های واقعی کد تخصصی می‌سازد؛ شیء trait هم ارسال پویا (Dynamic Dispatch) را با هزینه و انعطاف متفاوت ممکن می‌کند.

1. [تابع و struct generic](01-generic-functions-and-structs/README.md)
2. [تعریف و پیاده‌سازی trait](02-defining-and-implementing-traits/README.md)
3. [شیء trait در برابر ارسال ایستا](03-trait-objects-vs-static-dispatch/README.md)

نوع‌های خطا، اشاره‌گرهای هوشمند و پردازشگرهای `axum` روی همین مدل ساخته می‌شوند. trait را فقط برای یک انتزاع احتمالی نساز؛ این قرارداد باید استفاده‌کننده‌ای واقعی داشته باشد.

```senpai-visual
{"kind":"concept","labels":["T","trait bound","implementation"]}
```

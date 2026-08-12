# ۰۸.۲ — مبانی `macro_rules!`

## matcher و transcriber

هر arm ماکرو pattern ورودی یا matcher و کد خروجی یا transcriber دارد. compiler پیش از type checking نهایی tokenها را با matcher تطبیق می‌دهد و expansion را جای فراخوانی می‌گذارد. fragment specifierهایی مانند `expr` و `ident` نوع syntax پذیرفته‌شده را مشخص می‌کنند.

## تکرار

الگوهای `$($item:expr),*` صفر یا چند و `+` یک یا چند مورد را می‌گیرند. separator بیرون گروه می‌آید. ماکروی recursive مانند `max_of!` به base case نیاز دارد، وگرنه expansion تمام نمی‌شود.

## hygiene

identifierهای تعریف‌شده داخل ماکرو با bindingهای call site تصادف نمی‌کنند. بااین‌حال ماکرو می‌تواند control flow و تعداد ارزیابی expression را پنهان کند؛ باید هر ورودی را دقیقاً به تعداد قراردادی ارزیابی کرد.

```senpai-visual
{"kind":"concept","labels":["token ورودی","matcher","transcriber","کد expandشده"]}
```

## چه زمانی ماکرو ننویسیم؟

اگر function یا generic کافی است، همان خواناتر، type-checkableتر و قابل‌دیباگ‌تر است. ماکرو برای syntax تازه، تعداد آرگومان متغیر یا تولید ساختار تکراری مناسب است؛ نه صرفاً کوتاه‌کردن چند خط. early return پنهان از تابع caller و ارزیابی چندباره‌ی آرگومان‌ها API غافلگیرکننده می‌سازند.

## تمرین تو

`string_map!`، `max_of!` و `timed!` را کامل کن و مطمئن شو expression کار فقط یک بار اجرا می‌شود.

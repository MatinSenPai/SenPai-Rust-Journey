# ۰۶ — نبودن و شکست

Rust نه null دارد و نه استثنا. به‌جایش دو enumِ معمولی دارد — `Option` و `Result` — و یک عملگر که کار کردن با آن‌ها را دلپذیر می‌کند.

از فاز ۰ تا حالا مدام با `Option` روبه‌رو شده‌ای، هر بار که چیزی `Some` یا `None` برگردانده. این ماژول جایی است که `Option` از چیزی که دورش می‌زنی تبدیل می‌شود به ابزاری که سراغش می‌روی.

۱. [`Option` و ایمنی در برابرِ نبودن](01-option-and-null-safety/README.fa.md)
۲. [ترکیب‌گرهای `Option`](02-option-combinators/README.fa.md)
۳. [`Result` و علامتِ سؤال](03-result-and-question-mark/README.fa.md)
۴. [پنیک در برابرِ `Result`](04-panic-vs-result/README.fa.md)
۵. [`From` و تبدیلِ خطا](05-from-and-error-conversion/README.fa.md)

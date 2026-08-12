# راه‌حل

- `a.checked_add(b)` روی overflow مقدار `None` می‌دهد و سیاست را در نوع آشکار می‌کند.
- `(point.0 * point.0 + point.1 * point.1).sqrt()` عضو‌های tuple را positionally می‌خواند.
- `nums.iter().sum()` از iterator و `sum` استفاده می‌کند.
- `c.len_utf8()` تعداد byte لازم برای encode همان Unicode scalar مقدار در UTF-8 را می‌دهد.

`char` همیشه ۴ byte storage دارد، ولی UTF-8 متغیرطول است. به همین علت `String` indexing مستقیم بر اساس character ندارد و `String::len()` تعداد byteهاست.

# فاز ۳ — پایه‌ها و اصول بک‌اند (Backend Foundations)

این دقیقاً همون نقطه‌ایه که تمام تجربه‌ی قبلیت با جنگو (Django) به ثمر می‌شینه — و البته به چالش کشیده می‌شه. تو از قبل می‌دونی که یه REST API، یه دیتابیس، و سیستمِ احراز هویت (auth) از نظر *مفهومی* چی هستن؛ این فاز قراره همون درکِ شهودیت رو، این بار با فهمیدنِ اینکه زیرِ پوستِ یه فریم‌ورک دقیقاً چه اتفاقاتی می‌افته از نو بسازه، و قبل از اینکه اصلاً بخوای `axum` رو وارد کنی، اول با پایین‌ترین سطح یعنی TCP خام شروع می‌کنه.

1. **شبکه و HTTP از صفرِ مطلق**
   - [۰۱ — سرور اکوی TCP](01-networking-and-http-from-scratch/01-tcp-echo-server/README.md)
   - [۰۲ — پارسر HTTP دست‌ساز](01-networking-and-http-from-scratch/02-hand-rolled-http-parser/README.md)
2. **فریم‌ورک `axum` و طراحی REST API**
   - [۰۱ — مسیریابی (Routing)، هندلرها، استخراج‌کننده‌ها (extractors)](02-axum-and-rest-api-design/01-routing-handlers-extractors/README.md)
   - [۰۲ — عملیاتِ CRUD رو کاتالوگ انیمه (داخل حافظه - in-memory)](02-axum-and-rest-api-design/02-anime-catalog-crud-in-memory/README.md)
3. **سریال‌سازی (Serialization) و اعتبارسنجی (Validation)**
   - [۰۱ — کریت‌های `serde_json` و `validator`](03-serialization-and-validation/01-serde-json-and-validator/README.md)
4. **دیتابیس PostgreSQL و `sqlx`**
   - [۰۱ — اتصال به دیتابیس و مدیریتِ pool](04-postgres-and-sqlx/01-connecting-and-pooling/README.md)
   - [۰۲ — مایگریشن‌ها (Migrations)](04-postgres-and-sqlx/02-migrations/README.md)
   - [۰۳ — کاتالوگ انیمه، این‌بار متصل به دیتابیسِ Postgres](04-postgres-and-sqlx/03-anime-catalog-postgres-backed/README.md)
5. **طراحی دیتابیس و پرفورمنسِ کویری‌ها (Query performance)**
   - [۰۱ — ایندکس‌گذاری (Indexing)، دستورِ `EXPLAIN ANALYZE`، و مشکلِ N+1](05-database-design-and-query-performance/01-indexing-explain-analyze-n-plus-1/README.md)
6. **احراز هویت و امنیت (Auth & security)**
   - [۰۱ — هش کردن رمز عبور (Password hashing) با الگوریتم `argon2`](06-auth-and-security/01-password-hashing-argon2/README.md)
   - [۰۲ — توکن‌های JWT و استفاده از میان‌افزارِ (middleware) `tower`](06-auth-and-security/02-jwt-and-tower-middleware/README.md)
7. **مدیریت خطاها و تست کردن در مقیاس بزرگ**
   - [۰۱ — پاکت‌ها و قالب‌های خطایِ یکدست (error envelopes)](07-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.md)
   - [۰۲ — تست‌های یکپارچگی (Integration tests) با استفاده از `testcontainers`](07-error-handling-and-testing-at-scale/02-integration-tests-with-testcontainers/README.md)

**ایستگاهِ روحیه‌دهی و انگیزشی:** [ماموریت جانبی ۳ — سرویس نوتیفیکیشنِ وبتون (Webtoon Notification Service)](../side-quests/sq-03-webtoon-notifier-service/README.md)
— ترکیبی از `axum` + دیتابیس Postgres + یه جاب زمان‌بندی‌شده (scheduled job)، که پیش‌نمایشی از جاب‌های پس‌زمینه (background jobs) در فاز ۴ رو بهت میده.

**پیش‌نیاز:** نصب بودنِ PostgreSQL رو سیستم خودت (یا اجرا از طریق Docker) از ماژول ۴ به بعد — تو فایلِ `README.md` همون ماژول مراحل نصبش توضیح داده شده، تا قبل از اون هیچ نیازی بهش نیست.

وقتی فاز ۳ به طور کامل تو فایلِ [PROGRESS.md](../PROGRESS.md) تیک خورد و تموم شد، برو سراغ [فاز ۴](../phase4-backend-advanced/README.md).
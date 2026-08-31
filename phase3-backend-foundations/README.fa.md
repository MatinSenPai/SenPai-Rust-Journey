# فاز ۳ — پایه‌ها و اصول بک‌اند (Backend Foundations)

این دقیقاً همون نقطه‌ایه که تمام تجربه‌ی قبلیت با جنگو (Django) به ثمر می‌شینه — و البته به چالش کشیده می‌شه. تو از قبل می‌دونی که یه REST API، یه دیتابیس، و سیستمِ احراز هویت (auth) از نظر *مفهومی* چی هستن؛ این فاز قراره همون درکِ شهودیت رو، این بار با فهمیدنِ اینکه زیرِ پوستِ یه فریم‌ورک دقیقاً چه اتفاقاتی می‌افته از نو بسازه، و قبل از اینکه اصلاً بخوای `axum` رو وارد کنی، اول با پایین‌ترین سطح یعنی TCP خام شروع می‌کنه.

سی‌وسه درس تو هشت ماژول، به همون ترتیبی که قراره خونده بشن.

## ۱. [شبکه و HTTP از صفرِ مطلق](01-networking-and-http-from-scratch/README.fa.md)

قبل از `axum`، ببین خودِ `axum` روی چی ساخته شده.

۱. [سرور اکوی TCP](01-networking-and-http-from-scratch/01-tcp-echo-server/README.fa.md)
۲. [پارسر HTTP دست‌ساز](01-networking-and-http-from-scratch/02-hand-rolled-http-parser/README.fa.md)
۳. [چیزهایی از HTTP که باید بدونی](01-networking-and-http-from-scratch/03-http-semantics-you-must-know/README.fa.md)

## ۲. [فریم‌ورک `axum` و طراحی REST API](02-axum-and-rest-api-design/README.fa.md)

فریم‌ورکی که این دوره از اینجا به بعد روش بنا می‌شه.

۱. [مسیریابی، هندلرها، extractorها](02-axum-and-rest-api-design/01-routing-handlers-extractors/README.fa.md)
۲. [نوشتنِ extractor خودت (`FromRequestParts`)](02-axum-and-rest-api-design/02-writing-your-own-extractor/README.fa.md)
۳. [عملیاتِ CRUD رو کاتالوگ انیمه (داخل حافظه)](02-axum-and-rest-api-design/03-anime-catalog-crud-in-memory/README.fa.md)
۴. [`tower::Service`/`Layer`: میان‌افزار با دست](02-axum-and-rest-api-design/04-tower-service-and-layer-middleware/README.fa.md)
۵. [مفهوم CORS و اتصال به فرانت‌اند](02-axum-and-rest-api-design/05-cors-and-frontend-integration/README.fa.md)

## ۳. [سریال‌سازی و اعتبارسنجی](03-serialization-and-validation/README.fa.md)

اینکه `Json<T>` واقعاً چیکار می‌کنه، یه لایه عمیق‌تر.

۱. [عمقِ serde](03-serialization-and-validation/01-serde-depth/README.fa.md)
۲. [اعتبارسنجی](03-serialization-and-validation/02-validation/README.fa.md)
۳. [قراردادهایِ API و OpenAPI (`utoipa`)](03-serialization-and-validation/03-api-contracts-and-openapi/README.fa.md)
۴. [نسخه‌بندیِ API و تکاملش](03-serialization-and-validation/04-api-versioning-and-evolution/README.fa.md)

## ۴. [پیکربندی و ساختارِ اپلیکیشن](04-configuration-and-app-structure/README.fa.md)

زیرساختی که یه دیپلویِ واقعی قبل از دستش‌رسیدن به دیتابیس لازم داره.

۱. [پیکربندیِ ۱۲فاکتوری و سکرت‌ها](04-configuration-and-app-structure/01-config-and-secrets/README.fa.md)
۲. [وضعیتِ اپلیکیشن و سیم‌کشیِ وابستگی‌ها](04-configuration-and-app-structure/02-app-state-and-dependency-wiring/README.fa.md)
۳. [خاموشیِ آرام، health و readiness](04-configuration-and-app-structure/03-graceful-shutdown-health-readiness/README.fa.md)

## ۵. [دیتابیس PostgreSQL و `sqlx`](05-postgres-and-sqlx/README.fa.md)

یه دیتابیس که یه ری‌استارت رو دووم می‌آره.

۱. [اتصال و pooling](05-postgres-and-sqlx/01-connecting-and-pooling/README.fa.md)
۲. [مایگریشن‌ها](05-postgres-and-sqlx/02-migrations/README.fa.md)
۳. [کاتالوگ انیمه، این‌بار متصل به Postgres](05-postgres-and-sqlx/03-anime-catalog-postgres-backed/README.fa.md)
۴. [تراکنش‌ها](05-postgres-and-sqlx/04-transactions/README.fa.md)
۵. [الگویِ repository](05-postgres-and-sqlx/05-repository-pattern/README.fa.md)

## ۶. [طراحیِ دیتابیس و پرفورمنسِ کویری‌ها](06-database-design-and-query-performance/README.fa.md)

چیزی که وقتی حجمِ داده واقعی می‌شه، عوض می‌شه.

۱. [ایندکس‌گذاری، `EXPLAIN ANALYZE`، مسئله‌ی N+1](06-database-design-and-query-performance/01-indexing-explain-analyze-n-plus-1/README.fa.md)
۲. [صفحه‌بندی: offset در برابرِ keyset](06-database-design-and-query-performance/02-pagination/README.fa.md)
۳. [طراحیِ اسکیما برایِ یک سرویسِ واقعی](06-database-design-and-query-performance/03-schema-design-for-a-real-service/README.fa.md)

## ۷. [احراز هویت و امنیت](07-auth-and-security/README.fa.md)

هر چیزی که مدلِ `User`ِ جنگو مجانی بهت می‌داد، این‌بار با دست.

۱. [هش‌کردنِ پسورد با `argon2`](07-auth-and-security/01-password-hashing-argon2/README.fa.md)
۲. [Session در برابرِ JWT: مصالحه‌یِ واقعی](07-auth-and-security/02-sessions-vs-jwt/README.fa.md)
۳. [JWT و میان‌افزار در `tower`](07-auth-and-security/03-jwt-and-tower-middleware/README.fa.md)
۴. [چرخشِ refresh token و باطل‌سازیش](07-auth-and-security/04-refresh-token-rotation-and-revocation/README.fa.md)
۵. [مدل‌سازیِ RBAC و مجوزها](07-auth-and-security/05-modelling-rbac-and-permissions/README.fa.md)

## ۸. [مدیریتِ خطا، ردیابی و تست در مقیاس](08-error-handling-and-testing-at-scale/README.fa.md)

همون چیزی که هر چی قبلش اومده رو قابلِ‌اجرا می‌کنه.

۱. [پاکت‌هایِ خطایِ یکدست](08-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.fa.md)
۲. [ردیابیِ درخواست و correlation ID](08-error-handling-and-testing-at-scale/02-request-tracing-and-correlation-ids/README.fa.md)
۳. [تستِ یکپارچگی با `testcontainers`](08-error-handling-and-testing-at-scale/03-integration-tests-with-testcontainers/README.fa.md)
۴. [factory و fixture برایِ داده‌یِ تست](08-error-handling-and-testing-at-scale/04-test-data-factories-and-fixtures/README.fa.md)
۵. [WebSocket و SSE در `axum`](08-error-handling-and-testing-at-scale/05-websockets-and-sse-in-axum/README.fa.md)

---

**ایستگاهِ روحیه‌دهی و انگیزشی:** [ماموریت جانبی ۳ — سرویس نوتیفیکیشنِ وبتون (Webtoon Notification Service)](../side-quests/sq-03-webtoon-notifier-service/README.fa.md)
— ترکیبی از `axum` + دیتابیس Postgres + یه جاب زمان‌بندی‌شده (scheduled job)، که پیش‌نمایشی از جاب‌های پس‌زمینه (background jobs) در فاز ۴ رو بهت میده.

**پیش‌نیاز:** نصب بودنِ PostgreSQL رو سیستم خودت (یا اجرا از طریق Docker) از ماژول ۵ به بعد — تو فایلِ `README.md` همون ماژول مراحل نصبش توضیح داده شده، تا قبل از اون هیچ نیازی بهش نیست.

وقتی فاز ۳ به طور کامل تو فایلِ [PROGRESS.md](../PROGRESS.md) تیک خورد و تموم شد، برو سراغ [فاز ۴](../phase4-backend-advanced/README.fa.md).

# فاز ۳ — زیربنای backend

متین، اینجا تجربه‌ی Django هم به کارت می‌آید و هم به چالش کشیده می‌شود. REST API، database و auth را از نظر مفهومی می‌شناسی؛ این فاز نشان می‌دهد framework زیر پوستش چه می‌کند—از TCP خام، پیش از نخستین import مربوط به `axum`.

۱. **شبکه و HTTP از صفر**: TCP echo server و parser دستی HTTP.
۲. **`axum` و طراحی REST API**: routing، handler، extractor و CRUD در حافظه.
۳. **serialization و validation**: `serde_json` و `validator`.
۴. **PostgreSQL و `sqlx`**: اتصال، connection pool، migration و catalog متصل به database.
۵. **طراحی database و کارایی query**: index، `EXPLAIN ANALYZE` و N+1.
۶. **auth و security**: hashکردن password، JWT و middleware مربوط به `tower`.
۷. **error handling و test در مقیاس**: envelope خطای یکدست و `testcontainers`.

```senpai-visual
{"kind":"network","labels":["TCP خام","HTTP","axum","PostgreSQL","API قابل‌اعتماد"]}
```

پیش‌نیاز شروع ماژول چهارم PostgreSQL محلی یا Docker است؛ درس‌های پیش از آن به آن نیاز ندارند. پس از تیک‌خوردن این فاز در `PROGRESS.fa.md`، به فاز چهار برو.

ایست انگیزشی: side quest شمارهٔ ۳، سرویس اعلان Webtoon، ترکیب `axum`، PostgreSQL و job زمان‌بندی‌شده است و کارهای پس‌زمینه‌ی فاز چهار را پیش‌نمایش می‌دهد.

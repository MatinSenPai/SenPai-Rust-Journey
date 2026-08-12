# ماژول ۵ — طراحی database و کارایی query

ماژول چهار اتصال به Postgres واقعی بود. اینجا تفاوت queryای که فقط «کار می‌کند» با queryای که روی دو میلیون row هم کار می‌کند را می‌بینی: خواندن `EXPLAIN ANALYZE`، فهم ارزش index و تشخیص N+1 پیش از production.

۱. [index، `EXPLAIN ANALYZE` و N+1](01-indexing-explain-analyze-n-plus-1/README.fa.md): B-tree، `Seq Scan` در برابر `Index Scan` و N+1 raw SQL.
۲. [pagination: offset در برابر keyset](02-pagination/README.fa.md): کندشدن `OFFSET 100000`، تکرار row هنگام write هم‌زمان و cursor با کلید مرکب `(created_at, id)`.

در پایان می‌توانی query کند را از plan توضیح بدهی، N+1 را بشناسی و table بزرگ را بدون drift صفحه‌بندی کنی.

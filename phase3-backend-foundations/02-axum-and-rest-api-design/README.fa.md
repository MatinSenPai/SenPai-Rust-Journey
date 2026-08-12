# ماژول ۲ — `axum` و طراحی REST API

ماژول یک HTTP server را از byte خام ساخت. حالا همان فهم را با `axum`، framework async این دوره، به کار می‌گیری؛ همان نسبتی که Django با protocol خام WSGI زیرش دارد.

۱. [routing، handler و extractor](01-routing-handlers-extractors/README.fa.md): `Router`، handler async و `Path`/`Json`/`State` به‌جای parsing دستی.
۲. [CRUD catalog انیمه در حافظه](02-anime-catalog-crud-in-memory/README.fa.md): API کامل create/read/update/delete که در ماژول چهار روی PostgreSQL بازسازی می‌شود.
۳. [CORS و اتصال frontend](03-cors-and-frontend-integration/README.fa.md): same-origin policy مرورگر، preflight `OPTIONS` و `CorsLayer` برای dev و production، بدون نیاز به browser در test.

```senpai-visual
{"kind":"network","labels":["Router","extractor","handler async","state","JSON response"]}
```

پایان این ماژول می‌توانی REST API واقعی با status code درست و error JSON یکدست بسازی؛ database در ماژول بعد وارد می‌شود.

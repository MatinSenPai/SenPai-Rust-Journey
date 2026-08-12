# ماژول ۱ — شبکه و HTTP از صفر

پیش از `axum` ببین زیر آن چه خبر است: socket خام TCP و یک protocol متنی. هر درخواست Django که تا امروز handle کرده‌ای، فرض می‌کرد server مربوط به WSGI یا ASGI این مرحله را انجام داده است. یک بار آن را دستی می‌سازی تا router، extractor و handler دیگر جادویی به‌نظر نرسند.

۱. [۰۱ — TCP echo server](01-tcp-echo-server/README.fa.md): `TcpListener`، `TcpStream` و حلقه‌ی read/write.
۲. [۰۲ — parser دستی HTTP](02-hand-rolled-http-parser/README.fa.md): جداکردن request line و header از byteهای خام.

در پایان ماژول می‌دانی زیر هر `@app.route` یا `Router::new()` چه اتفاقی می‌افتد؛ ماژول بعد `axum` را می‌آورد تا دیگر این کار را دستی نکنی.

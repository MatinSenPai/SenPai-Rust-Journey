# Module 1 — Networking & HTTP from scratch

Before `axum` (next module), see what it's actually built on: a raw TCP
socket and a text protocol. Every Django request you've ever handled
assumed a WSGI server had already done this work — this module does it by
hand once, so `axum`'s routing, extractors, and handlers stop feeling like
magic the moment you meet them.

1. [01 — TCP echo server](01-tcp-echo-server/README.md) — `TcpListener`,
   `TcpStream`, and the read/write loop every higher-level server is built on.
2. [02 — Hand-rolled HTTP parser](02-hand-rolled-http-parser/README.md) —
   parsing a request line and headers out of raw bytes, by hand.

By the end of this module you'll know exactly what a web framework is
doing underneath every `@app.route`/`Router::new()` call — next module
hands you `axum` to stop doing it by hand.

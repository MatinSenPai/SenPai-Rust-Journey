# Module 1 — Networking & HTTP from scratch

Before you ever import `axum`, see what it's built on. Every framework's
`Request`/`Response` abstraction sits on top of a TCP socket and a text
protocol you can read with your own eyes — this module builds both, by hand,
so the framework module right after this one reads as "the same thing,
automated" instead of as magic.

1. [01 — TCP echo server](01-tcp-echo-server/README.md)
2. [02 — Hand-rolled HTTP parser](02-hand-rolled-http-parser/README.md)
3. [03 — HTTP semantics you must know](03-http-semantics-you-must-know/README.md)
   — methods, status codes, headers, content negotiation, keep-alive, and
   chunked encoding: the parts of the spec every framework assumes you
   already know.

By the end, `axum`'s `Router` and extractors in the next module will read as
a shortcut for exactly this, not as a separate thing.

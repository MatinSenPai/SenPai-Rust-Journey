# Checkpoint

1. `prod_cors` is configured for `https://anime.example.com`, and a
   preflight arrives from `https://evil.example.com`. What status code
   does the server answer with, and which header is (deliberately)
   missing? Explain why the answer is not `403` — *who* actually stops
   the evil page from reading the response, and at what moment?
2. Which of these requests triggers a preflight, and why: (a) a plain
   `GET /anime` with no custom headers, (b) a `POST /anime` with
   `Content-Type: application/json`, (c) a `GET /anime` with an
   `Authorization: Bearer ...` header?
3. `app` registers no `OPTIONS` route anywhere, yet every preflight in
   `tests/cors_test.rs` gets a `200` with the right headers back. Where
   is the `OPTIONS` request being answered, and what does that tell you
   about the order in which the layer and the router see a request?
4. Why do the spec (and `tower-http`, via a deliberate panic) forbid
   combining an `Any` origin with `allow_credentials(true)`? Describe the
   attack that combination would enable.
5. A teammate ships `dev_cors()` to production "temporarily." The API
   has no cookie-based auth — only `Authorization` headers the frontend
   attaches by hand. What does the wildcard still expose in that setup,
   and why is the exact-origin `prod_cors` the right call anyway?

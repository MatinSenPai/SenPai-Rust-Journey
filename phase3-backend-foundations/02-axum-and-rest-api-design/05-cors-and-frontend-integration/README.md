# 3.2.5 — CORS and frontend integration

## At a glance

After this lesson you can:

- Explain why a perfectly healthy API shows up in the browser console as "blocked by CORS policy", and say which side (the browser or the server) enforces the rule.
- Describe a preflight `OPTIONS` request, say when a browser sends one, and show with a test that your handler never sees it.
- Build a dev and a prod `CorsLayer`, and know which configuration `tower-http` refuses to build at all.
- Test a CORS policy with `oneshot` and simulate a preflight with `curl`, with no browser involved.

**Time:** ~60 minutes · **Prerequisites:**
[3.2.4 — `tower::Service` / `Layer`: middleware by hand](../04-tower-service-and-layer-middleware/README.md),
[3.2.1 — Routing, handlers, extractors](../01-routing-handlers-extractors/README.md),
[3.1.3 — HTTP semantics you must know](../../01-networking-and-http-from-scratch/03-http-semantics-you-must-know/README.md)

---

## Why this matters

Everything you have built in this module passes its tests and answers `curl`. Then someone points a React or Vite page at it, and the browser console fills with red. Nothing is wrong with your Rust. This is the first problem that only exists because a browser is in the loop, and nearly every API with a web frontend hits it in its first week.

Django folks know the same wall as `django-cors-headers`: a `CORS_ALLOWED_ORIGINS` setting plus one entry in `MIDDLEWARE`. In `axum` it is `tower-http`'s `CorsLayer`, attached with `.layer(...)`. After [3.2.4](../04-tower-service-and-layer-middleware/README.md), where you wrote a `Layer` and a `Service` by hand, `CorsLayer` is not magic: it is a ready-made layer, exactly like the one you wrote, and one of the things it does is skip the inner service on purpose. This lesson closes the module by using that.

---

## The concept

### The bug that isn't a bug

`examples/01-no-cors-layer.rs` is a router with no CORS layer. It receives the request a page on `http://localhost:5173` would send:

```text
GET /anime -> 200 OK
  access-control-allow-origin: None
  body: [{"id":1,"title":"Frieren"}]
OPTIONS /anime -> 405 Method Not Allowed
```

The server did its job: `200`, a body, no error. The only thing missing is a header. The page's JavaScript never sees that body, because the **browser** compares the response against the page's origin, finds no header vouching for it, and withholds it. The same-origin policy is enforced by the browser, not by the server. The server never "blocks" anyone; it declines to *vouch*, and the browser does the blocking.

The second line is the other half of the story: before a JSON `POST`, the browser sends an `OPTIONS` request, and with no layer in place it gets `405`. You never wrote a handler for it, and (as you will see) you should not have to.

### Origins, and who enforces the rule

An **origin** is the triple *scheme + host + port*. Relative to `http://localhost:3000`, each of these is a different origin:

| URL | Why it differs |
|---|---|
| `https://localhost:3000` | scheme (`https` vs `http`) |
| `http://localhost:5173` | port |
| `http://127.0.0.1:3000` | host (`localhost` and `127.0.0.1` are different strings) |

Since the browser is the enforcer, everything that is not a browser ignores the whole thing: `curl`, your `oneshot` tests, another backend service. CORS protects *users* (their cookies, their logged-in sessions) from malicious *websites*. It was never access control for your API: if you can `curl` it, so can everyone.

### Preflight: the OPTIONS request you never wrote a handler for

For a "simple" request (`GET`, `HEAD`, or a `POST` with a form-style content type) the browser just sends it with an `Origin` header and checks the response afterwards. Anything else gets a **preflight** first: the browser sends `OPTIONS` carrying `Origin`, `Access-Control-Request-Method` and, if custom headers are involved, `Access-Control-Request-Headers`. It sends the real request only if the answer approves. That covers:

- methods beyond the simple set: `PUT`, `PATCH`, `DELETE`
- custom request headers: `Authorization`, `X-Api-Key`
- **`Content-Type: application/json`**, which is not a "simple" content type

The last one is the punchline: every JSON `POST` your API accepts gets preflighted. The answer rides on three response headers:

| Header | Meaning | Appears on |
|---|---|---|
| `access-control-allow-origin` | which origin may read the response | preflight and real responses |
| `access-control-allow-methods` | which methods the real request may use | preflight only |
| `access-control-allow-headers` | which request headers it may carry | preflight only |

Why can a browser send a preflight without asking anyone's permission? Because `OPTIONS` is a **safe method** in [3.1.3](../../01-networking-and-http-from-scratch/03-http-semantics-you-must-know/README.md)'s table: it is not expected to change anything on the server, so asking "would you accept this?" is harmless by definition.

### The layer answers it, and the handler never runs

Here is a policy for one frontend, wrapped around a router that has a counter in its `POST` handler (`examples/02-preflight-never-reaches-the-handler.rs`):

```rust
let cors = CorsLayer::new()
    .allow_origin(AllowOrigin::list([HeaderValue::from_static(
        "http://localhost:5173",
    )]))
    .allow_methods([Method::GET, Method::POST])
    .allow_headers([header::CONTENT_TYPE]);
let app = Router::new()
    .route("/anime", post(create_anime))
    .with_state(hits.clone())
    .layer(cors);
```

It sends the preflight first, then the real request:

```text
preflight -> 200 OK
  access-control-allow-origin: "http://localhost:5173"
  access-control-allow-methods: "GET,POST"
  access-control-allow-headers: "content-type"
  body: 0 bytes
  handler ran 0 times
real POST -> 201 Created
  access-control-allow-origin: "http://localhost:5173"
  handler ran 1 times
```

Same router, same layer; the counter stays at zero after the preflight and moves only on the real `POST`. `CorsLayer` is a `Layer` like the one you wrote in 3.2.4, and on an `OPTIONS` request its `Service` does what your short-circuiting middleware did: it builds the response itself and returns it **without calling the inner service**. The preflight never reaches your router or your handler. That is why no `options(...)` route has to exist, and why the body is empty.

```senpai-visual
{"kind":"network","labels":["browser: OPTIONS preflight","CorsLayer answers, handler never reached","browser checks the allow headers","browser sends the real request","handler runs, CorsLayer adds allow-origin","browser allows or blocks the page"]}
```

Notice the real response carries `access-control-allow-origin` but not the other two: they are preflight-only. Also notice that a response vouching for one specific origin must differ per request, so `tower-http` adds `vary: origin, access-control-request-method, access-control-request-headers`. That keeps a cache from serving one origin's answer to another.

### The one forbidden configuration

`Access-Control-Allow-Origin: *` cannot be combined with `Access-Control-Allow-Credentials: true`. The combination would mean "any website on the internet may send this user's cookies to your API and read the answer", which is session hijacking as a config option. Browsers reject it, and `tower-http` refuses to build it: the layer panics when you apply it. "Errors you will meet" shows the real panic.

### Dev posture, prod posture

- **Dev: permissive.** Origins churn: Vite picks another port, a teammate uses `127.0.0.1`, a phone on the LAN loads the UI. Allow everything, locally.
- **Prod: locked.** Exactly the origin the frontend is served from, exactly the methods and request headers it uses. A wildcard in prod means any page a user visits can read your API's responses from inside their browser.

Choosing between the two belongs to configuration, and a hard-coded string in `main` is only a stand-in for it. [3.4.1 — 12-factor config and secrets](../../04-configuration-and-app-structure/01-config-and-secrets/README.md) is where the allowed origins stop being hard-coded.

### Testing CORS without a browser

A preflight is just an HTTP request: `OPTIONS` plus a few headers. So the `oneshot` technique from [3.2.1](../01-routing-handlers-extractors/README.md) can fabricate one and assert on the `access-control-*` response headers directly:

```rust
fn preflight(origin: &str, request_headers: &str) -> Request<Body> {
    Request::builder()
        .method("OPTIONS")
        .uri("/anime")
        .header("origin", origin)
        .header("access-control-request-method", "POST")
        .header("access-control-request-headers", request_headers)
        .body(Body::empty())
        .unwrap()
}
```

`tests/cors_test.rs` pins your whole policy this way, including the negative case: an unknown origin still gets `200`, with no `access-control-allow-origin`, because declining to vouch is not an error.

---

## Hands on

```sh
cargo run -p p3-02-05-cors-and-frontend-integration --example 01-no-cors-layer
cargo run -p p3-02-05-cors-and-frontend-integration --example 02-preflight-never-reaches-the-handler
```

Both print what is shown in "The concept". Now a real socket. `03-serve-with-cors` serves the API on port 3002 for the single origin `http://localhost:5173`; run it in one terminal and leave it running:

```sh
cargo run -p p3-02-05-cors-and-frontend-integration --example 03-serve-with-cors
```

In a second terminal, simulate what a browser sends before a JSON `POST`:

```sh
curl -si -X OPTIONS http://127.0.0.1:3002/anime \
  -H "Origin: http://localhost:5173" \
  -H "Access-Control-Request-Method: POST" \
  -H "Access-Control-Request-Headers: content-type"
```

```text
HTTP/1.1 200 OK
vary: origin, access-control-request-method, access-control-request-headers
access-control-allow-methods: GET,POST
access-control-allow-headers: content-type
access-control-allow-origin: http://localhost:5173
allow: GET,HEAD,POST
content-length: 0
date: Sun, 04 Oct 2026 10:13:26 GMT

```

Those `access-control-allow-*` lines are everything the browser needs. (The `allow:` line is `axum`'s own method list, added after the layer answered; the handler still did not run. The `date` changes every run.) Now change `Origin` to `https://evil.example.com` and run it again: the status is still `200` and `access-control-allow-origin` is gone. Stop the server with Ctrl+C.

Then the two broken ones. The first needs the `broken` feature; the second runs normally and is simply wrong:

```sh
cargo run -p p3-02-05-cors-and-frontend-integration --example 04-credentials-with-wildcard-broken --features broken
cargo run -p p3-02-05-cors-and-frontend-integration --example 05-origin-that-never-matches-trap
```

Then try these:

1. In `02-preflight-never-reaches-the-handler`, change the preflight's `access-control-request-method` to `DELETE`. What does `access-control-allow-methods` say now, and what does that mean for the browser?
2. In `01-no-cors-layer`, add `.layer(CorsLayer::permissive())` to the router. What do the two lines say now?

---

## Errors you will meet

### A run-time panic: credentials with a wildcard origin

```text
layer built, applying it to the router...

thread 'main' (31804) panicked at C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\tower-http-0.6.11\src\cors\mod.rs:797:9:
Invalid CORS configuration: Cannot combine `Access-Control-Allow-Credentials: true` with `Access-Control-Allow-Origin: *`
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

(The number in parentheses is the thread's id and changes every run; the path is where Cargo unpacked `tower-http` on this machine.)

**What's actually wrong:** `.allow_origin(Any).allow_credentials(true)` is "any website may send cookies and read the answer". Building the layer succeeds (the first line printed). The panic comes when the layer is applied with `.layer(...)`, which is while the app is being assembled, before the first request. That is a good place to fail: you find it at startup, not in production.

**The fix:** decide who may use credentials and name them:

```rust
CorsLayer::new()
    .allow_origin(AllowOrigin::list([HeaderValue::from_static(
        "https://anime.example.com",
    )]))
    .allow_credentials(true)
```

**Why this is the fix:** with credentials, the browser must be told exactly which origin may read the response. The same rule applies to `allow_methods` and `allow_headers`: they cannot be `Any` either when credentials are on.

### The preflight that lacks your header

```text
HTTP/1.1 200 OK
vary: origin, access-control-request-method, access-control-request-headers
access-control-allow-methods: GET,POST
access-control-allow-headers: content-type
access-control-allow-origin: http://localhost:5173
allow: GET,HEAD,POST
content-length: 0
date: Sun, 04 Oct 2026 10:13:26 GMT

```

This is the response to the same preflight as in "Hands on", except that the request now asks `Access-Control-Request-Headers: content-type, authorization`. The answer is byte-for-byte the same: `access-control-allow-headers` says `content-type` and nothing else.

**What's actually wrong:** your frontend adds an `Authorization` header, and your policy never allowed it. There is no error, no log line, and the status is `200`. The browser compares the header it wants to send with the list it got back, sees `authorization` missing, and cancels the real request. In the console you see a CORS failure, and the network tab shows a successful `OPTIONS` and no `POST` at all.

**The fix:** add the header to the list the policy allows (`header::AUTHORIZATION` next to `header::CONTENT_TYPE`), and re-run the preflight to see it appear.

**Why this is the fix:** the server never rejects a header. It publishes a list, and the browser holds the request against it. So the way to debug this is exactly what you just did: look at the preflight's response, not at your handler.

### No error at all: an origin that never matches

```text
allowed "https://anime.example.com/", Origin "https://anime.example.com"
  -> 200 OK, access-control-allow-origin: None
allowed "http://anime.example.com", Origin "https://anime.example.com"
  -> 200 OK, access-control-allow-origin: None
```

**What's actually wrong:** `examples/05-origin-that-never-matches-trap.rs` compiles, runs, and answers `200` every time. Both allowed origins were typed by hand. One has a trailing slash, the other the wrong scheme. A browser's `Origin` header is exactly scheme, host and port: no path, no trailing `/`. The policy compares bytes, so neither ever matches, and every browser request fails with no hint on the server side.

**The fix:** write the origin the way a browser sends it:

```rust
AllowOrigin::list([HeaderValue::from_static("https://anime.example.com")])
```

**Why this is the fix:** the comparison is exact, so your configuration has to be exact. This is also why `prod_cors_from_list` in the Build exercise rejects an entry that ends with `/`: a typo that can never match is better caught at startup than found by a user.

---

## Exercises

### Warm up

<details>
<summary>Your API answers <code>curl</code> perfectly but the browser console says "blocked by CORS policy". Which side blocked it?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

The browser. The server produced and sent the response; the browser withheld it from the page's JavaScript because no `access-control-allow-origin` header vouched for that page's origin. The server never blocks, it only declines to vouch.

</details>

<details>
<summary>A page calls <code>fetch</code> with a <code>GET</code> and no custom headers, then a <code>POST</code> with <code>Content-Type: application/json</code>. Which one is preflighted?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

Only the `POST`. A plain `GET` is a simple request, sent directly with an `Origin` header. `application/json` is not a simple content type, so the `POST` is preceded by an `OPTIONS` preflight.

</details>

<details>
<summary>Why is <code>allow_origin(Any)</code> together with <code>allow_credentials(true)</code> forbidden?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

Together they mean any website on the internet may send the logged-in user's cookies to your API and read the responses. The spec forbids it, and `tower-http` panics when you apply such a layer instead of letting it run.

</details>

<details>
<summary>An unknown origin sends a preflight to your prod layer. What status comes back, and why?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

`200 OK`, with no `access-control-allow-origin`. The server is not in the blocking business; it answers honestly and leaves the decision to the browser. Also, the handler never runs for this request, because the layer answers every `OPTIONS` itself.

</details>

### Repair

Fix both broken examples:

1. `examples/04-credentials-with-wildcard-broken.rs` builds the router without panicking, and still allows credentials, for one named origin.
2. `examples/05-origin-that-never-matches-trap.rs` prints an `access-control-allow-origin` header for the browser's origin in both cases.

### Implement

Two functions in `src/lib.rs`:

```sh
cargo test -p p3-02-05-cors-and-frontend-integration
```

Each is fully specified in its doc comment (which origins, which methods, which headers, whether credentials), so you should not need to read the tests:

- `dev_cors`: everything allowed, with the `*` wildcard.
- `prod_cors`: one origin, `GET` and `POST`, `content-type` only. It panics on an origin that is not a valid header value.

### Build

`prod_cors_from_list`: the same policy for several frontends, read from one comma-separated string like `"https://anime.example.com, http://localhost:5173"`, the shape an `ALLOWED_ORIGINS` setting usually has. It trims entries, skips empty ones, and panics on an entry ending in `/`, naming the entry. Three more tests in the same `cargo test` run check it.

### Challenge (optional)

JavaScript can only read a few "safe" response headers. Add a header such as `x-total-count` to a handler's response, then make `tower-http` expose it with `CorsLayer`'s `expose_headers`. Write a test of your own, in a new file under `tests/`, that sends a simple request with an `Origin` header and asserts on `access-control-expose-headers`. Does a preflight carry that header too?

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Origin | scheme + host + port | deciding who your frontend is |
| Same-origin policy | the browser withholds a cross-origin response unless the server vouches | every web frontend |
| CORS | the headers a server uses to vouch for an origin | a `CorsLayer` on every browser-facing API |
| Preflight | an `OPTIONS` request the browser sends before a non-simple request | any JSON `POST`, `PUT`, `DELETE`, or custom header |
| `CorsLayer` | a ready-made `Layer` that answers preflights itself | `.layer(...)` on your router |
| Credentials | cookies attached to a cross-origin request | cookie sessions, never with a wildcard |

### What you now know

- The browser enforces the same-origin policy. The server only vouches or declines, and `curl` and `oneshot` ignore the whole thing.
- A preflight is an `OPTIONS` request, safe in 3.1.3's sense, answered by `CorsLayer` without ever calling your router or handler.
- `*` with credentials is forbidden, and `tower-http` panics instead of building it.
- A header your policy did not list silently fails in the browser, and an origin with a trailing slash or the wrong scheme never matches.
- You can test a whole CORS policy with `oneshot`, and simulate a preflight with `curl`.

### What comes back later

- **Origins that come from the environment, not from code** — [3.4.1 — 12-factor config and secrets](../../04-configuration-and-app-structure/01-config-and-secrets/README.md)
- **Why cookies need `allow_credentials`, and the alternative to them** — [3.7.2 — Sessions vs. JWT: the real trade-off](../../07-auth-and-security/02-sessions-vs-jwt/README.md)
- **Request and response structs, now with real field rules** — [3.3.1 — Serde in depth](../../03-serialization-and-validation/01-serde-depth/README.md)
- **The same endpoints over a real database** — [Module 5 — PostgreSQL & `sqlx`](../../05-postgres-and-sqlx/README.md)

Looking back over the module: [3.2.1](../01-routing-handlers-extractors/README.md) gave you routes, handlers and extractors; [3.2.2](../02-writing-your-own-extractor/README.md) let you write your own extractor; [3.2.3](../03-anime-catalog-crud-in-memory/README.md) built a full resource and turned errors into responses; [3.2.4](../04-tower-service-and-layer-middleware/README.md) opened up `.layer(...)`; and this lesson used a ready-made layer, tested without a socket. Module 3 starts with [serde](../../03-serialization-and-validation/01-serde-depth/README.md), and module 5 puts a database behind these endpoints.

### Can you explain?

- Why does the server answer `200` to a request the browser then refuses to show the page?
- Why is a JSON `POST` always preflighted, and why can the browser send the preflight at all?
- How can you prove, with a test, that a preflight never reaches your handler?
- Why is `*` plus credentials forbidden, and when does `tower-http` panic over it?
- Why does a missing `authorization` in `access-control-allow-headers` produce no error on the server?

---

## Going further

- [MDN — Cross-Origin Resource Sharing (CORS)](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS): the whole mechanism from the browser's side, including which requests are "simple".
- [Fetch Standard — CORS protocol](https://fetch.spec.whatwg.org/#http-cors-protocol): the spec that defines the headers you configured today.
- [`tower_http::cors`](https://docs.rs/tower-http/0.6.11/tower_http/cors/index.html): every builder method of `CorsLayer`, including `max_age` and `AllowOrigin::predicate`.
- [Module 2 index](../README.md): the five lessons in order.

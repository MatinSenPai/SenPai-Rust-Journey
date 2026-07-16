# 02.3 — CORS and frontend integration

## The bug that isn't a bug

Lesson 02's catalog API passes every test and answers every `curl`. Now
point a frontend at it — a Vite/React dev server on
`http://localhost:5173` calling `fetch("http://localhost:3001/anime")` —
and the browser console fills with *"blocked by CORS policy."*

Nothing in your Rust code failed. The response was produced, sent — and
then **the browser refused to hand it to the page's JavaScript**. That's
the one mental model this lesson exists to install: the **same-origin
policy is enforced by the browser, not the server**. The server never
"blocks" anything; it just declines to *vouch* for a cross-origin caller,
and the browser, seeing no vouching headers, withholds the response.

An **origin** is the triple *scheme + host + port*. Relative to
`http://localhost:3000`, every one of these is a different origin:

| URL | Why it's different |
|---|---|
| `https://localhost:3000` | scheme (`https` vs `http`) |
| `http://localhost:5173` | port |
| `http://127.0.0.1:3000` | host — yes, really; `localhost` and `127.0.0.1` are different strings |

The flip side: `curl`, your `oneshot` tests, another backend service —
none of them enforce any of this. CORS protects *users* (their cookies,
their logged-in sessions) from malicious *websites*; it was never access
control for your API. If you can `curl` it, so can everyone.

Django folks have met this wall as `django-cors-headers`:
`CORS_ALLOWED_ORIGINS` in settings plus a `MIDDLEWARE` entry. The `axum`
equivalent is `tower_http::cors::CorsLayer`, attached to the `Router`
with `.layer(...)` — the same middleware slot Django's list fills.

## Preflight: the OPTIONS request you never wrote a handler for

For a small class of "simple" requests (`GET`/`HEAD`, or `POST` with
form-ish content types), the browser just sends the request with an
`Origin` header and checks the response's CORS headers after the fact.

Anything else triggers a **preflight**: before the real request, the
browser sends `OPTIONS` carrying `Origin`,
`Access-Control-Request-Method`, and (when custom headers are involved)
`Access-Control-Request-Headers` — and only sends the real request if the
preflight response approves. What counts as "anything else":

- methods beyond the simple set: `PUT`, `PATCH`, `DELETE`
- custom request headers: `Authorization`, `X-Api-Key`, …
- **`Content-Type: application/json`** — not a "simple" content type!

That last one is the punchline: every JSON `POST`/`PATCH` your lesson-02
API accepts gets preflighted by a browser. A JSON API with no CORS
configuration is simply unreachable from cross-origin frontend JS.

The server's answer rides on three response headers:

| Header | Meaning | Appears on |
|---|---|---|
| `access-control-allow-origin` | which origin may read responses | preflight *and* real responses |
| `access-control-allow-methods` | which methods the real request may use | preflight only |
| `access-control-allow-headers` | which request headers it may carry | preflight only |

One `axum` detail worth noticing: you never write
`.route("/anime", options(...))`. `CorsLayer` intercepts preflight
`OPTIONS` requests and answers them *itself*, before they ever reach the
router — which is why preflight responses in this lesson's tests have
empty bodies: no handler of yours ran.

## The one configuration that's flat-out forbidden

`Access-Control-Allow-Origin: *` cannot be combined with
`Access-Control-Allow-Credentials: true`. The spec forbids it because the
combination means "any website on the internet may send requests carrying
this user's cookies *and read the responses*" — session hijacking as a
config option. `tower-http` doesn't trust you to never ship it: combining
`Any` with `.allow_credentials(true)` **panics** instead of becoming a
silent security hole.

## Dev posture vs. prod posture

- **Dev: permissive.** Origins churn constantly — Vite picks a new port,
  a teammate uses `127.0.0.1`, a phone on the LAN loads the UI. Fighting
  your own tooling buys nothing; allow everything, locally.
- **Prod: locked.** Exactly the origin your frontend is served from,
  exactly the methods and headers it uses. A wildcard in prod means any
  webpage a user visits can probe your API from inside their browser.

That fork is precisely the exercise: two functions, `dev_cors()` and
`prod_cors(allowed_origin)`, each returning a `CorsLayer`. A real service
picks one at startup from its environment — exactly the kind of decision
Phase 4's config lesson turns into first-class configuration.

## Testing CORS without a browser

The practical payoff of the mental model: a preflight is *just an HTTP
request* — `OPTIONS` plus two headers. So `tower::ServiceExt::oneshot`
from the last two lessons can fabricate one, and you can assert on the
`access-control-allow-*` response headers directly. No browser, no
frontend project: `tests/cors_test.rs` pins down your entire CORS posture
in milliseconds — including the negative case, where an unknown origin
gets a `200` with *no* allow-origin header, because declining to vouch is
not an error.

## Your task

Implement the two `todo!()`s in `src/lib.rs`:

- `dev_cors()` — any origin, any method, any header.
- `prod_cors(allowed_origin: &str)` — exactly one origin, only `GET` and
  `POST`, only the `content-type` header.

The two JSON endpoints and `app(cors: CorsLayer)` are provided — the
router isn't the lesson, the layer wrapped around it is.

## Try it for real

```sh
cargo run -p p3-02-03-cors-and-frontend-integration &
# simulate a browser preflight by hand:
curl -i -X OPTIONS http://127.0.0.1:3002/anime \
  -H 'Origin: http://localhost:5173' \
  -H 'Access-Control-Request-Method: POST' \
  -H 'Access-Control-Request-Headers: content-type'
```

Look for the `access-control-allow-*` headers in the response — they're
everything a browser needs to see before letting a page send the real
`POST`.

## Checkpoint

`cargo test -p p3-02-03-cors-and-frontend-integration`, then
`CHECKPOINT.md`, then `solution/SOLUTION.md`.

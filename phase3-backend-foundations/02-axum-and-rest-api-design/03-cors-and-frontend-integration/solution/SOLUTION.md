# Solution

```rust
pub fn dev_cors() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}
```

`Any` is a unit struct from `tower_http::cors`, and it's a *type-level*
choice, not a string: `allow_origin` accepts `impl Into<AllowOrigin>`, and
`Any`, a `HeaderValue`, and a list of `HeaderValue`s all convert into it.
`Any` becomes a literal `*` in the response headers — which is why the
dev tests assert `access-control-allow-origin: *` exactly.
`CorsLayer::permissive()` is this same three-liner prebuilt; writing it
out once makes it obvious what "permissive" actually grants.

```rust
pub fn prod_cors(allowed_origin: &str) -> CorsLayer {
    let origin = allowed_origin
        .parse::<HeaderValue>()
        .expect("invalid allowed origin");

    CorsLayer::new()
        .allow_origin(origin)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE])
}
```

Three decisions worth defending:

- **`.expect` on the parse.** The allowed origin comes from configuration,
  and a bad one is a deployment mistake, not a runtime condition to limp
  through. Crashing at startup with "invalid allowed origin" is strictly
  better than a healthy-looking process whose every browser client
  silently fails — the same fail-fast instinct Phase 4's config lesson
  builds a whole `Config` type around.
- **An exact `HeaderValue`, not `Any`.** With an exact origin, tower-http
  compares the request's `Origin` header against it: on a match, the
  response carries `access-control-allow-origin: https://anime.example.com`
  (the exact value, never `*`); on a mismatch, the header is simply
  *omitted*. The response is still `200` — the server declines to vouch,
  and the browser does the actual blocking. That's the whole
  browser-enforced model, visible in one missing header.
- **Only `GET`/`POST` and only `content-type`.** The provided router only
  serves `GET` and `POST`, and the only non-simple thing a JSON frontend
  needs is the `content-type` header. Granting exactly what's used means a
  future `DELETE /anime/{id}` endpoint *fails closed* from browsers until
  someone consciously adds `Method::DELETE` here — a much better default
  than a wildcard quietly pre-approving methods that don't exist yet.

## On the checkpoint questions

**Q1 (unknown origin):** `200 OK`, with no `access-control-allow-origin`
header. Not `403`, because the server isn't in the blocking business at
all — it answered the preflight honestly ("here's who I vouch for:
nobody you know"). The *browser* stops the evil page, at the moment it
inspects the preflight response and finds no header vouching for
`https://evil.example.com`. The real request is never even sent.

**Q2 (what preflights):** (a) no — `GET` with no custom headers is a
simple request; the browser sends it directly with an `Origin` header and
checks the response after. (b) yes — `application/json` is not one of the
three simple content types (`form-urlencoded`, `multipart/form-data`,
`text/plain`), so even a plain `POST` gets preflighted. (c) yes — any
custom header, `Authorization` included, disqualifies a request from
being simple regardless of method.

**Q3 (who answers OPTIONS):** The `CorsLayer` itself. Middleware wraps
the router, so the layer sees every request *before* routing happens; on
a preflight it builds the response and returns it without ever calling
the inner service. That's why no handler runs (empty body) and why no
`options(...)` route needs to exist.

**Q4 (Any + credentials):** Credentials mean the browser attaches the
user's cookies to cross-origin requests. Combined with a wildcard origin,
any webpage the user visits could call your API *as that logged-in user*
and read the responses — silently exfiltrating whatever their session can
see. The spec forbids the combination outright, and tower-http panics
rather than let the config exist at runtime.

**Q5 (wildcard without cookies):** With no cookie auth, the wildcard
doesn't hand out user sessions — but it still lets any webpage on the
internet make browser-side requests to every endpoint and read every
response, which matters the moment any endpoint is unauthenticated,
IP-allowlisted, or reachable from an internal network the visitor's
browser happens to sit inside (a classic way to probe intranets). And
CORS posture tends to outlive auth design: the day someone adds a cookie
or relaxes a check, the wildcard is already there waiting. Locking to the
one real frontend origin costs nothing and removes the entire class.

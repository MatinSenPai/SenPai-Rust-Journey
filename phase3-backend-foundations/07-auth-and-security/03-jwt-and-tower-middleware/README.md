# 3.7.3 — JWTs and `tower` middleware

## At a glance

After this lesson you can:

- Take a JWT apart by hand into header, payload and signature, and say exactly what the signature does and does not protect.
- Issue and verify HS256 tokens with `jsonwebtoken` 9: pin the algorithm, require `exp`, and test expiry and leeway with an injected clock instead of the real one.
- Gate `axum` routes with `from_fn_with_state` middleware that hands the verified identity to handlers through request extensions, and tell `401` from `403`.
- Name the JWT mistakes that actually get services broken into, and show which line of this lesson's code stops each one.

**Time:** ~100 minutes · **Prerequisites:**
[3.2.4 — `tower::Service` and `Layer`: middleware by hand](../../02-axum-and-rest-api-design/04-tower-service-and-layer-middleware/README.md),
[3.2.1 — Routing, handlers, extractors](../../02-axum-and-rest-api-design/01-routing-handlers-extractors/README.md),
[3.7.1 — Password hashing with `argon2`](../01-password-hashing-argon2/README.md),
[3.7.2 — Sessions vs. JWT: the real trade-off](../02-sessions-vs-jwt/README.md)

---

## Why this matters

[3.7.1](../01-password-hashing-argon2/README.md) answered "is this really you?" once, at login. Every request after that needs the same question answered again, and sending the password each time is out of the question. A session answers it by making the server remember. A **JWT** (JSON Web Token) answers it by making the client carry the proof: the server signs a small statement, "this is `user-42`, good until 3 pm", hands it over at login, and later only has to check its own signature. [3.7.2](../02-sessions-vs-jwt/README.md) argues when each approach is the right one. This lesson assumes you chose the JWT and builds it properly.

In Django you know the other half. `rest_framework_simplejwt` has a `JWTAuthentication` class that reads `Authorization: Bearer ...`, verifies the token and sets `request.user`. You never saw the verification, and most JWT breaches live inside it: a verifier that trusts the token's own `alg` field, a token with no expiry, a secret that sits in the repository. In `axum` there is no framework class to hide behind. You write the middleware, so this lesson makes you write it once and see every decision in it.

It also pays off the promise at the end of [3.2.4](../../02-axum-and-rest-api-design/04-tower-service-and-layer-middleware/README.md): "a middleware that rejects unauthenticated requests". There you wrote `Layer` and `Service` by hand. Here you use the shortcut it ended on, `from_fn`, because auth middleware belongs to your app and needs nothing the shortcut lacks.

---

## The concept

### A JWT is three base64url pieces joined by dots

`examples/01-anatomy-of-a-jwt.rs` signs one token with fixed numbers (so the output never changes) and then cuts it apart with nothing but `split('.')` and a base64url decoder. No key is used to read it:

```rust
let claims = Claims {
    sub: "user-42".to_string(),
    iat: 1_700_000_000,
    exp: 1_700_003_600,
};
let key = EncodingKey::from_secret(b"demo-secret-for-3-7-3");
let token = encode(&Header::default(), &claims, &key).unwrap();
let parts: Vec<&str> = token.split('.').collect();
```

```text
token:     eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ1c2VyLTQyIiwiaWF0IjoxNzAwMDAwMDAwLCJleHAiOjE3MDAwMDM2MDB9.E2gWSvvYZVXaJTL1pz8Iu8meUGFH0SK4zZewTyYtefI
parts:     3
header:    {"typ":"JWT","alg":"HS256"}
payload:   {"sub":"user-42","iat":1700000000,"exp":1700003600}
signature: 43 base64url characters (32 bytes of HMAC-SHA256)
```

That is the whole format, from RFC 7519. The first piece is the **header**: which algorithm signed this token. The second is the **payload**, whose fields are called **claims**: `sub` ("subject": who the token is about), `iat` ("issued at") and `exp` ("expires at"), the last two in whole seconds since the Unix epoch. The third is the **signature**: an HMAC-SHA256 over the first two pieces joined by a dot, computed with a secret only the server knows.

```senpai-visual
{"kind":"concept","labels":["header: alg and typ, base64url","payload: sub, iat, exp, base64url","signature: HMAC-SHA256 of header dot payload","joined with dots: the whole token","only the signature needs the secret"]}
```

### Signed, not encrypted

Look at the output again. The header and the payload came out as plain JSON, and the only thing used to produce them was a decoder with no key. **Base64url is an encoding, not encryption.** Anyone who holds the token can read every claim. The signature gives a different guarantee: if someone changes one character of the payload, the signature no longer matches, and the server can tell.

So a JWT is a tamper-evident note, not a locked box. Never put a password, an API key or anything private in the claims. A user id, a role and an expiry are fine. This is the first of the real pitfalls, and the reason `Claims` in `src/lib.rs` carries a doc comment saying so.

Django bridge, and where it stops: a Django session cookie also holds an opaque id that means nothing to the browser. A JWT is the opposite: the client can read the contents, and the server stores nothing. The cost of that is the next problem. The server cannot take a token back, because it never kept a list. [3.7.4](../04-refresh-token-rotation-and-revocation/README.md) deals with that.

### Verifying with `jsonwebtoken`: a `Validation` is the checklist

`jsonwebtoken` 9.3.1 does the cryptography. You hand `decode` the token, a key, and a `Validation` that says what to insist on:

```rust
let key = DecodingKey::from_secret(SECRET);
let validation = Validation::new(Algorithm::HS256);
decode::<serde_json::Value>(token, &key, &validation)
```

`examples/02-validation-knobs.rs` runs `decode` against eight tokens and prints what came back:

```text
default leeway: 60 s
valid until 2100         ok, sub = "user-42"
expired in 1970          ExpiredSignature
expired 30 s ago         ok, sub = "user-42"
30 s ago, leeway 0       ExpiredSignature
HS512, HS256 pinned      InvalidAlgorithm
HS512, both allowed      ok, sub = "user-42"
no exp claim             MissingRequiredClaim("exp")
alg none                 Json(Error("unknown variant `none`, expected one of `HS256`, `HS384`, `HS512`, `ES256`, `ES384`, `RS256`, `RS384`, `RS512`, `PS256`, `PS384`, `PS512`, `EdDSA`", line: 1, column: 13))
```

Each line is one field of `Validation::new(Algorithm::HS256)`:

- `exp` is required and checked. A token with no `exp` is refused (`MissingRequiredClaim`), and one in the past is `ExpiredSignature`. A token that never expires is the second real pitfall, and `jsonwebtoken` already refuses it by default.
- `leeway` defaults to **60 seconds**. The token that expired 30 seconds ago was still accepted, until the example set `leeway` to 0. Leeway exists because two machines never agree exactly on the time (clock skew). It also means every `exp` is really `exp + 60` unless you say otherwise.
- `algorithms` is a list, and it starts as just the one you passed. A token whose header names something else is `InvalidAlgorithm`.

### Pin the algorithm; `alg: none` is not an option

The header says which algorithm signed the token, and the token comes from the client. A verifier that does what the header says is letting the attacker choose how they are checked. That is the classic JWT attack, in two forms. The first is `alg: none`: the header claims "no signature needed", and a naive library skips the check. The second is **algorithm confusion**: a server verifies RS256 tokens with a public key, and the attacker sends an HS256 token signed with that same public key used as an HMAC secret.

Pinning kills both. `algorithms` lists what you accept, the header is checked against it, and nothing else is tried. In the output above, an HS512 token with the correct secret was `InvalidAlgorithm` while only HS256 was allowed, and succeeded the moment the list was widened. So widening the list is a decision, not a convenience. And `none` never gets as far as a decision: `jsonwebtoken` has no such algorithm, so the header itself fails to parse, which is the long `Json(Error(...))` line.

One more detail from the library's source (`crypto/mod.rs`): the signature comparison uses `ring`'s constant-time `verify_slices_are_equal`, so you do not have to worry about comparing it byte by byte yourself.

### Time is an input: the injected clock

`decode` checks `exp` against the real system clock. That is fine in production and impossible to test: a test that needs "30 seconds after `exp`" would have to sleep or fake the machine's time. So this crate makes the clock a parameter, the same idea as the injectable clock of [3.7.2](../02-sessions-vs-jwt/README.md), as a closure instead of a trait. Here is the entire idea, from `src/lib.rs`:

```rust
pub type Clock = Arc<dyn Fn() -> u64 + Send + Sync>;

pub struct JwtConfig {
    pub secret: String,
    pub ttl_secs: u64,
    pub leeway_secs: u64,
    pub clock: Clock,
}
```

`JwtConfig::new` installs the real clock (`system_now`). A test calls `.with_clock(Arc::new(|| 1_700_000_000))` and "now" is that number, forever. Your `verify_token` will turn the library's own `exp` check off and compare `now > exp + leeway_secs` itself, using `config.clock`. The library is still doing the signature, the algorithm pin and the claim parsing. Only the one comparison that needs a clock moved into your code.

`JwtConfig` also has a hand-written `Debug` that prints `<redacted>` for the secret. A derived `Debug` would put the secret into every log line that formats the config, which is the sort of leak nobody notices for a year.

### The quick middleware: `from_fn_with_state`

3.2.4 ended with `from_fn`: an `async fn(Request, Next) -> Response` that `axum` turns into a `Layer`. For auth you need two more things, and both are in the function's arguments. The secret has to come from somewhere, so `from_fn_with_state` passes your state in as a `State(...)` extractor, the same extractor a handler uses. And the function may return a `Result`, so it can refuse. `examples/03-protected-server.rs` is the quick version, with the real clock and a bare `401`. A loose three-line `bearer` helper above it (not shown here) reads the header; the strict version is your first exercise:

```rust
async fn require_auth(
    State(secret): State<&'static str>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let key = DecodingKey::from_secret(secret.as_bytes());
    let token = bearer(&request).ok_or(StatusCode::UNAUTHORIZED)?;
    let data = decode::<Claims>(token, &key, &Validation::new(Algorithm::HS256))
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    request.extensions_mut().insert(AuthUser(data.claims.sub));
    Ok(next.run(request).await)
}
```

Three outcomes hide in it. A missing or malformed header is an early `Err`, and `next` is never called: the middleware **short-circuited**, exactly like `CorsLayer` on a preflight in 3.2.4. A token that fails `decode` is the same. Only a token that passes reaches `next.run(request)`, which is the call to the inner service. Start it, and talk to it with `curl`:

```text
$ curl -i http://127.0.0.1:3190/health
HTTP/1.1 200 OK
content-type: text/plain; charset=utf-8
content-length: 2
date: Sun, 04 Oct 2026 11:17:39 GMT

ok
$ curl -i http://127.0.0.1:3190/whoami
HTTP/1.1 401 Unauthorized
content-length: 0
date: Sun, 04 Oct 2026 11:17:39 GMT

$ curl -s -X POST -H 'content-type: application/json' -d '{"user":"matin"}' http://127.0.0.1:3190/login
{"token":"eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJtYXRpbiIsImlhdCI6MTc5MTExMjY2MCwiZXhwIjoxNzkxMTE2MjYwfQ.Q42TLWeqdNBZG7S4renyFyPJmJPGhAy9H1efRF-T46g"}
$ curl -i -H "Authorization: Bearer $TOKEN" http://127.0.0.1:3190/whoami
HTTP/1.1 200 OK
content-type: application/json
content-length: 19
date: Sun, 04 Oct 2026 11:17:40 GMT

{"user_id":"matin"}
$ curl -i -H "Authorization: Bearer ${TOKEN}x" http://127.0.0.1:3190/whoami
HTTP/1.1 401 Unauthorized
content-length: 0
date: Sun, 04 Oct 2026 11:17:40 GMT

```

The `date:` line and the token change on every run, because the token carries the time it was issued. Appending one character to the token broke the signature, and the request never reached the handler. `POST /login` here hands a token to anyone who asks. It stands in for the real login from 3.7.1, which would check an `argon2` hash first.

```senpai-visual
{"kind":"concept","labels":["request arrives with an Authorization header","bearer_token: pull the token out of the header","verify_token: algorithm, signature, then expiry","valid: insert AuthUser into the request extensions","next.run: the handler reads Extension of AuthUser","invalid at any step: answer 401, handler never runs"]}
```

The Django picture: this is `AuthenticationMiddleware` plus `JWTAuthentication` in one function, and the place where `request.user` gets set. It stops being exact in one way that matters. Django's `request.user` is loaded from the database on every request, so a deleted or deactivated user stops working immediately. This middleware trusts the signed claims until `exp` and never asks a database anything. That is the speed, and it is the revocation problem again.

### Request extensions carry the identity

`request.extensions_mut().insert(AuthUser(...))` puts a value into the request's **extensions**: a small map keyed by Rust type, which is part of the request's `Parts` since [3.2.2](../../02-axum-and-rest-api-design/02-writing-your-own-extractor/README.md). A handler further in asks for it by type with the `Extension` extractor:

```rust
pub async fn whoami(Extension(user): Extension<AuthUser>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "user_id": user.0 }))
}
```

The handler never parses a header and never verifies anything. It trusts that if it runs, the middleware ran first. That trust is a wiring fact, not something the compiler checks, and `examples/04-forgot-the-layer.rs` shows the price. It mounts `whoami` on a router with no auth layer, and sends a request:

```text
status: 500 Internal Server Error
body:   Missing request extension: Extension of type `p3_07_03_jwt_and_tower_middleware::AuthUser` was not found. Perhaps you forgot to add it? See `axum::Extension`.
```

It compiles, it runs, and it answers `500`. Nothing in the types connects "this handler needs `AuthUser`" to "something must insert it". A test that sends a request without a token to every protected route is the only thing that catches this, which is why this lesson's tests do exactly that.

### Where the layer sits: `route_layer` and what comes after it

`app` in `src/lib.rs` is the wiring:

```rust
pub fn app(config: JwtConfig) -> Router {
    Router::new()
        .route("/whoami", get(whoami))
        .route_layer(from_fn_with_state(config, require_auth))
        .route("/health", get(|| async { "ok" }))
}
```

Two rules meet here. The first is from 3.2.4: a layer only wraps the routes added before it, so `/health`, added afterwards, is public: the first `curl` above got `200 ok` without a token. That is deliberate, since a load balancer must be able to ask "are you alive?" without logging in, and it is also the easiest way to expose something by accident. The second is new: it is `route_layer`, not `layer`. `route_layer` only runs on requests that match a route. A request for a path that does not exist gets `404` whether or not it carries a token. With `layer` it would get `401` first, which answers the wrong question: the caller never asked for something that exists.

### Failing well: a typed `AuthError`

The quick version answers every failure with an empty `401`. That is hard to debug and hard to test, so the crate has an enum, `AuthError`, with five cases (`Missing`, `Malformed`, `BadSignature`, `WrongAlgorithm`, `Expired`), and an `IntoResponse` for it, the same move as the `AnimeError` of [3.2.3](../../02-axum-and-rest-api-design/03-anime-catalog-crud-in-memory/README.md). `require_auth` returns `Result<Response, AuthError>`, so an `Err` becomes the response. The finished server (`examples/08-admin-server.rs`, which needs your own code to pass first) answers like this:

```text
$ curl -i http://127.0.0.1:3191/whoami
HTTP/1.1 401 Unauthorized
content-type: application/json
www-authenticate: Bearer
content-length: 25
date: Sun, 04 Oct 2026 11:19:44 GMT

{"error":"missing_token"}
$ curl -i -H "Authorization: Bearer nope" http://127.0.0.1:3191/whoami
HTTP/1.1 401 Unauthorized
content-type: application/json
www-authenticate: Bearer
content-length: 25
date: Sun, 04 Oct 2026 11:19:44 GMT

{"error":"invalid_token"}
```

Two decisions are in there. The `WWW-Authenticate: Bearer` header is what RFC 6750 says a `401` for a bearer token should carry; it names the scheme the client should have used. And the body collapses five internal cases into three codes. `missing_token` tells the client to log in, `token_expired` tells it to refresh or log in again, and `invalid_token` covers everything else: bad signature, wrong algorithm, garbage. An attacker probing the server learns nothing about *why* a forged token failed. The five-way `AuthError` is for your logs and your tests. The three-way code is for the wire. The shape of a whole API's error bodies is [3.8.1](../../08-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.md)'s subject.

The order of the checks inside `verify_token` is part of the design, too. The signature is checked before expiry, so a forged token is `BadSignature` no matter what its `exp` says, and nobody can use the expiry answer to learn which tokens were once real.

### `401` is "who are you", `403` is "you may not": a second middleware

[3.1.3](../../01-networking-and-http-from-scratch/03-http-semantics-you-must-know/README.md) drew the line: `401` means no valid identity, `403` means a known identity without permission. In the crate that is two middlewares, each doing one thing, stacked. `require_auth` establishes who. A second one, which you write in the Build rung, reads `Extension<AuthUser>` and decides whether that subject may go through. `admin_app` puts both in front of `GET /admin`, and only `require_auth` in front of `GET /whoami`.

Which one runs first is the onion from 3.2.4: the layer added last is outermost. So the check for the admin sits on the inside, added first, and `require_auth` is added after it and wraps it:

```senpai-visual
{"kind":"concept","labels":["request comes in","require_auth: outermost, runs first, answers 401","require_subject: inside it, needs AuthUser, answers 403","handler: admin_ping or whoami","a request with no token never reaches the 403 check"]}
```

That ordering is the reason a request with no token gets `401` on `/admin` and not `403`: the second middleware cannot even look up `AuthUser`, because it never runs. If you had them the other way round, `require_subject` would run first, find no `AuthUser` in the extensions and fail with a `500`, the same mistake as `04`. Here is the finished server again, as `matin` (the admin) and as `someone-else`, who has a perfectly valid token:

```text
$ curl -i -H "Authorization: Bearer $MATIN" http://127.0.0.1:3191/admin
HTTP/1.1 200 OK
content-type: text/plain; charset=utf-8
content-length: 4
date: Sun, 04 Oct 2026 11:19:44 GMT

pong
$ curl -i -H "Authorization: Bearer $OTHER" http://127.0.0.1:3191/admin
HTTP/1.1 403 Forbidden
content-length: 0
date: Sun, 04 Oct 2026 11:19:45 GMT

$ curl -i http://127.0.0.1:3191/admin
HTTP/1.1 401 Unauthorized
content-type: application/json
www-authenticate: Bearer
content-length: 25
date: Sun, 04 Oct 2026 11:19:45 GMT

{"error":"missing_token"}
```

Comparing the subject to the string `admin` is a placeholder for a real permission model. A real one asks "may this user do this to that thing", which is [3.7.5 — Modelling RBAC and permissions](../05-modelling-rbac-and-permissions/README.md).

### The JWT mistakes that actually get services broken into

Everything above was one pitfall at a time. Here they are together, with the line of this lesson that answers each:

| Pitfall | What goes wrong | What stops it here |
|---|---|---|
| Trusting the header's `alg` (`none`, RS256/HS256 confusion) | the attacker chooses how the token is checked | `Validation::new(Algorithm::HS256)`; `verify_token` accepts HS256 only |
| No expiry | a stolen token works forever | `exp` is required, and `verify_token` checks it against the clock |
| Secrets or personal data in the claims | anyone holding the token can read them | claims are `sub`, `iat`, `exp` only |
| A short or guessable HMAC secret | an offline attacker brute-forces the secret from any one token, then forges any token | 32+ random bytes from configuration ([3.4.1](../../04-configuration-and-app-structure/01-config-and-secrets/README.md)), never from source |
| Long lifetimes with no way to revoke | a leaked token stays valid until `exp` | short `ttl_secs`, and [3.7.4](../04-refresh-token-rotation-and-revocation/README.md) |
| Keeping the token where script can read it | one XSS bug and every token is stolen from `localStorage` | this lesson only issues and checks tokens; where a browser keeps one is [3.7.2](../02-sessions-vs-jwt/README.md)'s trade-off |
| Telling the client why verification failed | a free oracle for probing | three public error codes, five private ones |

The `localStorage` row deserves a longer note. `localStorage` is readable by any script that runs on the page, including an injected one. A cookie marked `HttpOnly` is not readable by script at all, at the price of other problems that 3.7.2 weighs. There is no storage that is both free of cost and safe from everything, which is exactly why that lesson exists.

---

## Hands on

Run the examples. `03` and `08` are servers: start one in a terminal, use `curl` from another, and stop it with Ctrl+C when you are done. `08` only answers after your ladder passes.

```sh
cargo run -p p3-07-03-jwt-and-tower-middleware --example 01-anatomy-of-a-jwt
cargo run -p p3-07-03-jwt-and-tower-middleware --example 02-validation-knobs
cargo run -p p3-07-03-jwt-and-tower-middleware --example 03-protected-server
cargo run -p p3-07-03-jwt-and-tower-middleware --example 04-forgot-the-layer
cargo run -p p3-07-03-jwt-and-tower-middleware --example 08-admin-server
```

Then the broken ones. They are gated behind a feature, and all three fail to compile:

```sh
cargo build -p p3-07-03-jwt-and-tower-middleware --example 05-extension-not-clone-broken --features broken
cargo build -p p3-07-03-jwt-and-tower-middleware --example 06-wrong-state-type-broken --features broken
cargo build -p p3-07-03-jwt-and-tower-middleware --example 07-error-not-a-response-broken --features broken
```

Then the tests. They fail now, and the first failure shows you a `todo!()` message:

```sh
cargo test -p p3-07-03-jwt-and-tower-middleware --test jwt_test bearer
```

```text
running 2 tests
test bearer_token_accepts_the_scheme_in_any_case ... FAILED
test bearer_token_rejects_everything_else ... FAILED

failures:

---- bearer_token_accepts_the_scheme_in_any_case stdout ----

thread 'bearer_token_accepts_the_scheme_in_any_case' (39828) panicked at phase3-backend-foundations\07-auth-and-security\03-jwt-and-tower-middleware\src\lib.rs:175:5:
not yet implemented: return the token from a well-formed Bearer header value, or None
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- bearer_token_rejects_everything_else stdout ----

thread 'bearer_token_rejects_everything_else' (2620) panicked at phase3-backend-foundations\07-auth-and-security\03-jwt-and-tower-middleware\src\lib.rs:175:5:
not yet implemented: return the token from a well-formed Bearer header value, or None


failures:
    bearer_token_accepts_the_scheme_in_any_case
    bearer_token_rejects_everything_else

test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p p3-07-03-jwt-and-tower-middleware --test jwt_test`
```

The number in parentheses after `thread '...'` is the thread id, and it changes on every run. Now try these:

1. In `02-validation-knobs`, change `strict.leeway = 0` to `strict.leeway = 31`. Which line of the output changes, and why?
2. In `03-protected-server`, change `Validation::new(Algorithm::HS256)` to allow `HS512` too. Which token would now be accepted that was not before?
3. In `04-forgot-the-layer`, wrap the router in the quick `from_fn_with_state` layer from `03`. What does the same request get now?

---

## Errors you will meet

Every transcript below is the real output for the example named, with this lesson's own `todo!()` warnings left out. The long number in a `long-type-...txt` file name is different on every run. The first three are `E0277`, and the second and third are the kind that `axum` is famous for: the message names a type you never wrote and never says which of your arguments is wrong.

### `E0277` — `Extension<T>` needs `T: Clone`

```text
error[E0277]: the trait bound `fn(Extension<AuthUser>) -> impl Future<Output = String> {whoami}: Handler<_, _>` is not satisfied
   --> phase3-backend-foundations\07-auth-and-security\03-jwt-and-tower-middleware\examples\05-extension-not-clone-broken.rs:17:62
    |
 17 |     let _router: Router = Router::new().route("/whoami", get(whoami));
    |                                                          --- ^^^^^^ the trait `Handler<_, _>` is not implemented for fn item `fn(Extension<AuthUser>) -> impl Future<Output = String> {whoami}`
    |                                                          |
    |                                                          required by a bound introduced by this call
    |
    = note: Consider using `#[axum::debug_handler]` to improve the error message
note: required by a bound in `axum::routing::get`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\method_routing.rs:167:16
    |
167 |             H: Handler<T, S>,
    |                ^^^^^^^^^^^^^ required by this bound in `get`
...
441 | top_level_handler_fn!(get, GET);
    | -------------------------------
    | |                     |
    | |                     required by a bound in this function
    | in this macro invocation
    = note: this error originates in the macro `top_level_handler_fn` (in Nightly builds, run with -Z macro-backtrace for more info)

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p3-07-03-jwt-and-tower-middleware` (example "05-extension-not-clone-broken") due to 1 previous error
```

**What the compiler is objecting to:** the error points at `get(whoami)`, not at `AuthUser`, and says only that the function is not a `Handler`. That means one of its arguments is not an extractor, and the compiler does not say which. The `note:` line is the real hint: put `#[axum::debug_handler]` on `whoami` and the same mistake is reported against the argument.

**The fix:** derive `Clone` on the type.

```rust
#[derive(Clone)]
struct AuthUser(String);
```

**Why this is the fix:** the `Extension` extractor does not take the value out of the request, it clones it, because the same extension may be read by several handlers and layers. So the stored type must be `Clone`. `AuthUser` in `src/lib.rs` derives `Debug, Clone`. If a type is expensive to clone, store an `Arc` of it instead.

### `E0277` — the middleware asks for a different state than the layer was given

```text
error[E0277]: the trait bound `FromFn<fn(..., ..., ...) -> ... {check}, ..., ..., _>: Service<...>` is not satisfied
   --> phase3-backend-foundations\07-auth-and-security\03-jwt-and-tower-middleware\examples\06-wrong-state-type-broken.rs:22:22
    |
 22 |         .route_layer(from_fn_with_state(config, check));
    |          ----------- ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ unsatisfied trait bound
    |          |
    |          required by a bound introduced by this call
    |
    = help: the trait `tower_service::Service<axum::http::Request<Body>>` is not implemented for `FromFn<fn(State<String>, ..., ...) -> ... {check}, ..., ..., _>`
    = help: the following other types implement trait `tower_service::Service<Request>`:
              axum::middleware::FromFn<F, S, I, (T1, T2)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6, T7)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6, T7, T8)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6, T7, T8, T9)>
            and 8 others
note: required by a bound in `Router::<S>::route_layer`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\mod.rs:324:21
    |
321 |     pub fn route_layer<L>(self, layer: L) -> Self
    |            ----------- required by a bound in this associated function
...
324 |         L::Service: Service<Request> + Clone + Send + Sync + 'static,
    |                     ^^^^^^^^^^^^^^^^ required by this bound in `Router::<S>::route_layer`
    = note: the full name for the type has been written to 'M:\SenPai-Rust-Journey\target\debug\examples\06_wrong_state_type_broken.long-type-12508816414026072018.txt'
    = note: consider using `--verbose` to print the full type name to the console

error[E0277]: the trait bound `FromFn<fn(..., ..., ...) -> ... {check}, ..., ..., _>: Service<...>` is not satisfied
  --> phase3-backend-foundations\07-auth-and-security\03-jwt-and-tower-middleware\examples\06-wrong-state-type-broken.rs:20:27
   |
20 |       let _router: Router = Router::new()
   |  ___________________________^
21 | |         .route("/", get(|| async { "ok" }))
22 | |         .route_layer(from_fn_with_state(config, check));
   | |_______________________________________________________^ unsatisfied trait bound
   |
   = help: the trait `tower_service::Service<axum::http::Request<Body>>` is not implemented for `FromFn<fn(State<String>, ..., ...) -> ... {check}, ..., ..., _>`
   = help: the following other types implement trait `tower_service::Service<Request>`:
             axum::middleware::FromFn<F, S, I, (T1, T2)>
             axum::middleware::FromFn<F, S, I, (T1, T2, T3)>
             axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4)>
             axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5)>
             axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6)>
             axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6, T7)>
             axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6, T7, T8)>
             axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6, T7, T8, T9)>
           and 8 others
   = note: the full name for the type has been written to 'M:\SenPai-Rust-Journey\target\debug\examples\06_wrong_state_type_broken.long-type-12508816414026072018.txt'
   = note: consider using `--verbose` to print the full type name to the console

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p3-07-03-jwt-and-tower-middleware` (example "06-wrong-state-type-broken") due to 2 previous errors
```

**What the compiler is objecting to:** `from_fn_with_state` built a `FromFn` value, and `route_layer` needs it to be a `tower::Service`. It is not, and the compiler cannot say why in one line, because a `FromFn` is a service only when *every* argument of your function is something `axum` can supply. The only clue is in the `help:` line: the function's first argument is `State<String>`, and the state passed to the layer was a `JwtConfig`. The two errors are the same fact reported twice, once at the call and once at the whole expression.

**The fix:** make the types agree. Either the function takes `State<JwtConfig>`, or the layer is given a `String`:

```rust
async fn check(State(_config): State<JwtConfig>, request: Request, next: Next) -> Response {
    next.run(request).await
}
```

**Why this is the fix:** `from_fn_with_state(state, f)` stores `state` and, on each request, extracts `State<S>` for `f` from it. If `f` asks for a different `S`, there is no way to build its arguments, and the "is a Service" bound fails. When you see this error on a `from_fn` middleware, check the arguments in this order: the state type, the extractors before `Request` (they must implement `FromRequestParts`, and `Request` and `Next` must come last), then the return type, which is the next error.

### `E0277` — the middleware's error type is not an `IntoResponse`

```text
error[E0277]: the trait bound `FromFn<fn(Request<Body>, ...) -> ... {gate}, (), ..., _>: Service<...>` is not satisfied
   --> phase3-backend-foundations\07-auth-and-security\03-jwt-and-tower-middleware\examples\07-error-not-a-response-broken.rs:26:22
    |
 26 |         .route_layer(from_fn(gate));
    |          ----------- ^^^^^^^^^^^^^ unsatisfied trait bound
    |          |
    |          required by a bound introduced by this call
    |
    = help: the trait `tower_service::Service<axum::http::Request<Body>>` is not implemented for `FromFn<fn(Request<Body>, Next) -> ... {gate}, (), ..., _>`
    = help: the following other types implement trait `tower_service::Service<Request>`:
              axum::middleware::FromFn<F, S, I, (T1, T2)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6, T7)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6, T7, T8)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6, T7, T8, T9)>
            and 8 others
note: required by a bound in `Router::<S>::route_layer`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\mod.rs:324:21
    |
321 |     pub fn route_layer<L>(self, layer: L) -> Self
    |            ----------- required by a bound in this associated function
...
324 |         L::Service: Service<Request> + Clone + Send + Sync + 'static,
    |                     ^^^^^^^^^^^^^^^^ required by this bound in `Router::<S>::route_layer`
    = note: the full name for the type has been written to 'M:\SenPai-Rust-Journey\target\debug\examples\07_error_not_a_response_broken.long-type-15268988596771203291.txt'
    = note: consider using `--verbose` to print the full type name to the console

error[E0277]: the trait bound `FromFn<fn(Request<Body>, ...) -> ... {gate}, (), ..., _>: Service<...>` is not satisfied
  --> phase3-backend-foundations\07-auth-and-security\03-jwt-and-tower-middleware\examples\07-error-not-a-response-broken.rs:24:27
   |
24 |       let _router: Router = Router::new()
   |  ___________________________^
25 | |         .route("/", get(|| async { "ok" }))
26 | |         .route_layer(from_fn(gate));
   | |___________________________________^ unsatisfied trait bound
   |
   = help: the trait `tower_service::Service<axum::http::Request<Body>>` is not implemented for `FromFn<fn(Request<Body>, Next) -> ... {gate}, (), ..., _>`
   = help: the following other types implement trait `tower_service::Service<Request>`:
             axum::middleware::FromFn<F, S, I, (T1, T2)>
             axum::middleware::FromFn<F, S, I, (T1, T2, T3)>
             axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4)>
             axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5)>
             axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6)>
             axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6, T7)>
             axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6, T7, T8)>
             axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6, T7, T8, T9)>
           and 8 others
   = note: the full name for the type has been written to 'M:\SenPai-Rust-Journey\target\debug\examples\07_error_not_a_response_broken.long-type-15268988596771203291.txt'
   = note: consider using `--verbose` to print the full type name to the console

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p3-07-03-jwt-and-tower-middleware` (example "07-error-not-a-response-broken") due to 2 previous errors
```

**What the compiler is objecting to:** the same message as the one before, with nothing in it about the cause. This time every argument is fine: the `help:` line shows `fn(Request<Body>, Next)`. What is wrong is the return type, `Result<Response, NotAllowed>`: a `from_fn` function must return something that implements `IntoResponse`, and `Result<T, E>` does only when both `T` and `E` do.

**The fix:** implement `IntoResponse` for the error, or return a type that already has it. This crate's `AuthError` does the first; the quick version in `03` does the second with `StatusCode`:

```rust
impl IntoResponse for NotAllowed {
    fn into_response(self) -> Response {
        StatusCode::UNAUTHORIZED.into_response()
    }
}
```

**Why this is the fix:** a middleware that fails has to *answer*, since there is no outer code to catch an error and `hyper` would close the connection. `axum` turns an `Err` into a response by calling `into_response` on it, which is why 3.2.4 required `Error = Infallible` for hand-written layers. `from_fn` does that conversion for you, but only if the error knows how. This is also why `AuthError` is an enum with an `IntoResponse`: one place decides what every auth failure looks like on the wire.

### Not a compiler error: `500` with "Missing request extension"

The fourth thing you will meet has no error code, because it compiles. A handler that takes `Extension<AuthUser>` on a route with no auth layer in front of it answers `500` and the message you saw in `examples/04-forgot-the-layer.rs`. The fix is never in the handler. Mount the route behind `require_auth`, the way `app` does, and write a test that sends it a request with no token. If it answers `401`, the layer is there. If it answers `500`, it is not.

---

## Exercises

### Warm up

<details>
<summary>A friend says "the token is safe, it is base64, nobody can read that". Is this right?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

No. Base64url is an encoding, not encryption: anyone holding the token can decode the payload with no key. What the signature protects is *integrity*: change one character and verification fails. It does not protect *secrecy*. So the payload must never hold anything private.

</details>

<details>
<summary>A token is issued at <code>1_700_000_000</code> with a lifetime of 3600 seconds and 30 seconds of leeway. Is it valid at <code>1_700_003_630</code>? At <code>1_700_003_631</code>?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

Valid at `1_700_003_630`, expired at `1_700_003_631`. `exp` is `1_700_003_600`, and the rule is "expired when now is greater than `exp + leeway`", so exactly `exp + leeway` is still valid.

</details>

<details>
<summary>A router has <code>.route("/whoami", ...)</code>, then <code>.route_layer(from_fn_with_state(config, require_auth))</code>, then <code>.route("/health", ...)</code>. Does <code>/health</code> need a token?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

No. A layer only wraps the routes added before it, so `/health` is public. That is useful for health checks and dangerous when it happens by accident.

</details>

<details>
<summary>A request to <code>/admin</code> carries a valid token for the user <code>someone-else</code>, and the admin is <code>matin</code>. Is the answer <code>401</code> or <code>403</code>? What if there is no token at all?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

`403 Forbidden` for the valid token: the server knows who you are and you may not. With no token it is `401 Unauthorized`, because `require_auth` is the outer layer and answers before the subject check ever runs.

</details>

<details>
<summary>A verifier is configured with <code>algorithms = [HS256, HS512]</code> "to be flexible". What did it give up?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

It now accepts a second way of being checked, and each extra entry is more room for algorithm-confusion bugs. It should list exactly the algorithms the server itself issues, and for this lesson that is one.

</details>

### Repair

Fix every broken example so it compiles, or answers correctly:

1. `examples/05-extension-not-clone-broken.rs` compiles.
2. `examples/06-wrong-state-type-broken.rs` compiles.
3. `examples/07-error-not-a-response-broken.rs` compiles, and its failure answers `401` rather than being an `Err` nobody handles.
4. `examples/04-forgot-the-layer.rs` answers `401` instead of `500`. This one needs the Implement rung first, because the fix is to put the route behind your `require_auth`.

### Implement

Four functions in `src/lib.rs`, in this order. Everything above the line "the ladder" is given: `Claims`, `Clock`, `JwtConfig`, `AuthError` with its `IntoResponse`, `AuthUser`, `whoami` and `app`.

1. `bearer_token`: pull the token out of an `Authorization` header value.
2. `issue_token`: sign HS256 claims for a user with the config's clock.
3. `verify_token`: the full verification, with the outcomes in the order the doc comment lists.
4. `require_auth`: the middleware, which ties the other three together.

```sh
cargo test -p p3-07-03-jwt-and-tower-middleware --test jwt_test
```

The doc comment above each function is the whole specification, including the exact order of `verify_token`'s checks. You never need to read the tests. They build tokens by hand where they have to (a forged payload, a token with no `exp`, an `alg: none` header), and every one of them uses a fixed clock, so none depends on the time you run it. The tests send requests with `oneshot`, as in 3.2.1. You will need to add the `jsonwebtoken` imports yourself. When it all passes, `04` and `08` work.

### Build

`admin_app`, in the same file: a router with `GET /whoami` for any valid token and `GET /admin` only for the subject named by the `admin` argument. It needs a second `from_fn_with_state` middleware that you write yourself, one that reads the `AuthUser` your `require_auth` stored. The doc comment states the status codes, including which one wins when both checks would fail.

```sh
cargo test -p p3-07-03-jwt-and-tower-middleware --test jwt_test admin
```

Think about the order in which you add the two layers before you write them, using the onion from 3.2.4. The wrong order compiles and fails with the `500` from `04`.

### Challenge (optional)

Key rotation. A secret that cannot change is a secret that leaks forever, so real systems sign with a key id (`kid` in the header) and keep the old keys for verification only. Give `JwtConfig` a list of retired secrets and make `verify_token` try the one named by the token's `kid`. Nothing tests this one. Write your own tests with a fixed clock, and decide what a token with an unknown `kid` should be, and why it must not fall back to trying every key. This reaches forward to how secrets are loaded in [3.4.1](../../04-configuration-and-app-structure/01-config-and-secrets/README.md).

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| JWT | a signed token: base64url header, payload and signature joined by dots | stateless login for an API |
| claims | the fields of the payload: `sub`, `iat`, `exp` and your own | deciding who a request is from |
| signed, not encrypted | the payload is readable by anyone, but tampering is detectable | what you may put in a token |
| HS256 | HMAC-SHA256 with one shared secret | the signing algorithm of this lesson |
| algorithm pinning | accepting only the algorithm you issue, never the one the header names | stopping `alg: none` and algorithm confusion |
| leeway | seconds of grace past `exp` for clock skew | `Validation::leeway` and `JwtConfig::leeway_secs` |
| injected clock | "now" passed in as a function, not read from the system | deterministic tests for anything time-based |
| bearer token | a credential sent as `Authorization: Bearer <token>` | every protected request |
| `from_fn_with_state` | the `from_fn` shortcut with state handed to the middleware | auth, anything that needs configuration |
| request extensions | a type-keyed map on the request; `Extension<T>` reads it | passing the verified identity to handlers |
| `401` / `403` | no valid identity / a known identity without permission | authentication vs authorization |

### What you now know

- A JWT is `header.payload.signature`, each part base64url, and only the signature needs the secret. Anyone can read the payload.
- `jsonwebtoken` 9 verifies with a `Validation`: `exp` required and checked, `leeway` 60 seconds by default, and an `algorithms` list that starts as the single algorithm you named.
- You pin the algorithm so the client never chooses how it is checked, and `alg: none` cannot parse at all.
- Time is an input: inject a clock, turn off the library's own `exp` check, and compare yourself, so every expiry case is a plain unit test.
- `from_fn_with_state` makes auth middleware a function: refuse with an `Err` that implements `IntoResponse`, or insert an `AuthUser` into the request's extensions and call `next`.
- A layer wraps only the routes added before it, and the last layer added is the outermost, which is why `401` comes before `403`.
- Five internal error cases can be one public code each, and the signature is checked before expiry so a forged token learns nothing.

### What comes back later

- **Why a stolen JWT cannot simply be revoked, short-lived access tokens, and refresh-token rotation** — [3.7.4 — Refresh-token rotation and revocation](../04-refresh-token-rotation-and-revocation/README.md)
- **A real permission model instead of `subject == "admin"`** — [3.7.5 — Modelling RBAC and permissions](../05-modelling-rbac-and-permissions/README.md)
- **Where a browser should keep a token, and the sessions-versus-JWT trade-off** — [3.7.2 — Sessions vs. JWT: the real trade-off](../02-sessions-vs-jwt/README.md)
- **Loading the signing secret from configuration instead of a constant** — [3.4.1 — 12-factor config and secrets](../../04-configuration-and-app-structure/01-config-and-secrets/README.md)
- **One error-body shape for the whole API, instead of three codes for auth only** — [3.8.1 — Consistent error envelopes](../../08-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.md)
- **Letting a browser on another origin send the `Authorization` header** — [3.2.5 — CORS and frontend integration](../../02-axum-and-rest-api-design/05-cors-and-frontend-integration/README.md)

### Can you explain?

- What are the three parts of a JWT, and which of them needs the secret to read?
- Why is "signed, not encrypted" the first thing to know, and what must never go in the claims?
- What does `Validation::new(Algorithm::HS256)` check by default, and what is surprising about its `leeway`?
- What is `alg: none`, what is algorithm confusion, and what one habit prevents both?
- Why does this crate take a clock as a parameter, and what exactly moved from the library into `verify_token` because of it?
- How does an identity get from the middleware to the handler, and what happens if nothing inserted it?
- Why does `/admin` answer `401` with no token and `403` with the wrong user, and what does the order of the two layers have to do with it?
- Why does the client see three error codes while your code has five error cases?

---

## Going further

- [RFC 7519: JSON Web Token](https://www.rfc-editor.org/rfc/rfc7519): the claims, including the ones this lesson skipped (`iss`, `aud`, `nbf`, `jti`).
- [RFC 8725: JSON Web Token Best Current Practices](https://www.rfc-editor.org/rfc/rfc8725): the official list of the pitfalls in the table above, with the attacks named.
- [RFC 6750: Bearer Token Usage](https://www.rfc-editor.org/rfc/rfc6750): why the `401` carries `WWW-Authenticate: Bearer`.
- [`jsonwebtoken` 9.3.1 on docs.rs](https://docs.rs/jsonwebtoken/9.3.1/jsonwebtoken/): `Validation`, `Algorithm` and the `ErrorKind` variants.
- [`axum::middleware`](https://docs.rs/axum/0.8.9/axum/middleware/index.html): `from_fn`, `from_fn_with_state` and the rules about the order of layers.
- [OWASP JWT Cheat Sheet for Java](https://cheatsheetseries.owasp.org/cheatsheets/JSON_Web_Token_for_Java_Cheat_Sheet.html): written for Java, but its list of token attacks and where to store a token is language-independent.

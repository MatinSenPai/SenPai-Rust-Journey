# 06.2 — JWTs and `tower` middleware

Module 1 hashed passwords so a login endpoint can check "is this really
you." This lesson answers the next question: once a user has proven who
they are once, how does every *later* request prove it again, without
sending the password every time and without the server having to remember
every logged-in user in memory? The answer this lesson builds is a **JWT**
(JSON Web Token) issued at login and verified by an `axum` middleware on
every protected request after that.

## What a JWT actually is

A JWT is three base64url-encoded segments joined by dots:

```
eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ1c2VyLTQyIiwiZXhwIjoxNzUyMDgwMDAwfQ.dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk
└──────── header ────────┘└─────────────── payload ───────────────┘└──────────── signature ────────────┘
```

- **Header** — algorithm + token type, e.g. `{"alg":"HS256","typ":"JWT"}`.
- **Payload** — the **claims**: `{"sub":"user-42","exp":1752080000}` in this
  lesson. `sub` ("subject") is the standard claim name for "who is this
  token about"; `exp` ("expiration") is a Unix timestamp.
- **Signature** — `HMAC-SHA256(header + "." + payload, secret)` for the
  `HS256` algorithm this lesson uses (the `jsonwebtoken` crate's default).

**Critical fact, worth repeating until it's automatic: a JWT is signed, not
encrypted.** Base64url is an *encoding*, not encryption — it's reversible by
anyone, no key required. Paste any JWT into <https://jwt.io> and its header
and payload appear instantly in plain text; only the *signature* needs the
secret. The signature proves the payload came from your server and hasn't
been tampered with since — it does **not** hide the payload from whoever
holds the token. **Never put a password, a secret, or anything you wouldn't
put in a URL query string into a JWT's claims.** A user id, a role, an
expiration — fine. A password or an API key — never.

## Why a JWT instead of a server-side session

A traditional session (Django's default `sessionid` cookie, for example)
stores session state *on the server* — a database row or a cache entry the
server looks up on every request to answer "who is this." A JWT flips that:
all the state (who you are, when this expires) travels *inside the token
itself*, cryptographically signed so the server can trust it without
looking anything up. The trade-off that matters most: a session can be
revoked instantly (delete the row); a JWT is valid until it expires,
full stop, unless you build a separate revocation mechanism (a
denylist store, which reintroduces the "look something up on every
request" cost JWTs were meant to avoid). That's exactly why `exp` matters so
much, and why real systems keep JWT lifetimes short (this lesson uses one
hour) and pair them with a separate, longer-lived refresh-token flow rather
than issuing something that's valid — and unrevokable — for a month.

## Issuing a token, verifying it in middleware

Two halves, matching this crate's two `todo!()`s:

```rust
pub fn issue_token(user_id: &str, secret: &str) -> String {
    // build Claims { sub: user_id, exp: one hour from now }
    // encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
}

pub async fn require_auth(
    State(secret): State<String>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // pull "Authorization: Bearer <token>" off the request
    // decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &Validation::default())
    // Ok => stash the user id in request.extensions_mut(), then next.run(request).await
    // Err => StatusCode::UNAUTHORIZED
}
```

`require_auth` is `axum::middleware::from_fn_with_state`-compatible — the
same shape as
[`capstone-taskforge/taskforge-api/src/auth.rs`](../../../capstone-taskforge/taskforge-api/src/auth.rs)'s
`require_bearer_token`. Read that file side by side with this lesson's
`require_auth`: both extract the `Authorization` header, both return `401`
on failure, both call `next.run(request).await` to let an authorized
request continue. The difference is entirely in *what counts as
authorized*: `require_bearer_token` does a plain string comparison against
one static shared secret (good enough for a small trusted set of internal
callers, as its own doc comment says), while `require_auth` here decodes
and cryptographically verifies a signed, per-user, *expiring* JWT — the
more realistic shape for a public-facing, multi-user API where different
callers need different identities and tokens need to eventually stop
working on their own.

## `axum::middleware::from_fn_with_state` and request extensions

```rust
pub fn app(secret: String) -> Router {
    Router::new()
        .route("/whoami", get(whoami))
        .route_layer(from_fn_with_state(secret, require_auth))
}
```

`route_layer` wraps every route registered *before* it in the chain with a
`tower` middleware — here, `require_auth`. `from_fn_with_state(secret,
require_auth)` captures `secret` once and hands a clone to `require_auth`
via its `State<String>` extractor on every call, independent of anything
else in the router. A request that fails `require_auth` never reaches
`whoami` at all — the middleware returns `Err(StatusCode::UNAUTHORIZED)`
and the handler is simply never called, the same short-circuiting shape
Django's `LoginRequiredMixin`/`permission_classes` gives you, just
expressed as an explicit function in the request pipeline instead of a
decorator.

On success, `require_auth` inserts an `AuthUser` into
`request.extensions_mut()` — a typed slot on the request that any handler
downstream can pull out with the `Extension<AuthUser>` extractor:

```rust
pub async fn whoami(Extension(user): Extension<AuthUser>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "user_id": user.0 }))
}
```

This is how a JWT's claims become available to your actual business logic
without every handler re-parsing and re-verifying the header itself — the
middleware does that work exactly once, and hands the *result* forward.

## Your task

Open `src/lib.rs`. Two `todo!()`s:

- `issue_token` — build `Claims` with a one-hour expiration, sign it with
  `jsonwebtoken::encode`.
- `require_auth` — extract the bearer token, verify it with
  `jsonwebtoken::decode`, insert `AuthUser` into the request's extensions
  on success, return `401` on any failure (missing header, malformed token,
  wrong secret, or expired — `decode` checks `exp` for you automatically).

Both `todo!()`s note exactly which `use` statements to add — they're left
out of the starter's imports on purpose, so you don't get `unused_imports`
warnings for code you haven't written yet.

Tests live in `tests/jwt_test.rs` (not inline) — this is a "mini-project"
lesson (a router, a middleware, and a handler working together), matching
`docs/conventions.md`'s rule that this shape of lesson tests only the
crate's public surface, the same way `taskforge-api` and the anime catalog
lesson do.

## Checkpoint

`cargo test -p p3-06-02-jwt-and-tower-middleware`, then `CHECKPOINT.md`,
then `solution/SOLUTION.md`.

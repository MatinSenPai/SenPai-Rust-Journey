# Solution

```rust
pub fn issue_token(user_id: &str, secret: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after the Unix epoch")
        .as_secs();
    let claims = Claims {
        sub: user_id.to_string(),
        exp: (now + ONE_HOUR_IN_SECONDS) as usize,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .expect("encoding a valid Claims struct should never fail")
}
```

`Header::default()` picks `HS256` (HMAC-SHA256), the symmetric algorithm
`jsonwebtoken` defaults to — the same secret both signs and later verifies
the token, which is why `issue_token` and `require_auth` both take a
`secret: &str`/`State<String>` for the *same* value. `EncodingKey::from_secret`
just wraps that secret's bytes; `encode` serializes `Claims` to JSON,
base64url-encodes the header and payload, computes the HMAC signature over
`header.payload`, and joins all three with dots — literally the three
segments described in `README.md`.

```rust
pub async fn require_auth(
    State(secret): State<String>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let data = decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &Validation::default())
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    request.extensions_mut().insert(AuthUser(data.claims.sub));

    Ok(next.run(request).await)
}
```

The `and_then` chain is doing the same job `taskforge-api::auth::
require_bearer_token`'s `match` does, just written point-free: "get the
header, or bail" → "turn it into `&str`, or bail" → "strip the `Bearer `
prefix, or bail" → `.ok_or(StatusCode::UNAUTHORIZED)?` turns whatever
`None` fell out of that chain into an early return, same as
`require_bearer_token`'s `_ => Err(ApiError { .. })` catch-all arm.
`decode::<Claims>` is where the actual cryptographic work happens:
recompute the HMAC over the token's header+payload using `secret`, compare
it to the signature the token carries, and — because `Validation::default()`
includes `exp` in its `required_spec_claims` — reject the token outright if
`exp` is in the past, no separate time check needed on our end.
`.map_err(|_| StatusCode::UNAUTHORIZED)` deliberately throws away *why*
`decode` failed (bad signature vs. malformed token vs. expired are all
distinct `jsonwebtoken::errors::ErrorKind` variants) — from the caller's
side, all of them mean the same thing: not authorized, full stop.

## Why this is `require_bearer_token`, upgraded

Read `taskforge-api::auth::require_bearer_token` next to `require_auth`:
both take a captured piece of state via `State<_>`, both extract the same
`Authorization` header, both return early with an unauthorized response,
both call `next.run(request).await` on success. The entire difference is in
*what "authorized" means*:

```rust
// taskforge-api: is this literally the one shared secret string?
Some(value) if value == format!("Bearer {}", state.auth_token) => Ok(next.run(request).await),
```

vs.

```rust
// this lesson: is this a validly-signed, not-yet-expired token for *some* user?
let data = decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &Validation::default())?;
```

`require_bearer_token`'s own doc comment says exactly when its simpler
approach is enough: "good enough for an admin/internal API... nowhere near
enough for a multi-tenant public one (no per-user identity, no scopes, no
expiry)." `require_auth` buys back all three of those: `sub` gives you
per-user identity (`AuthUser`, inserted into request extensions, is
something `require_bearer_token`'s world has no equivalent for — there's
only "the one shared token," not "which caller"), the JWT format leaves
room to add scopes/roles as additional claims later, and `exp` gives every
token a hard, built-in expiry the static-string approach has no way to
express at all.

## On the recall questions

**Q1 (signed vs. encrypted, concretely):** Pasting a JWT into jwt.io with
no secret shows you the full header and payload immediately, decoded and
readable — `alg`, `typ`, `sub`, `exp`, all of it, in plain JSON. What you
*can't* do without the secret is get jwt.io to tell you the signature is
valid — that check needs the same key the server used to sign it. That's
"signed, not encrypted" in one sentence: the payload was never hidden, only
the "this came from the real server and hasn't changed" property required a
secret.

**Q2 (why one status code for four failure modes):** From the client's
point of view, "your token is missing," "your token is garbage," "your
token was signed by the wrong party," and "your token expired" are all the
same instruction: reauthenticate. Splitting them into different status
codes would leak information that's more useful to an attacker probing
your auth than to a legitimate client — e.g., a `400` for "malformed" vs.
`401` for everything else tells an attacker whether their forged token at
least parsed correctly, information they shouldn't get for free. One `401`
for the whole family keeps the failure surface uniform, the same reasoning
`verify_password` (lesson 06.1) uses for collapsing "wrong password" and
"malformed hash" into one `false`.

**Q3 (what a token with no `exp` would allow):** Without `exp`,
`require_auth`'s only check would be "is the signature valid" — a token
issued once would remain valid *forever*, with no way to force it to stop
working short of rotating the signing secret (which invalidates every
outstanding token for every user simultaneously, not just one). Any token
that ever leaked — logged accidentally, intercepted, stolen from a
compromised client — would grant permanent access. `exp` bounds the blast
radius of a leaked token to however long its remaining lifetime is, which
is exactly why real systems keep that window short.

**Q4 (what's available downstream):** After `require_bearer_token`
succeeds, a handler knows only "the caller had the right shared secret" —
there's no per-caller identity at all, every authorized caller looks
identical. After `require_auth` succeeds, a handler can pull `Extension<
AuthUser>` and know specifically *which* user made this request (`sub`),
without re-parsing or re-verifying anything itself — `whoami` is a direct
demonstration of this: it does zero auth logic of its own, it just reads
what the middleware already decided.

**Q5 (why test through `app().oneshot(...)` and not `require_auth` bare):**
`require_auth` needs a real `axum::extract::Request` (to read headers from
and mutate extensions on) and a real `Next` (an opaque handle to "the rest
of the middleware chain plus the handler") to run at all — `Next` isn't
something you can trivially construct by hand outside of a real router.
Driving everything through `app().oneshot(request)` exercises `require_auth`
exactly as it will actually run in production: wired into a real `Router`,
wrapping a real handler, deciding in each test whether that handler gets
called at all. It's also just more informative — these tests prove not
only "the middleware makes the right decision" but "a rejected request
never reaches `whoami`," which testing `require_auth` in isolation
couldn't show as directly.

**Q6 (what "something extra" for revocation looks like):** The standard
sketch is a **denylist**: when you need to revoke a token early (logout,
compromised account, password change), record its unique id (a `jti` claim
you'd add to `Claims`) or its user id in a fast lookup store (Redis, a
database table with a TTL matching the token's remaining lifetime), and
have `require_auth` check that store on every request *in addition to*
verifying the signature. That check is precisely the "look something up on
every request" cost a pure JWT was designed to avoid — you've reintroduced
a server-side, stateful check for the one case (early revocation) a
stateless token structurally can't express on its own. In practice this
trade-off is usually accepted only for the revocation case specifically
(a small, fast lookup) rather than abandoning JWTs' stateless-verification
benefit entirely for every request.

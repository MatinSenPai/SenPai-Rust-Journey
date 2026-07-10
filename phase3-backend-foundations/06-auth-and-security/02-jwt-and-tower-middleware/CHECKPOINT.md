# Checkpoint

1. Paste a JWT (any one — mint one with `issue_token` and print it, or use
   the example in `README.md`) into <https://jwt.io> without providing a
   secret. What can you read? What can you *not* verify without the
   secret? Use that to explain, in your own words, exactly what "signed,
   not encrypted" means in practice.
2. `require_auth` returns `Err(StatusCode::UNAUTHORIZED)` for four
   completely different underlying problems: a missing header, a malformed
   token, a token signed with the wrong secret, and an expired token. Why
   is collapsing all four into one `401` the right call for the client-
   facing response, rather than, say, a `400` for "malformed" and a `401`
   only for "expired"?
3. `jsonwebtoken::decode` checks `exp` against the current time
   automatically as part of `Validation::default()`. What would go wrong —
   concretely, what could an attacker do — if `Claims` had no `exp` field
   at all, and `require_auth` only checked the signature?
4. Compare `require_auth` to `taskforge-api::auth::require_bearer_token`
   side by side. Both extract the same header and return the same kind of
   error. What's the one architectural difference in *what information is
   available downstream* after each middleware succeeds — what can a
   handler behind `require_auth` know about the caller that a handler
   behind `require_bearer_token` cannot?
5. This lesson's tests live in `tests/jwt_test.rs`, calling only `app`,
   `issue_token`, and `Claims` — never `require_auth` directly. Why does
   testing through `app().oneshot(...)` (the full router + middleware +
   handler stack) rather than calling `require_auth` as a bare function
   actually matter here, given that `require_auth` needs a real `Request`
   and a real `Next` to run at all?
6. README.md argues a JWT can't be revoked the way a server-side session
   can — it's valid until `exp`, full stop, unless you build something
   extra. In your own words, what would "something extra" look like (a
   sketch, not code), and why does adding it partially defeat the
   "server doesn't have to look anything up" appeal of JWTs in the first
   place?

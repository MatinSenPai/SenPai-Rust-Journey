# 05.3 — Auth strategies compared

No code in this lesson. You've already built one full strategy end to end —
`phase3-backend-foundations/06-auth-and-security/02-jwt-and-tower-middleware`'s
stateless JWT auth — and hit a real limitation along the way that this
lesson names precisely and gives you the standard fixes for. This lesson
also places that strategy alongside the others you'll be asked to choose
between in a real system: sessions, SSO, and OAuth2.

## Session-based auth

The traditional shape, and the one Django uses by default: on login, the
server creates a **session** — a record of "this user is logged in,"
stored server-side (a database row, or more commonly a fast in-memory store
like Redis). The server hands the client a cookie containing nothing but a
**session id** — an opaque reference, not the session data itself. On every
subsequent request, the server takes that id, looks up the session, and
learns who the request is from.

- **Stateful** — the server has to store something, and look it up on every
  request.
- **Easy to revoke** — logging a user out, or an admin forcibly ending a
  session, is a single row delete. The very next request with that session
  id finds nothing and is rejected. Instant, no waiting for anything to
  "expire."
- **The scaling cost**: a session store that lives in one server's memory
  works fine with one server. The instant you run more than one replica
  (`phase4-backend-advanced/06-system-design-fundamentals/01-cap-scaling-lb-idempotency-locking`'s
  horizontal-scaling section covers exactly this shape of problem), a
  session created by instance A is invisible to instance B — a user's next
  request, load-balanced onto a different instance, would look logged-out.
  The fix is a **shared** session store every replica can reach (Redis is
  the standard choice), which reintroduces a network hop and a shared piece
  of state that every instance now depends on — exactly the kind of
  dependency that stateless design tries to avoid in the first place.

## Token-based auth (JWT)

The shape `phase3-backend-foundations/06-auth-and-security/02-jwt-and-tower-middleware`
already built: the server signs a **self-contained** token at login — the
user's identity and an expiration are encoded directly *inside* the token
(see the previous lesson in this module for why "encoded," not "encrypted,"
is the precise word). Every later request carries the token, and `require_auth`
verifies the signature and checks `exp` — no lookup, no shared store, no
per-request round trip to anywhere.

- **Stateless** — the server holds nothing. Any replica can verify any token
  using only the shared signing secret, which is exactly why
  `phase4-backend-advanced/06-system-design-fundamentals/01-cap-scaling-lb-idempotency-locking`
  singles out statelessness as the property that makes horizontal scaling
  trivial: a JWT-authenticated API scales the same way `taskforge-api` does,
  zero shared state, zero coordination between replicas.
- **The tradeoff, named honestly**: hard to revoke before expiry. This isn't
  a hypothetical gap — it's a specific, concrete limitation of the exact
  code you already wrote. Walk through what `require_auth` in
  `phase3-backend-foundations/06-auth-and-security/02-jwt-and-tower-middleware`
  actually checks: it verifies the signature (does this token's signature
  match what `HMAC-SHA256(header + "." + payload, secret)` produces?) and it
  checks `exp` (has this timestamp passed?). That's it. That's the entire
  set of checks. There is no step anywhere in that function that asks "has
  this *specific* token been invalidated?" — there's nowhere for that
  question to even be answered, because the whole design point of a JWT is
  that the server doesn't store anything about which tokens it's issued. If
  a user's account is compromised, or they hit "log out," or an admin needs
  to force a user off the system *right now*, there is genuinely no way to
  invalidate that one token before its `exp` arrives — not without changing
  the code. The one blunt instrument that *would* work — rotating the
  signing secret — invalidates **every** token for **every** user
  simultaneously, not just the one you wanted to revoke, which is rarely an
  acceptable trade for a single compromised account. This is a real,
  honest limitation of the lesson's implementation as built, not a solved
  problem glossed over.

  The standard fixes, in order of how often each is actually used:
  - **Short-lived access tokens + a longer-lived refresh token.** The JWT
    itself (the "access token") gets a short lifetime — minutes, not the
    one hour `phase3-backend-foundations/06-auth-and-security/02-jwt-and-tower-middleware`
    uses for teaching simplicity, sometimes even shorter in production. A
    separate **refresh token**, opaque and long-lived, is stored server-side
    (in a database, exactly like a session) and used only to mint new access
    tokens. Revocation now means deleting the refresh token row — the
    already-issued access token is still technically valid for its
    remaining few minutes, but that's a much smaller exposure window than
    "valid for up to an hour," and no new access tokens can be minted after
    revocation. This is the industry-standard answer, and it's a hybrid: the
    access token stays fully stateless for the common case (most requests),
    while the *rare* case (revocation, refresh) pays the stateful lookup
    cost that sessions pay on *every* request.
  - **A token-blocklist** for the rare "log out everywhere" or "this token
    is definitely compromised" case: a small, shared store (Redis again) of
    token ids that should be rejected even though their signature and `exp`
    are still valid. `require_auth` would need one more check — "is this
    token's id in the blocklist?" — which reintroduces exactly the
    per-request lookup JWTs were meant to avoid, which is precisely why this
    is reserved for the rare case rather than the default path: you're
    trading away statelessness's whole benefit, on purpose, for the specific
    requests that need it.

## Sessions vs. JWTs, side by side

| | Session-based | JWT / token-based |
|---|---|---|
| State | Server-side (DB/Redis) | Self-contained in the token |
| Scaling | Needs a *shared* store across replicas | Stateless, scales with zero shared state |
| Revocation | Instant (delete the row) | Not possible before `exp`, without extra machinery |
| Per-request cost | A lookup, every request | Just signature verification, no lookup |

Neither one is "correct" in the abstract, the same way CAP's C-vs-A tradeoff
in `phase4-backend-advanced/06-system-design-fundamentals/01-cap-scaling-lb-idempotency-locking`
isn't correct in the abstract — it's a tradeoff you make deliberately for
your system's actual requirements. A system where instant revocation matters
a lot (banking, anything security-sensitive) leans toward sessions, or
toward the refresh-token hybrid above. A system where horizontal scale and
minimizing shared state matter more than instant revocation leans toward
pure JWTs.

## SSO (Single Sign-On)

SSO solves a different problem than either of the above: **one login,
trusted across multiple separate applications.** Log into your company's
identity provider once, and every internal tool — the wiki, the CI
dashboard, the expense-report system — trusts that one login without asking
you to authenticate again at each one. SSO isn't a competing mechanism to
sessions or JWTs — it's typically **built on top of** one of them (often
JWTs, sometimes SAML, an older XML-based standard for the same idea): the
identity provider issues a token after your one login, and each downstream
application verifies that token the same way `require_auth` verifies a JWT,
just trusting a shared identity provider's signature instead of minting its
own tokens independently.

## OAuth2: authorization, not authentication

This is the single most commonly confused piece of vocabulary in this whole
space, worth stating as bluntly as possible: **OAuth2 is an authorization
framework, not an authentication mechanism.** Authorization answers "what is
this party allowed to do" — OAuth2's actual job is letting a user grant a
*third-party application* limited access to *their data on some other
service*, without ever handing that third-party app their actual
credentials for that service. "Let this photo-printing app access my Google
Photos library, without giving it my Google password" is the canonical
OAuth2 use case — note that nowhere in that sentence does the photo-printing
app learn *who you are*, it just gets a scoped, revocable grant to touch
your photos.

The confusion comes entirely from **"Sign in with Google" buttons**, which
genuinely do use OAuth2 machinery under the hood — but authentication isn't
what OAuth2 itself provides there. **OpenID Connect (OIDC)** is a thin
authentication layer built *on top of* OAuth2 specifically to close that
gap: it adds a standardized "here's who this user is" token (itself a JWT,
called an **ID token**) to the OAuth2 flow, which is what actually lets the
"Sign in with Google" button tell your app "this is
`senpaimatin@gmail.com`," as opposed to just "this app may now access
whatever scope of Google data was granted." Plain OAuth2, without OIDC on
top, only ever answers "what can this app touch" — never "who is this."
Treating OAuth2 alone as an authentication system is the mistake to name
explicitly: it can *look* like login because the user experience is
identical (a redirect to Google, a consent screen, a redirect back), but the
thing actually flowing back to your app in raw OAuth2 is an **access token**
scoped to some data, not a verified identity claim.

## Next

No `cargo test` for this lesson — it is a reading lesson.

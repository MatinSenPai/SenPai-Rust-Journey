# 3.7.2 — Sessions vs. JWT: the real trade-off

## At a glance

After this lesson you can:

- Say which problem each approach solves (revocation, scaling, size, who can steal the credential) and why "JWT for everything" is usually the wrong default for a web app with one backend.
- Build the server-side half in plain Rust: a `SessionStore` with an injectable clock, and a pure `Set-Cookie` builder that gets `HttpOnly`, `Secure`, `SameSite` and `Max-Age` right.
- Defend a cookie-based API against session fixation and CSRF with a fresh id at login and an `Origin` check, and prove it with `oneshot` tests.
- Read `E0004` and `E0277` in the shapes this code produces them, and fix them.

**Time:** ~110 minutes · **Prerequisites:**
[3.7.1 — Password hashing with `argon2`](../01-password-hashing-argon2/README.md),
[3.2.3 — Anime catalog CRUD (in-memory)](../../02-axum-and-rest-api-design/03-anime-catalog-crud-in-memory/README.md),
[3.2.5 — CORS and frontend integration](../../02-axum-and-rest-api-design/05-cors-and-frontend-integration/README.md),
[3.4.2 — Application state and dependency wiring](../../04-configuration-and-app-structure/02-app-state-and-dependency-wiring/README.md),
[3.1.3 — HTTP semantics you must know](../../01-networking-and-http-from-scratch/03-http-semantics-you-must-know/README.md)

---

## Why this matters

3.7.1 answered "is this password right?". The next question is what the server does *after* it said yes. HTTP is stateless: the next request arrives with no memory of the login. Something has to ride along with every request to say "this is the same person". There are exactly two designs for that something:

- **Stateful:** the client carries a meaningless random id, and the server keeps a table `id -> who`. Nothing the client holds says anything by itself.
- **Stateless:** the client carries a self-contained, signed statement ("user 7, until 14:30"), and the server keeps no table at all. It only checks the signature.

Django gave you the first one without asking: `django.contrib.sessions` puts a `sessionid` cookie in the browser and a row in a table, and `login()` / `logout()` manage both. A lot of tutorials then teach the second as if it were the modern replacement. It is not a replacement. It is a trade, and for the common case (one web app, one backend, your own frontend) the trade is bad: you give up the ability to end a login, and you get a scaling benefit you do not need.

This lesson makes the trade concrete. You build the stateful side for real, and you demonstrate the stateless side's central weakness with fake tokens. 3.7.3 builds real JWTs, and 3.7.4 deals with the revocation problem this lesson shows. Knowing *why* those two lessons are shaped as they are is the point.

---

## The concept

### Two ways to remember who is logged in

```senpai-visual
{"kind":"network","labels":["browser: POST /login","server: create session, store id -> user","Set-Cookie: sid=random id","browser: GET /me with the cookie","server: look the id up in the store","user, or 401 if the id is gone"]}
```

In the stateful design the cookie is a *reference*; the facts live on the server. In the stateless design the token is the facts, plus a signature that proves the server wrote them. Everything else in this lesson follows from that one difference.

| | Stateful session | Stateless signed token |
|---|---|---|
| What the client holds | an opaque random id (here 32 hex characters) | a signed claim set, often 200 to 1000 bytes or more |
| What the server keeps | a table of live sessions | nothing (only the signing key) |
| Check on each request | a lookup | a signature check and an expiry check |
| End a login early | delete one row | impossible without adding state back |
| Change who someone is (role, ban) | next request sees it | old tokens keep the old claims until they expire |
| Extra servers | must share the table | share only the key |

Where the Django analogy stops: Django's default session backend keeps the table in your database, which is a stateful session. Its `signed_cookies` backend is the stateless design wearing the same API, and its documentation lists the drawbacks that come with it. One framework, both designs, and the docs tell you which one to use by default.

### A session is an id in a cookie and a row on the server

`SessionStore` is plain Rust, like 3.2.3's store: no `axum`, no HTTP. `create(user)` mints a 128-bit random id (16 bytes from the operating system's random source, as 32 hex characters), records the user and an expiry, and returns the id. `lookup(id)` answers who, or nothing. `revoke(id)` deletes the row.

The expiry needs "now", and a test that sleeps for an hour is not a test. So the store reads time through a trait you inject, the same `Clock` shape as in [3.4.2](../../04-configuration-and-app-structure/02-app-state-and-dependency-wiring/README.md):

```rust
pub trait Clock: Send + Sync {
    fn now(&self) -> u64;
}
// production: SystemClock. tests: ManualClock, moved by hand.
```

The `Send + Sync` bound is not decoration: handlers run on many threads, and the clock is shared between them. "Errors you will meet" shows what happens when a clock is built from a `Cell`.

A session is live while `now < expires_at`, so it dies *at* the expiry second, not after it. Nothing sweeps expired rows in this lesson; `lookup` deletes the one it just found expired. A real store adds a periodic purge, and the table grows until it has one.

### Revocation is the whole argument

`examples/01-token-revocation-needs-state.rs` builds a stateless token from scratch. The tokens are fake (`user.expires.check`, with a toy check value standing in for a signature; 3.7.3 uses real signatures). It verifies one, "logs out", and verifies again:

```text
token        : matin.1000.14950887eb8cd
verify @ 100 : Some("matin")
after logout : Some("matin")  (a stolen copy still works)
with deny-list: None
deny-list entries to keep until expiry: 1
```

`verify` looks only at the token and the clock. That is the feature: no table, no lookup. It is also why "log out" has nothing to delete: the server never wrote anything down, so there is nothing to un-write. A copy of the token that was stolen, or just left on a lost laptop, works until its `exp`.

The fix on the last two lines is a **deny-list**: remember every revoked token until it would have expired anyway, and check that list on every request. Look at what that is: a table, consulted on every request, that every server must share. It is a session store, with the id replaced by a much bigger token. You have rebuilt the stateful design, plus a signature check you no longer needed.

That is the honest summary of the trade. Statelessness is not "free scaling". It is "no revocation unless you add the state back", and tokens made short-lived (minutes) shrink the problem without removing it. 3.7.4 is the lesson about living with that.

### Cookie flags: `HttpOnly`, `Secure`, `SameSite`

A cookie is set by a response header and sent back by the browser automatically on matching requests. The header is plain text, and getting it wrong is silent. `examples/02-cookie-attributes.rs` prints the three shapes you will use:

```text
dev  : sid=3f9a; Path=/; HttpOnly; SameSite=Lax; Max-Age=3600
prod : sid=3f9a; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age=3600
bank : sid=3f9a; Path=/; HttpOnly; Secure; SameSite=Strict; Max-Age=900

HttpOnly -> page JavaScript cannot read it (limits XSS theft)
Secure   -> only sent over HTTPS (limits network sniffing)
SameSite -> not sent on most cross-site requests (limits CSRF)
Max-Age  -> the browser forgets it; the SERVER still decides
```

Each attribute closes one door:

- **`HttpOnly`**: `document.cookie` does not show it. A script injected into your page (XSS) cannot read the id and send it away. It can still *use* the session: it can make requests from inside the page, and the browser attaches the cookie. So `HttpOnly` stops theft, not abuse. Always set it on a session cookie.
- **`Secure`**: the browser only sends the cookie over HTTPS. On plain HTTP, anyone on the network path could read it. Behind HTTPS in production it is mandatory; on `http://localhost` during development you turn it off (the lesson's `AppState` has a `secure` flag for exactly that; [3.4.1](../../04-configuration-and-app-structure/01-config-and-secrets/README.md) shows how to read such a flag from the environment).
- **`SameSite`**: whether the browser attaches the cookie to a request that starts on *another site*. `Strict` never does, not even when a user clicks a link to you from another site, so they appear logged out on arrival. `Lax` sends it on top-level navigations with a safe method (a link click) but not on cross-site `POST`s, form submissions and fetches. `None` always sends it, and browsers only accept `None` together with `Secure`. Say it explicitly: browsers differ in what a missing `SameSite` means.
- **`Max-Age`**: how long the browser keeps it. `Max-Age=0` deletes it, which is how logout clears the cookie. This is a *request to the browser*. The server's own expiry is the one that counts, because a client can keep a cookie it was told to drop.
- **`Path=/`**: sent for every path on the site.

"Site" is not "origin". An origin (3.2.5) is scheme + host + port. A site is the scheme plus the registrable domain, so `http://localhost:3000` and `http://localhost:5173` are different origins but the *same site*, and `app.example.com` and `evil.example.net` are different sites, but `a.example.com` and `b.example.com` are the same site. `SameSite` therefore does not protect you from a hostile sibling subdomain.

### Where to keep a token: cookie or `localStorage`

If you build the stateless design, the client must store the token somewhere, and the browser offers two places. Neither is free:

| | `HttpOnly` cookie | `localStorage` |
|---|---|---|
| Readable by injected JavaScript (XSS) | no | yes: one line, `localStorage.getItem(...)` |
| Sent automatically with every request | yes | no: your code adds an `Authorization` header |
| So exposed to CSRF | yes (defences below) | no, nothing is attached for the attacker's page |
| Size and lifetime controls | `Max-Age`, flags | none, and it persists until cleared |

So `localStorage` trades CSRF for XSS theft, and an HttpOnly cookie trades the other way. XSS is the more common bug in real apps, and the XSS of a `localStorage` token is full account takeover from anywhere (the attacker copies the token and leaves). That is why an HttpOnly cookie is the better home for a credential, whichever design is behind it. A JWT in an HttpOnly cookie is legal, and it keeps every revocation problem above.

### CSRF: the price of cookies

Cookies are attached by the *browser*, not by your page's code. A page on `evil.example` can contain a form that posts to your `/logout-all`, and the victim's browser will attach your cookie to it. The attacker never sees the cookie; they borrow it. That is **cross-site request forgery** (CSRF), and OWASP lists it as a standing concern for any cookie-authenticated app.

Defences, in the order to reach for them:

1. **`SameSite=Lax` or `Strict`** on the session cookie, which stops the cross-site `POST` from carrying it in modern browsers.
2. **State changes are never `GET`.** `Lax` still sends the cookie on a link click, so `GET /delete-account` would be forgeable. 3.1.3's safe-method rule is a security rule here.
3. **Check the `Origin` header** on state-changing requests. Browsers send `Origin` on cross-origin `POST`s and attackers' pages cannot change it. This lesson's `origin_ok` passes a request with no `Origin` (not a browser form or fetch) and a request whose `Origin` is exactly the one origin you serve, and refuses everything else.
4. **A CSRF token** (a secret per session that your own page puts in a header or hidden field) is the classic defence and OWASP's primary recommendation. Django does this with `CsrfViewMiddleware` and `{% csrf_token %}`. This lesson stops at `SameSite` plus `Origin`, which OWASP calls defence in depth, and you should add the token for anything that matters.

CORS (3.2.5) is not a CSRF defence. It controls which pages may *read* responses; the forged `POST` is sent and executed either way.

### Session fixation

Here is a login handler with a quiet flaw. It keeps whatever session id the browser came with and marks that session as logged in. `examples/03-session-fixation-trap.rs` runs it:

```text
victim logs in; the browser's id is still: planted-by-attacker
attacker uses the id they planted: Some("victim")
```

An attacker who can plant an id in the victim's browser *before* the login (through a subdomain, a URL parameter, a shared computer) now knows a valid, authenticated id. The attack works because the id did not change when privilege changed. The rule, from OWASP's session management guidance: **issue a new id at login, and invalidate the old one**. Django's `login()` does it by calling `request.session.cycle_key()`. The lesson's `login` handler does it in two lines:

```rust
if let Some(old) = sid_of(&headers) {
    s.store.revoke(&old); // the planted id dies
}
let id = s.store.create(user); // and the user gets a fresh one
```

### Size and scaling: where stateless honestly wins

The stateless design has real strengths, and you should know when they apply:

- **Many services, one login.** If five backend services must each decide "who is this?" without calling an auth service on every request, a token they can all verify with a shared key (or a public key) removes the shared table. This is the situation JWT was designed for.
- **Short-lived access.** A token that lives five minutes needs no revocation story beyond "wait". 3.7.4 pairs a short token with a longer-lived, revocable one.
- **No lookup on the hot path.** A signature check does not touch a database. A session lookup in a fast store is cheap, but it is a dependency.

Against that, the costs you pay for the rest of the app's life: the token is sent on *every* request (a 32-character id against hundreds of bytes of headers and claims), its claims go stale (a user demoted to "reader" keeps "admin" in the token they hold), and the revocation problem above.

For scaling the stateful design, remember that your `SessionStore` here is in one process: a second server instance would not know about sessions the first created. Real deployments put the table in something shared (a database, or a cache such as Redis) or route a user to one instance. The Postgres module begins at [3.5.1 — Connecting and pooling](../../05-postgres-and-sqlx/01-connecting-and-pooling/README.md). The store being a struct behind `Arc` (3.2.3's shape, held in `AppState` as in 3.4.2) is what makes that swap local.

### The honest verdict

- One web app, one backend, your own frontend: **server-side sessions in an HttpOnly cookie.** Revocation, logout, "log out everywhere" and role changes all just work, and Django users already trust it.
- Many services, or third parties that must verify without calling you: **signed tokens, short-lived**, with a plan for revocation (3.7.3, 3.7.4).
- "JWT for everything" is usually a mistake because it picks the harder design for the case where its single advantage does not apply.

---

## Hands on

Run the three examples that compile. (`04` and `05` are broken on purpose and sit behind the `broken` feature; "Errors you will meet" shows them.)

```sh
cargo run -p p3-07-02-sessions-vs-jwt --example 01-token-revocation-needs-state
cargo run -p p3-07-02-sessions-vs-jwt --example 02-cookie-attributes
cargo run -p p3-07-02-sessions-vs-jwt --example 03-session-fixation-trap
```

The outputs are the blocks in "The concept". The tests start out red, because every function in `src/lib.rs` that you implement is a `todo!()` with a doc comment that says exactly what it must do:

```sh
cargo test -p p3-07-02-sessions-vs-jwt --test store_test 2>&1 | grep 'test result'
```

```text
test result: FAILED. 0 passed; 12 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

When everything is green, run the real server (it listens on `127.0.0.1:3180`) and use `curl` with a cookie jar. These transcripts were captured against the solution. The session id and the `date` header change every run.

```sh
cargo run -p p3-07-02-sessions-vs-jwt &
curl -si -c /tmp/jar -d matin http://127.0.0.1:3180/login; echo
curl -s -b /tmp/jar -w '\n%{http_code}\n' http://127.0.0.1:3180/me
```

```text
HTTP/1.1 200 OK
content-type: text/plain; charset=utf-8
set-cookie: sid=2293830ce5e8d16e6f82d916e60ff9e1; Path=/; HttpOnly; SameSite=Lax; Max-Age=3600
content-length: 13
date: Sun, 04 Oct 2026 11:16:23 GMT

welcome matin
matin
200
```

That is the lesson's `Set-Cookie` builder on a real response. Now the refusals: a forged cookie, then a logout that claims to come from another site:

```sh
curl -s -w '\n%{http_code}\n' -H 'Cookie: sid=0123456789abcdef0123456789abcdef' http://127.0.0.1:3180/me
curl -s -w '\n%{http_code}\n' -b /tmp/jar -H 'Origin: http://evil.example' -X POST http://127.0.0.1:3180/logout
curl -s -w '\n%{http_code}\n' -b /tmp/jar http://127.0.0.1:3180/me
```

```text
not logged in
401
forbidden origin
403
matin
200
```

The forged logout was refused, and the session survived. Now the point of the whole lesson: save a copy of the cookie (what a thief would hold), log out, and try the copy:

```sh
COPY=$(grep sid /tmp/jar | awk '{print $7}')
curl -si -b /tmp/jar -X POST http://127.0.0.1:3180/logout
curl -s -w '\n%{http_code}\n' -H "Cookie: sid=$COPY" http://127.0.0.1:3180/me
```

```text
HTTP/1.1 204 No Content
set-cookie: sid=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0
date: Sun, 04 Oct 2026 11:16:24 GMT

not logged in
401
```

The browser was told to drop the cookie (`Max-Age=0`), but the copy is dead for a better reason: the *server* deleted the row. Compare that with the token in example 01. Finally, fixation: log in while presenting a planted id.

```sh
curl -si -H 'Cookie: sid=attackerchosenid' -d matin http://127.0.0.1:3180/login | grep -i -E '^HTTP|set-cookie'
curl -s -w '\n%{http_code}\n' -H 'Cookie: sid=attackerchosenid' http://127.0.0.1:3180/me
```

```text
HTTP/1.1 200 OK
set-cookie: sid=<32 hex>; Path=/; HttpOnly; SameSite=Lax; Max-Age=3600
not logged in
401
```

(The 32 hex characters are replaced here by `<32 hex>`; the real output has a random id.) The user got a fresh id, and the planted one authenticates nobody. Stop the server when you are done (`kill %1` in the same shell).

Then try these:

1. Log in twice with `curl` and list what each `Set-Cookie` returned. Are the two ids related? Should they be?
2. In `src/lib.rs`, change `SameSite::Lax` in `cookie_opts` to `SameSite::None` and run the tests. What breaks, and which rule of `set_cookie` is the cause?
3. Add `; Domain=example.com` to a cookie value passed to your `set_cookie` and see which error you get.

---

## Errors you will meet

### `E0004` — a `SameSite` the `match` forgot

```text
error[E0004]: non-exhaustive patterns: `SameSite::None` not covered
  --> phase3-backend-foundations\07-auth-and-security\02-sessions-vs-jwt\examples\04-set-cookie-samesite-non-exhaustive-broken.rs:13:11
   |
13 |     match same_site {
   |           ^^^^^^^^^ pattern `SameSite::None` not covered
   |
note: `SameSite` defined here
  --> phase3-backend-foundations\07-auth-and-security\02-sessions-vs-jwt\examples\04-set-cookie-samesite-non-exhaustive-broken.rs:6:6
   |
 6 | enum SameSite {
   |      ^^^^^^^^
...
 9 |     None,
   |     ---- not covered
   = note: the matched value is of type `SameSite`
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern or an explicit pattern as shown
   |
15 ~         SameSite::Lax => "SameSite=Lax",
16 ~         SameSite::None => todo!(),
   |

For more information about this error, try `rustc --explain E0004`.
error: could not compile `p3-07-02-sessions-vs-jwt` (example "04-set-cookie-samesite-non-exhaustive-broken") due to 1 previous error
```

**What the compiler is objecting to:** `SameSite` has three variants and the `match` handles two. A `match` has to cover every case, and the compiler names the missing one. `SameSite::None` is the interesting one: it is the variant with a rule attached (browsers only accept it with `Secure`).

**The fix:** add the arm, and let it mean something:

```rust
SameSite::None => "SameSite=None",
```

**Why this is the fix:** the `todo!()` the compiler suggests would panic in production the first time someone chose `None`. A real arm forces you to decide what `None` does, and your `set_cookie` goes one step further: it refuses `None` without `Secure`. A wildcard arm (`_ =>`) would also compile, and it would hide the next variant someone adds. An exhaustive match is a checklist that grows with the enum.

### `E0277` — a clock that is not `Sync`

```text
error[E0277]: `Cell<u64>` cannot be shared between threads safely
   --> phase3-backend-foundations\07-auth-and-security\02-sessions-vs-jwt\examples\05-clock-with-cell-not-sync-broken.rs:21:27
    |
 21 |     let t = thread::spawn(move || for_handler.now());
    |             ------------- ^^^^^^^^^^^^^^^^^^^^^^^^^ `Cell<u64>` cannot be shared between threads safely
    |             |
    |             required by a bound introduced by this call
    |
    = help: within `ManualClock`, the trait `Sync` is not implemented for `Cell<u64>`
    = note: if you want to do aliasing and mutation between multiple threads, use `std::sync::RwLock` or `std::sync::atomic::AtomicU64` instead
note: required because it appears within the type `ManualClock`
   --> phase3-backend-foundations\07-auth-and-security\02-sessions-vs-jwt\examples\05-clock-with-cell-not-sync-broken.rs:9:8
    |
  9 | struct ManualClock(Cell<u64>);
    |        ^^^^^^^^^^^
    = note: required for `Arc<ManualClock>` to implement `Send`
note: required because it's used within this closure
   --> phase3-backend-foundations\07-auth-and-security\02-sessions-vs-jwt\examples\05-clock-with-cell-not-sync-broken.rs:21:27
    |
 21 |     let t = thread::spawn(move || for_handler.now());
    |                           ^^^^^^^
note: required by a bound in `spawn`
   --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\std\src\thread\functions.rs:128:8
    |
125 | pub fn spawn<F, T>(f: F) -> JoinHandle<T>
    |        ----- required by a bound in this function
...
128 |     F: Send + 'static,
    |        ^^^^ required by this bound in `spawn`

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p3-07-02-sessions-vs-jwt` (example "05-clock-with-cell-not-sync-broken") due to 1 previous error
```

**What the compiler is objecting to:** a clock a test can move needs interior mutability (a `&self` method that changes a number), and `Cell` is the first tool people reach for (2.6.5). But a `Cell` is only safe on one thread. `Arc<ManualClock>` can be sent to another thread only if `ManualClock` is `Sync`, and `Cell<u64>` is not. Every `axum` handler runs on a runtime thread, so a clock inside `AppState` has the same requirement. This is 2.8.4's rule, and the compiler's `note:` already names the repair.

**The fix:** use the thread-safe cell for a plain number:

```rust
struct ManualClock(std::sync::atomic::AtomicU64);
```

**Why this is the fix:** `AtomicU64` can be read and written through `&self` from many threads, so it is `Sync`. That is what the lesson's `ManualClock` uses (`fetch_add` for `advance`, `store` for `set`). A `Mutex<u64>` would work too, and costs more for a single number.

### No error at all: a login that keeps the old id

```text
victim logs in; the browser's id is still: planted-by-attacker
attacker uses the id they planted: Some("victim")
```

**What's actually broken:** `examples/03-session-fixation-trap.rs` compiles, runs, and "logs in" correctly. Its `login` inserts the *incoming* id into the session table instead of minting a new one. Every status code is right and every test that only checks "login then `/me` works" passes. The vulnerability is that an id the attacker chose now authenticates the victim.

**The fix:** never carry an id across a login. Revoke the one that came in and create a new one, as the lesson's `login` does.

**Why this is the fix:** the attack needs the attacker to *know* the authenticated id. If the id is generated after the password check, from the operating system's random source, the attacker had no way to learn it. The compiler cannot see this; only a test that logs in with a planted id and then asserts the old id is dead does, and `tests/api_test.rs` has exactly that test.

---

## Exercises

### Warm up

<details>
<summary>A user clicks "log out" in an app whose only credential is a signed token with a one-hour expiry, and the server keeps no state. A copy of the token was stolen ten minutes ago. Is the thief locked out?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

No. The server never wrote the token down, so there is nothing to delete. Verification only checks the signature and the expiry, so the copy works for the remaining fifty minutes. Logging out in the browser deletes the browser's copy, not the thief's. To end it early, the server would have to keep a deny-list, which is state.

</details>

<details>
<summary>A session cookie has <code>HttpOnly</code> set. An attacker injects a script into your page. What can and what can't the script do with the session?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

It cannot read the cookie, so it cannot copy the id and use it from another machine. It can still make requests from the victim's own browser, and the browser attaches the cookie, so it can act as the user for as long as the page is open. `HttpOnly` limits theft, not abuse, which is why it does not replace fixing the XSS.

</details>

<details>
<summary>The cookie is <code>SameSite=Lax</code>. A page on <code>evil.example</code> auto-submits a <code>POST</code> form to your <code>/logout-all</code>. Is the cookie sent? What about a link to <code>GET /delete-account</code> that the user clicks?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

The form `POST` does not carry the cookie, so the forged logout fails. The link click *does* carry it: `Lax` allows the cookie on top-level navigations with a safe method. That is why a state change must never be a `GET`, and why 3.1.3's "safe methods change nothing" is also a security rule.

</details>

<details>
<summary>A session is created at time 1000 with a lifetime of 100. Is it live at 1099? At 1100?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

Live at 1099, dead at 1100. The rule is `now < expires_at`, with `expires_at = 1100`. Choosing `<` rather than `<=` means a session never lives past its stated lifetime, and `a_session_is_live_until_the_instant_it_expires` tests both sides of the boundary.

</details>

<details>
<summary>Why must <code>login</code> create a new id even if the request already carried a valid one for this very user?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

An id that existed before the password was checked might be known to someone else (planted, or from a shared machine). Privilege has changed, so the identifier must change with it. Keeping the old id is exactly session fixation. The cost of a new id is nothing, so there is no reason to make an exception.

</details>

### Repair

Fix the three broken examples:

1. `examples/04-set-cookie-samesite-non-exhaustive-broken.rs` compiles: add the missing arm with the right text, not a `todo!()`. Build it with `--features broken` to check.
2. `examples/05-clock-with-cell-not-sync-broken.rs` compiles and runs: make the clock `Sync`.
3. `examples/03-session-fixation-trap.rs` runs and the attacker's planted id is **not** authenticated afterwards: make `login` return a new id and drop the old one.

### Implement

Everything in `src/lib.rs` that is a `todo!()` except the two for the next rungs: `SessionStore::create`, `lookup`, `revoke`, `revoke_all_for`, and the pure functions `set_cookie`, `expire_cookie`, `session_id_from_cookie_header`, `origin_ok`. Each doc comment is the complete specification (the exact attribute order, the characters that are refused, the order the errors are checked in), so you should never need to open the tests to know what to build. The handlers and `app` are given: once the functions work, the whole API works.

```sh
cargo test -p p3-07-02-sessions-vs-jwt --test store_test --test cookie_test --test api_test
```

`tests/store_test.rs` (12 tests) checks the store with plain calls and a clock moved by hand. `tests/cookie_test.rs` (19 tests) checks the pure functions. `tests/api_test.rs` (14 tests) checks the whole stack through `oneshot`: login, `/me`, logout, "log out everywhere", fixation and the `Origin` check.

### Build

Sliding expiry. Implement `SessionStore::touch` (the doc comment specifies it), then make the `/me` handler call it so that an active user is not logged out in the middle of working, and the cookie's `Max-Age` stays in step. Decide what the response should carry to refresh the cookie, and write your own test in a new `tests/sliding_test.rs` that moves the clock, calls `/me`, and shows the session outliving its original lifetime. `tests/build_test.rs` already checks `touch` itself (2 of its 3 tests).

### Challenge (optional)

"Log out all my other devices". Implement `SessionStore::revoke_others` (the third test in `tests/build_test.rs`), then add `POST /logout-others` that keeps the caller's own session, applying the same `Origin` check as the other state-changing routes. This is where the stateful design pays for itself: the stateless design cannot do this at all without a deny-list. There are no provided tests for the route: write your own for the happy path, the cross-origin refusal and the not-logged-in answer.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Stateful session | the client holds a random id; the server keeps `id -> user` | server-rendered apps, first-party web apps |
| Stateless token | a self-contained signed claim set; the server keeps only the key | service-to-service calls, short-lived access tokens |
| Revocation | ending a credential before it expires | logout, stolen device, password change |
| Deny-list | a server-side set of revoked tokens, checked on every request | stateless tokens that need revocation (3.7.4) |
| Injectable clock | time passed in as a trait object so tests can move it | expiry logic, rate limits, caches |
| `HttpOnly` / `Secure` / `SameSite` | cookie attributes against script theft, sniffing and cross-site sends | every session cookie |
| CSRF | a foreign site makes the victim's browser send an authenticated request | every cookie-authenticated, state-changing route |
| Session fixation | an attacker plants a session id and waits for the victim to log in on it | every login handler |

### What you now know

- A session is an opaque id in a cookie and a row on the server. A stateless token is the facts plus a signature. The first can be ended by deleting a row; the second cannot be ended without adding that state back.
- "JWT for everything" is usually a mistake: it gives up revocation and fresh claims to buy scaling you rarely need. Its real strength is many services verifying without a shared table.
- A session cookie should be `HttpOnly`, `Secure` in production, explicit about `SameSite`, and given a `Max-Age`, but the server's own expiry is the one that counts.
- An HttpOnly cookie resists XSS theft and needs CSRF defences. A `localStorage` token is immune to CSRF and wide open to XSS theft.
- CSRF is defended with `SameSite`, no state changes on `GET`, an `Origin` check, and a CSRF token. CORS is not a CSRF defence.
- Always mint a new session id at login and drop the old one.

### What comes back later

- **Real signed tokens: claims, signing keys, verification as a `tower` middleware**: [3.7.3 — JWTs and `tower` middleware](../03-jwt-and-tower-middleware/README.md)
- **The part tutorials skip: short tokens, refresh tokens, rotation and ending a login early**: [3.7.4 — Refresh-token rotation and revocation](../04-refresh-token-rotation-and-revocation/README.md)
- **What the logged-in user is allowed to do once you know who they are**: [3.7.5 — Modelling RBAC and permissions](../05-modelling-rbac-and-permissions/README.md)
- **Moving the session table out of the process so more than one server shares it**: [3.5.1 — Connecting and pooling](../../05-postgres-and-sqlx/01-connecting-and-pooling/README.md)

### Can you explain?

- What does a stateless token not let you do, and what does a deny-list have to be to fix it?
- Why does logging out in the browser not kill a copy of a stateless token, but does kill a copy of a session id?
- What does `HttpOnly` stop, and what can an XSS script still do with an `HttpOnly` cookie?
- Why is a `GET` that changes state forgeable even with `SameSite=Lax`?
- What is the difference between an origin and a site, and why does `SameSite` not protect you from a sibling subdomain?
- Why does `login` have to issue a new session id even when the request already had one?
- When would you choose signed tokens over sessions, and when is that choice a mistake?

---

## Going further

- [OWASP Session Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html): id entropy, cookie attributes, fixation, expiry and logout.
- [OWASP CSRF Prevention Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html): tokens, `Origin` checks and `SameSite`, and how they layer.
- [Django: How to use sessions](https://docs.djangoproject.com/en/stable/topics/http/sessions/): backends (database, cache, `signed_cookies`), `cycle_key`, expiry settings.
- [MDN: Using HTTP cookies](https://developer.mozilla.org/en-US/docs/Web/HTTP/Cookies): every attribute, with browser notes.
- [RFC 6265 — HTTP State Management Mechanism](https://www.rfc-editor.org/rfc/rfc6265): the cookie spec, including the exact syntax `set_cookie` follows.
- [RFC 7519 — JSON Web Token](https://www.rfc-editor.org/rfc/rfc7519): the stateless design, ahead of 3.7.3.
- [2.8.4 — `Send` and `Sync`](../../../phase2-intermediate/08-concurrency/04-send-and-sync/README.md): the rule behind today's `E0277`.

# 3.7.4 — Refresh-token rotation and revocation

## At a glance

After this lesson you can:

- Explain why an API hands out a short-lived access token *and* a long-lived refresh token, and what each one is for.
- Rotate refresh tokens so every token works exactly once, and say what it means when an already-used one shows up again.
- Store only a SHA-256 hash of each refresh token, so a leaked database does not leak working sessions.
- Revoke one session (logout) or every session of a user (logout everywhere), and say honestly what revocation cannot undo.
- Write the `rotate` function with every outcome (new pair, `Expired`, `Unknown`, `ReuseDetected`) spelled out, and test it without ever sleeping.

**Time:** ~75 minutes · **Prerequisites:**
[3.7.3 — JWTs and `tower` middleware](../03-jwt-and-tower-middleware/README.md)

---

## Why this matters

A signed access token, as you met it in 3.7.3, has one awkward property: once it is issued, the server cannot take it back. It is valid until it expires, because checking it needs no lookup, and that is the whole reason to use one. If it lives for a week, a stolen copy works for a week. If it lives for fifteen minutes, the user has to log in again four times an hour.

The standard way out is two tokens. A short-lived **access token** goes on every request. A long-lived **refresh token** is used rarely, only to ask for a new access token. Django's session cookie did both jobs at once, and the server could delete the session row to log you out. Here the refresh token is the key to that row: a lookup the server controls, and so the place where a session can be ended.

But a long-lived secret that sits in a phone or a browser is exactly what an attacker wants. This lesson is about making that secret hard to abuse: every token works once, a replay is treated as theft, and the server never stores the token itself. OWASP's session guidance and the OAuth 2.0 Security Best Current Practice both describe this idea, called refresh-token rotation. You will build it, in memory, with no database.

---

## The concept

### Two tokens, two jobs

| | Access token | Refresh token |
|---|---|---|
| Sent on | every API request | only to the refresh endpoint |
| Lifetime | short (here 15 minutes) | long (here 7 days) |
| Checked by | its signature, no lookup | a lookup in the server's store |
| Can the server revoke it? | not before it expires | yes, delete the record |

In this lesson the access token is a stand-in string, `access.<user_id>.<expires_at>`; 3.7.3 showed how to make it a real signed token, and nothing here depends on the difference. The interesting object is the refresh token and the store behind it.

### Rotation: every refresh token works once

A refresh token is single-use. When a client presents one, the server marks it **used** and hands back a *new* pair, including a brand-new refresh token. The client throws the old one away. After a few rounds the store holds a chain of tokens, all but the newest already used:

```senpai-visual
{"kind":"concept","labels":["login: token A issued","refresh with A: A marked used, token B issued","refresh with B: B marked used, token C issued","only C is still usable","A, B, C form one family"]}
```

`examples/01-login-and-rotate.rs` runs it. The refresh tokens are random, so yours differ; the access token is deterministic because the clock is a `ManualClock` the example moves by hand:

```text
login   access=access.matin.1700000900
        refresh=c19ea5a230d24616
rotate1 access=access.matin.1700001500
        refresh=bda069e2c1904fd4
rotate2 access=access.matin.1700002100
        refresh=21f110143ad1435f
rotate3 access=access.matin.1700002700
        refresh=18968df2fd994054
records stored: 4
hash of newest: 0c88bc768479d8f5
```

Four records for one login plus three rotations: the old ones are kept, marked used, and that is deliberate. The next section shows why.

### Reuse detection: an old token coming back means theft

Think about who can still hold token A after the real client rotated it into B. Only someone who copied A earlier: a thief. The real client does not have it any more. So when a *used* token is presented, the server cannot tell the thief from the victim, but it knows one of them is not the real client. The safe response is to stop trusting the whole **family** (every token descended from one login) and make everybody log in again.

```senpai-visual
{"kind":"concept","labels":["thief copies token A","real client rotates A into B","thief presents A: already used","server revokes the whole family","real client's B is dead too: login again"]}
```

`examples/02-reuse-detection.rs`:

```text
client rotated, records: 2
thief replays old token:  Err(ReuseDetected)
records after the replay: 0
client's newest token:    Err(Unknown)
```

The thief lost, but so did the honest client: its newest token is gone too. That is the price. It is also why the used records have to stay in the store: a deleted record would make a replayed token look merely `Unknown`, and the server could not tell a replay from garbage.

### Store the hash, never the token

Think of the refresh store as a table that someone will eventually dump. If it holds the tokens themselves, the dump is a pile of working sessions. So the store keeps `hash_token(token)`, the lowercase hex SHA-256, and the server hashes whatever a client presents before looking it up. `examples/03-what-a-leak-shows.rs` plays the attacker with a copy of the store (the values are random and differ on every run):

```text
leaked rows:     1
leaked value:    5a524203b1b10f931aad0641a20e488e3e5675a791c42cfa4eb892750bc8c0fc
real token:      ef7443cb876f408d9e72318f917e4dcfd2cd08947247402998a2d0870d9513e2
leak == token?   false
hash of token:   true
replay the hash: Err(Unknown)
```

The leaked value is useless as a token: hashing it again gives a different value that is not in the store. 3.7.1 told you to hash passwords with a deliberately slow function. A refresh token does not need one. It is 64 random hex characters, 256 bits' worth, so there is nothing to guess and a fast hash is enough. A password is short and chosen by a human, which is why that one needs the slow hash.

### Logout is deleting a family

Logout is the other half of revocation: the user, not a thief, ends a session. The server deletes the family of the presented token, so even the newest token stops working. `logout_all(user)` deletes every family of one user, one per device. What revocation cannot do is recall an access token already issued: it stays valid until it expires, at most fifteen minutes here. Shortening that window, or keeping a denylist of access tokens, is a trade-off you pay for with lookups, and it is the reason the refresh token is the thing worth revoking.

### Time is injected

`rotate` has to ask whether a token has expired. If it called the system clock directly, a test of a seven-day expiry would have to wait seven days. So the service is generic over a small `Clock` trait with `now() -> u64`. Production uses `SystemClock`; tests use `ManualClock`, whose `advance(seconds)` jumps forward instantly. Clones of a `ManualClock` share one counter, so a test keeps one clone and hands the other to the service.

```senpai-visual
{"kind":"result","labels":["rotate(presented)","hash the token, look it up","not found: Unknown","already used: revoke family, ReuseDetected","expired: Expired","fresh: mark used, Ok new pair"]}
```

---

## Hands on

```sh
cargo run -p p3-07-04-refresh-token-rotation-and-revocation --example 01-login-and-rotate
cargo run -p p3-07-04-refresh-token-rotation-and-revocation --example 02-reuse-detection
cargo run -p p3-07-04-refresh-token-rotation-and-revocation --example 03-what-a-leak-shows
```

Then the three broken ones. Two need the `broken` feature; the third runs normally and is simply wrong:

```sh
cargo build -p p3-07-04-refresh-token-rotation-and-revocation --example 04-token-moved-broken --features broken
cargo run -p p3-07-04-refresh-token-rotation-and-revocation --example 05-unwrap-on-reuse-broken --features broken
cargo run -p p3-07-04-refresh-token-rotation-and-revocation --example 06-never-marks-used-trap
```

The examples use a finished copy of the store in `examples/common/store.rs`, so they run before you write anything. Then try these:

1. In `02-reuse-detection`, make the real client rotate once more before the thief replays. Which token does the thief hold now, and what is the result?
2. In `01-login-and-rotate`, advance the clock by seven days before the last rotation. Which error comes out?
3. In `03-what-a-leak-shows`, call `svc.logout` with the leaked hash. What does it return, and why?

---

## Errors you will meet

### `E0382` — using a refresh token after handing it away

```text
error[E0382]: borrow of moved value: `pair.refresh_token`
    --> phase3-backend-foundations\07-auth-and-security\04-refresh-token-rotation-and-revocation\examples\04-token-moved-broken.rs:17:27
     |
  16 |     save_to_cookie(pair.refresh_token);
     |                    ------------------ value moved here
  17 |     let next = svc.rotate(&pair.refresh_token);
     |                           ^^^^^^^^^^^^^^^^^^^ value borrowed here after move
     |
     = note: move occurs because `pair.refresh_token` has type `String`, which does not implement the `Copy` trait
     = note: borrow occurs due to deref coercion to `str`
note: deref defined here
    --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\alloc\src\string.rs:2832:5
     |
2832 |     type Target = str;
     |     ^^^^^^^^^^^

For more information about this error, try `rustc --explain E0382`.
error: could not compile `p3-07-04-refresh-token-rotation-and-revocation` (example "04-token-moved-broken") due to 1 previous error
```

**What the compiler is objecting to:** `save_to_cookie` takes a `String` by value, so passing `pair.refresh_token` moves the token out of `pair`. The next line borrows a value that is already gone.

**The fix:** let the function borrow instead:

```rust
fn save_to_cookie(token: &str) { /* write the cookie */ }
save_to_cookie(&pair.refresh_token);
```

**Why this is the fix:** the cookie writer only needs to read the token. Borrowing leaves the owner intact, and the `&` at the call site tells the reader that no second copy of the secret was made.

### A run-time panic: `unwrap` on a rotation that was refused

```text
thread 'main' (22032) panicked at phase3-backend-foundations\07-auth-and-security\04-refresh-token-rotation-and-revocation\examples\05-unwrap-on-reuse-broken.rs:14:50:
called `Result::unwrap()` on an `Err` value: ReuseDetected
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
error: process didn't exit successfully: `target\debug\examples\05-unwrap-on-reuse-broken.exe` (exit code: 101)
```

(The number in parentheses is the thread's id and changes every run.)

**What's actually broken:** `rotate` returned `Err(ReuseDetected)` and the example called `.unwrap()` on it. A refused rotation is not a bug in your program; it is a normal answer, and often the interesting one. Crashing the handler turns a security signal into a 500.

**The fix:** match on the error and answer each case on purpose:

```rust
match svc.rotate(&token) {
    Ok(pair) => println!("new pair: {pair:?}"),
    Err(RefreshError::ReuseDetected) => println!("log this: possible theft"),
    Err(other) => println!("log in again: {other:?}"),
}
```

**Why this is the fix:** every error ends with "log in again" for the client, but the server can treat `ReuseDetected` differently, because it is the only one that suggests an attack.

### No error at all: a `rotate` that never marks the token used

```text
client rotates: Some("token-2")
thief replays:  Some("token-3")  (should have been refused)
```

**What's actually broken:** `examples/06-never-marks-used-trap.rs` compiles and runs, and every call returns a plausible `Some`. Its `rotate` hands out a new token but leaves the old one valid. Rotation then only makes the store bigger; a stolen token works for as long as the thief likes, and nobody is told.

**The fix:** change the old record's state when the new token is issued: mark it used (or remove it), so the next presentation is recognised as a replay.

**Why this is the fix:** the guarantee is "every token works once", and nothing in the types enforces it. Only a test that presents the same token twice and expects the second to fail does, which is why the tests in this lesson do exactly that.

---

## Exercises

### Warm up

<details>
<summary>A client's network drops after the server rotated its token, so the client retries with the same old token. What does the server answer?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

`ReuseDetected`, and the family is revoked. The server cannot tell a retry from a replay, so the honest client is logged out too. That is the known cost of strict rotation, and the Challenge below is about softening it.

</details>

<details>
<summary>An attacker steals a copy of the refresh table. Can they call the refresh endpoint with the values in it?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

No. The table holds SHA-256 hashes. A presented hash gets hashed again, which matches nothing, so the answer is `Unknown`. This is example `03`.

</details>

<details>
<summary>A token has been used and is also past its expiry. Which error does <code>rotate</code> return?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

`ReuseDetected`. The used check comes before the expiry check, because a replayed token is a theft signal whatever its age.

</details>

<details>
<summary>The user taps "log out". Does their current access token stop working at once?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

No. Logout deletes the refresh family, so no new access token can be minted, but the existing access token stays valid until it expires (at most `ACCESS_TTL`, 15 minutes). Ending it sooner needs a lookup on every request.

</details>

### Repair

Fix all three broken examples:

1. `examples/04-token-moved-broken.rs` compiles without cloning the token.
2. `examples/05-unwrap-on-reuse-broken.rs` prints the outcome of the second rotation instead of panicking.
3. `examples/06-never-marks-used-trap.rs` refuses the thief's replay: the second `rotate` on the same token returns `None`.

### Implement

Two functions in `src/lib.rs`. Each is fully specified in its doc comment, so you never need to open the tests:

```sh
cargo test -p p3-07-04-refresh-token-rotation-and-revocation
```

- `hash_token`: the lowercase hex SHA-256 of a token, 64 characters. `sha2::{Digest, Sha256}` is not imported in the skeleton, so add the `use` yourself.
- `rotate`: the four outcomes in order: `Unknown`, then `ReuseDetected` (family revoked), then `Expired`, then a new pair for the same user and family.

### Build

`logout` and `logout_all` in the same file. `logout` removes a whole family given one of its tokens and reports whether the token was known. `logout_all` removes every family of one user and counts the families.

### Challenge (optional)

Strict rotation punishes a legitimate retry. Real systems add a short **grace window**: a used token presented again within, say, 10 seconds of its rotation is refused without revoking the family. Add a `used_at` time to the record and a `GRACE_SECONDS` constant, and write the tests with `ManualClock`. Decide what a reuse inside the window returns, and write that decision in the doc comment.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Access token | short-lived credential sent with every request | every authenticated API call |
| Refresh token | long-lived, single-use credential for getting a new pair | the refresh endpoint |
| Rotation | issuing a new refresh token on every refresh and retiring the old one | limiting a stolen token's lifetime |
| Token family | all refresh tokens descended from one login | revoking one session at once |
| Reuse detection | treating a used token's return as theft and revoking its family | the `ReuseDetected` outcome |
| Revocation | ending a session on the server's side | logout, logout everywhere, theft response |
| Injectable clock | time passed in as a value so tests control it | any code that checks expiry |

### What you now know

- Access tokens are cheap to check and impossible to recall; refresh tokens are checked against a store and so can be revoked.
- Rotation makes each refresh token single-use, which turns a replay into a detectable event.
- Reuse of a used token revokes its whole family, and the honest client pays for it with a new login.
- A store that keeps only SHA-256 hashes of high-entropy tokens leaks nothing a thief can present.
- Logout deletes a family; `logout_all` deletes every family of a user; neither recalls an access token already issued.
- An injected `Clock` lets a seven-day expiry be tested in microseconds.

### What comes back later

- **Putting this store in a table instead of a `HashMap`** — [3.5 — PostgreSQL & `sqlx`](../../05-postgres-and-sqlx/README.md)
- **Checking the access token on every request in a middleware** — [3.7.3 — JWTs and `tower` middleware](../03-jwt-and-tower-middleware/README.md)
- **Deciding what an authenticated user is allowed to do** — [3.7.5 — Modelling RBAC and permissions](../05-modelling-rbac-and-permissions/README.md)
- **Returning refresh failures as one consistent error shape** — [3.8.1 — Consistent error envelopes](../../08-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.md)

### Can you explain?

- Why can a server not revoke an access token, but can revoke a refresh token?
- Why must used refresh tokens stay in the store instead of being deleted?
- Why does a replayed token revoke the whole family, including the newest token?
- Why is SHA-256 enough for refresh tokens when passwords need a slow hash?
- Why does `rotate` check "already used" before "expired"?

---

## Going further

- [OWASP — Session Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html): expiry, renewal and invalidation of session identifiers.
- [OAuth 2.0 Security Best Current Practice (RFC 9700)](https://www.rfc-editor.org/rfc/rfc9700.html): the section on refresh-token protection describes rotation and replay detection for public clients.
- [`sha2` on docs.rs](https://docs.rs/sha2/0.10/sha2/): the `Digest` trait you use inside `hash_token`.

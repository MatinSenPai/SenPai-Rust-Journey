# Solution — 3.7.2 Sessions vs. JWT: the real trade-off

The full code is `solution/src/lib.rs`; it passes every test in `solution/tests/` (45 for the Implement rung, plus `build_test.rs` for Build and Challenge). `solution/src/main.rs` serves the app on `127.0.0.1:3180`.

## `SessionStore`

```rust
pub fn create(&self, user: &str) -> String {
    let id = new_session_id();
    let expires_at = self.clock.now().saturating_add(self.ttl_secs);
    let session = Session { user: user.to_string(), expires_at };
    self.sessions.lock().unwrap().insert(id.clone(), session);
    id
}
```

The expiry is computed once, at creation, from the injected clock. `saturating_add` means a huge lifetime cannot overflow. Every call mints a new id, so one user with two devices has two rows.

```rust
pub fn lookup(&self, id: &str) -> Option<String> {
    let now = self.clock.now();
    let mut sessions = self.sessions.lock().unwrap();
    match sessions.get(id) {
        Some(s) if now < s.expires_at => Some(s.user.clone()),
        Some(_) => { sessions.remove(id); None }
        None => None,
    }
}
```

The guard is held across the `match` on purpose: the check and the removal must happen under one lock, or two lookups could race. `now < expires_at` makes the boundary exact (`a_session_is_live_until_the_instant_it_expires`). Removing the expired entry here is the only cleanup the lesson has.

`revoke` removes the entry first and then reports `matches!(removed, Some(s) if now < s.expires_at)`, so an expired session is deleted but answers `false`. `revoke_all_for` does the same inside `retain`, counting the live ones it drops.

## `set_cookie` and friends

```rust
let mut out = format!("{name}={value}; Path=/; HttpOnly");
if opts.secure { out.push_str("; Secure"); }
out.push_str(match opts.same_site { /* Strict, Lax, None */ });
if let Some(n) = opts.max_age { out.push_str(&format!("; Max-Age={n}")); }
```

Validation comes first: `bad_char` rejects anything outside `!`..=`~` plus `; , " \`, which is what stops a value like `a; Domain=evil.example` or a CRLF from smuggling in an attribute or a second header. Only after that does the "`SameSite=None` needs `Secure`" rule run, which is why `invalid_input_is_reported_before_the_samesite_rule`. `Some(0)` is written out because `if let Some(n)` does not treat zero as "absent". `expire_cookie` copies `opts` with `max_age: Some(0)` and calls `set_cookie` with an empty value, so it inherits every rule.

`session_id_from_cookie_header` splits on `;`, trims, `split_once('=')` (the first `=` only, so `sid=a=b` keeps `a=b`), takes the first exact-name match, and filters out an empty value. `origin_ok` is `None => true, Some(o) => o == expected`: exact string equality, so a trailing slash and `"null"` both fail.

## Build: `touch`

`touch` looks the entry up with `get_mut`, and only when `now < expires_at` sets `expires_at = now + ttl`. An expired entry is left alone and the answer is `false`: a session that already died must not come back. To wire it in, call `touch` in `me` after a successful `lookup` and re-send the cookie built from `set_cookie(.., Some(ttl))` so the browser's `Max-Age` moves with the server's expiry. Without the new `Set-Cookie`, the browser would drop the cookie at the original time even though the server would still accept it.

## Challenge: `revoke_others`

`retain` again, keeping an entry when it belongs to another user or its id equals `keep`, and counting the live ones it drops. The route reads the caller's session, looks up the user, calls `revoke_others(user, id)`, and answers `204`. It must apply `origin_allowed` like the other state-changing routes, or a forged cross-site `POST` could log you out everywhere else.

## On the warm-ups and the errors

- **Example 04 (`E0004`).** Add `SameSite::None => "SameSite=None"`. Do not use `_ =>` or the suggested `todo!()`.
- **Example 05 (`E0277`).** Replace `Cell<u64>` with `AtomicU64` (`get` becomes `load(Ordering::SeqCst)`, `set` becomes `store`).
- **Example 03 (fixation).** `login` should generate a new id, remove the incoming id from the table, insert the new id as logged in, and return the new id. The test that matters: the planted id is no longer present afterwards.

## What this lesson was really about

None of the code is long. What matters is what each piece can *do*: a server-side table can end a login by deleting a row, and a stateless token cannot. The rest (cookie flags, the `Origin` check, the fresh id at login) is what it costs to run the stateful design safely, and it is small.

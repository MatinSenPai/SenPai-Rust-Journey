# Solution

```rust
pub fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("hashing an in-memory password should never fail")
        .to_string()
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    let Ok(parsed_hash) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}
```

## Why `.expect(...)` in `hash_password` but a graceful `false` in `verify_password`

These two functions treat failure completely differently on purpose, and
the difference is about *what kind of failure is even possible* at each
call site. `hash_password` takes a `&str` — already valid UTF-8, already in
memory — and hands its bytes to `Argon2::default().hash_password(...)`.
There's no genuinely recoverable failure mode here (the crate's own docs
describe the realistic failure cases as things like "output buffer too
small," which can't happen with `Argon2::default()`'s parameters against an
ordinary string), so `.expect(...)` documents that assumption directly in
the code rather than pretending this function might meaningfully fail and
forcing every caller to handle a `Result` that in practice never returns
`Err`. `verify_password`, by contrast, takes `hash: &str` from *outside*
this function's control — a database column, a network payload, wherever
you stored it — and `PasswordHash::new` genuinely can fail on that input if
it's been truncated, corrupted, or simply isn't a PHC string at all. That's
not a programmer error the way a bad `hash_password` call would be; it's an
ordinary "this untrusted input was bad" case, so it gets a real `let-else`
that turns "unparsable" into `false` instead of a panic. Same discipline as
Phase 1-2's "make illegal states unrepresentable, make failure explicit" —
just applied to *which* failures are worth propagating as values versus
which ones represent a genuine "this should never happen" invariant.

## Why `bool`, not `Result<bool, Error>`

A caller checking a login has exactly one decision to make: let the request
through, or reject it. Whether the rejection came from "wrong password" or
"the stored hash is somehow corrupt" changes nothing about what the caller
does next — both mean 401. Collapsing both into one `false` (rather than a
`Result` the caller would have to `match` on, or worse, `.unwrap()` and
crash the whole request on a bad row) is the same principle as
`taskforge-api`'s `require_bearer_token` treating "no header" and "wrong
token" identically as one `Err(ApiError { status: UNAUTHORIZED, .. })` — an
auth boundary should present the *outside world* with one uniform
"authorized or not," even when its internals distinguish several different
reasons a check could fail.

## Salts, concretely

`SaltString::generate(&mut OsRng)` pulls fresh randomness from the
operating system's cryptographically secure RNG every single call — that's
the entire mechanism behind
`hashing_the_same_password_twice_produces_different_hashes` passing. Once
that salt is baked into the PHC string `hash_password` returns, verifying
never needs the caller to supply the salt separately: `PasswordHash::new`
parses it back out of the stored string, and `Argon2::default()
.verify_password(...)` uses *that* embedded salt (not a fresh one) to
re-derive a hash and compare. If verification instead generated a new
random salt every time, it would never match anything — the salt has to be
the *same* one the password was originally hashed with, which is exactly
why it travels inside the stored hash instead of living in a separate
column.

## On the checkpoint questions

**Q1 (hash, don't encrypt):** You never need the original password back —
login only ever needs to answer "does this input match what was stored,"
never "what was the original value." Encryption exists for data you
genuinely need to recover later (a credit card number you'll charge), which
requires keeping a decryption key somewhere — a key that itself becomes a
single point of catastrophic failure if it leaks. A one-way hash has no
key to steal that would recover every password at once; even a full
database leak only exposes hashes, not passwords, and cracking each one
individually is exactly what Argon2's cost is designed to make expensive.

**Q2 (why salting breaks rainbow tables):** A rainbow table is a
precomputed `hash -> password` lookup built once, ahead of time, against
*unsalted* (or fixed-salt) hashes — its entire value is that the expensive
precomputation happens exactly once and then gets reused against any
target sharing that same hash function and salt. A random, unique
per-password salt means `hash(password + salt)` is different for every
single row, so a precomputed table would need to be rebuilt from scratch
*per salt* — which means per row — which costs exactly as much computation
as just brute-forcing that one password directly. The "precompute once,
reuse forever" economics that make rainbow tables worthwhile collapse
entirely.

**Q3 (why nondeterminism is correct):** If `hash_password` were
deterministic, two users with the same password would have byte-identical
rows in your database — visibly leaking "these two accounts share a
password" to anyone with read access, and reintroducing exactly the
rainbow-table vulnerability salting exists to close (a deterministic hash
is just an unsalted hash with extra steps). Different output every call, for
the same input, is the salt doing its job.

**Q4 (what memory-hardness denies an attacker):** Custom cracking hardware
(GPUs, and especially purpose-built ASICs/FPGAs) is economical specifically
*because* it can run enormous numbers of parallel hash attempts cheaply —
but that parallelism assumes each attempt is cheap to run side-by-side.
Memory is the resource that doesn't parallelize cheaply: RAM is
comparatively expensive and can't be duplicated as freely as raw compute
cores. Forcing every single Argon2 attempt to allocate real memory (tens of
MB by default) means an attacker's "run a billion attempts in parallel"
hardware would need a billion times that memory footprint too — collapsing
the economic advantage that made building the hardware worthwhile in the
first place. bcrypt, being CPU-slow but not memory-hard, doesn't impose
that same constraint, which is exactly why GPU/ASIC bcrypt crackers are
viable in a way GPU/ASIC Argon2 crackers are far less so.

**Q5 (what a panic on malformed input would cost you):** If a malformed
`password_hash` value in your database caused `verify_password` to panic
instead of returning `false`, then a single corrupted row (a botched
migration, a truncated column, a bit flip) would crash whatever request
tried to authenticate that user — turning a "this one user can't log in"
problem into "this endpoint panics," which in many server setups takes down
or restarts the handling task/worker. Worse, if any part of the failure
were even slightly attacker-influenced, a panic-on-bad-input path is a
denial-of-service lever: an attacker who can get a malformed hash into your
system (or even just probe with crafted `Authorization` values, depending
on how the value flows) could crash your auth path on demand.

**Q6 (why cost parameters travel with the hash):** Argon2's memory/time/
parallelism costs need to keep rising as hardware gets faster — a setting
that's "expensive enough" today won't be in five years. If those parameters
lived only in your application code as a fixed constant, raising them would
require a data migration: re-hash every stored password with the new
settings, which you can't do without the plaintext (which, correctly, you
never kept). Because the PHC string embeds its *own* parameters, old hashes
keep verifying correctly forever under whatever settings they were created
with, while `hash_password` naturally uses your *current*
`Argon2::default()` settings for every new hash — you upgrade incrementally,
one user at a time, the next time each of them successfully logs in and you
choose to re-hash their password with current settings, never all at once
under time pressure.

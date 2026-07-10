# 06.1 — Password hashing with `argon2`

## Never store a plaintext password

If your `users` table has a `password` column with the literal password in
it, every leak of that table is a leak of every user's real, reusable
password — for your app *and* for every other site they reused it on
(they will have). This isn't a hypothetical: it's the single most common
root cause behind "we got breached" postmortems. The fix isn't "encrypt it
so we can decrypt it later" — you never need the plaintext back, so there's
no reason to keep it recoverable at all. The fix is a **one-way hash**:
store something you can *check a password against*, but can't turn back
into the password.

You already know this instinct from Django — `User.objects.create_user(...)`
never stores what you typed, and `check_password()` is how login actually
verifies it. This lesson builds the same thing by hand, once, so "hashed,
not encrypted" stops being something you take on faith.

## Why hashing alone isn't enough: rainbow tables

A plain hash (`sha256("hunter2")`) is *deterministic* — the same input
always produces the same output. That determinism is exactly what an
attacker exploits with a **rainbow table**: a precomputed table mapping
common passwords (and every password from every previous leaked database,
which is a lot of passwords) to their hash. If your database leaks and your
`password_hash` column is just `sha256(password)`, the attacker doesn't
crack your hash — they look it up in a table they already built once,
covering millions of sites at once.

## The salt

A **salt** is random data generated fresh *per password* and mixed into the
hash. `hash(password + salt)` means two users with the same password get
completely different stored hashes (see the
`hashing_the_same_password_twice_produces_different_hashes` test below —
even the *same* password hashed twice produces two different results,
because a fresh salt is generated every time). A rainbow table is
precomputed for *unsalted* hashes; a per-user, per-hash random salt means
an attacker would need a fresh rainbow table for every single row, which is
computationally equivalent to just brute-forcing each password
individually — the entire shortcut rainbow tables provide disappears. The
salt itself doesn't need to be secret; it just needs to be random and
unique per hash, which is why it's stored right alongside the hash (you'll
see it embedded directly in the encoded string below, not hidden anywhere).

## Why Argon2, not bcrypt or plain SHA-256

- **SHA-256 (or any general-purpose hash) is disqualified outright.** It's
  designed to be *fast* — that's a feature for checksums and content
  addressing, and a liability for passwords: fast means an attacker with a
  GPU can try billions of guesses per second against a leaked hash.
- **bcrypt** fixed the speed problem decades ago by being deliberately slow
  (tunable via a cost/work factor), and was a huge improvement over naive
  hashing. Its weakness today: it's *only* CPU-hard, not memory-hard.
  Modern attackers don't brute-force passwords on CPUs — they build custom
  hardware (GPUs, FPGAs, ASICs) that's cheap to parallelize across as long
  as the algorithm needs little memory per attempt. bcrypt needs only a
  small, fixed amount of memory, so it parallelizes beautifully on that
  kind of hardware.
- **Argon2 is memory-hard by design** — its cost function requires a
  configurable, genuinely large amount of memory (tens of MB by default)
  *per hash attempt*, not just CPU cycles. Custom cracking hardware is
  built to be cheap and highly parallel, which works great when each
  attempt needs almost no memory — but memory is expensive to parallelize
  at scale. Forcing every guess to allocate real memory closes exactly the
  loophole that makes GPU/ASIC cracking so effective against bcrypt.
  Argon2 won the 2015 Password Hashing Competition specifically for this
  property, and Argon2id (the default variant used by the `argon2` crate)
  is the current OWASP-recommended choice for new applications.
- **Django's own trajectory backs this up**: Django's default password
  hasher today is PBKDF2 (iteration-based, similar spirit to bcrypt — slow,
  but not memory-hard), but Django ships first-class support for Argon2
  (`pip install django[argon2]`) and its own documentation recommends
  switching to it — "Argon2 is the winner of the 2015 Password Hashing
  Competition... and is not vulnerable to the same kinds of attacks as
  PBKDF2." This lesson is Rust catching you up to where Django's own docs
  already point.

## Reading an encoded Argon2 hash

`hash_password` returns a string that looks like:

```
$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$RdescudvJCsgt3ub+b+dWRWJTmaaJObG
```

This is the **PHC string format**, and it's genuinely all you need to store
(one `TEXT` column) — everything required to verify a password later
travels with the hash itself:

- `argon2id` — the algorithm variant.
- `v=19` — the Argon2 version.
- `m=19456,t=2,p=1` — the cost parameters: memory (KiB), time
  (iterations), and parallelism (threads/lanes) used to produce this hash.
- The next segment is the salt (base64), and the last segment is the hash
  output itself (base64).

Because the parameters travel with the hash, you can raise your cost
settings in the future (hardware gets faster; yesterday's "expensive enough"
becomes tomorrow's "crackable") and old hashes made with the old, cheaper
settings still verify correctly — `PasswordHash::new` reads whatever
parameters are embedded in that specific hash, not whatever your current
`Argon2::default()` would produce. You only pay the cost of *upgrading* a
user's stored hash to the new settings the next time they log in
successfully (out of scope for this lesson, but worth knowing the door is
open).

## Walking the starter code

- `hash_password(password: &str) -> String` — generates a random salt with
  `SaltString::generate(&mut OsRng)`, then calls
  `Argon2::default().hash_password(...)`, which returns a `PasswordHash`
  you turn into the encoded string with `.to_string()`.
- `verify_password(password: &str, hash: &str) -> bool` — parses the stored
  hash string back into a `PasswordHash` with `PasswordHash::new(hash)`,
  then calls `Argon2::default().verify_password(...)`, which re-derives a
  hash using the *embedded* salt and parameters and compares it to the
  stored one. Note the return type is a plain `bool`, not a `Result` — a
  wrong password and a corrupted/malformed hash string both just mean
  "reject this login," and a caller checking credentials never needs to
  tell those two failure modes apart.
- Both functions come from `argon2::password_hash` (re-exported from the
  `password-hash` crate) — `PasswordHash`, `PasswordHasher`,
  `PasswordVerifier`, and `SaltString` are all traits/types from there;
  `Argon2` itself (the algorithm implementation) is what implements
  `PasswordHasher`/`PasswordVerifier`.

## Your task

Open `src/lib.rs`. Implement the two `todo!()`-gated functions:

- `hash_password` — generate a salt, hash, return the encoded string.
- `verify_password` — parse the stored hash, verify, return a `bool`
  (never panic on a malformed hash — that path is exercised directly by
  `verifying_against_a_malformed_hash_fails_instead_of_panicking`).

No database, no server, no `#[ignore]`d tests here — this is pure
computation. `cargo test -p p3-06-01-password-hashing-argon2` should just
work once both `todo!()`s are filled in.

## Checkpoint

`cargo test -p p3-06-01-password-hashing-argon2`, then `CHECKPOINT.md`,
then `solution/SOLUTION.md`.

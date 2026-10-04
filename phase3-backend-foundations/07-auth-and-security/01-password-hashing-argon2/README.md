# 3.7.1 — Password hashing with `argon2`

## At a glance

After this lesson you can:

- Explain why a password is hashed and not encrypted, what a salt adds, and why `Argon2id` beats `SHA-256` and `bcrypt` for this one job.
- Read an encoded Argon2 hash (a PHC string) field by field, and decide whether a stored hash is weaker than your current settings.
- Write a login check that answers "unknown user" and "wrong password" identically, in about the same time, and compares secrets without an early exit.

**Time:** ~75 minutes · **Prerequisites:**
[2.1.2 — `HashMap` in depth](../../../phase2-intermediate/01-collections/02-hashmap-in-depth/README.md),
[3.4.1 — 12-factor config and secrets](../../04-configuration-and-app-structure/01-config-and-secrets/README.md)

---

## Why this matters

Every database leaks eventually: a forgotten backup, a SQL injection, a laid-off admin. The question is what the attacker holds afterwards. If your `users` table stores the passwords, they hold every user's real, reusable password, for your app and for every other site those users reused it on. That is why the OWASP Top 10 has a category for it: Cryptographic Failures (A02 in the 2021 edition), which covers passwords stored in the clear or with a fast, unsalted hash.

You have used the safe version without writing it. Django's `make_password("hunter2")` returns a string like `argon2$argon2id$v=19$...`, `User.objects.create_user(...)` stores only that string, and `check_password("hunter2", encoded)` is how login verifies it. Django also re-hashes a password on a successful login when the stored hash uses an old algorithm or old settings. This lesson builds those pieces by hand, in a `UserStore` that holds users in memory. Nothing here needs a database or a server, so every test runs in milliseconds. When you reach the PostgreSQL module, [module 5](../../05-postgres-and-sqlx/README.md), the `users` table will hold one `TEXT` column with exactly the string you produce today.

---

## The concept

### Hash, never encrypt

You never need the original password back. Login only asks "does what was typed match what was stored?" Encryption is for data you must read again, such as a card number you will charge, and it needs a key. That key is one more thing to steal, and whoever steals it reads every password at once. A **one-way hash** has no key and no way back. All the attacker can do is guess a password, hash the guess, and compare.

The rest of the lesson is about making each guess as expensive as possible.

```senpai-visual
{"kind":"concept","labels":["register: password goes in once","salt + Argon2id: slow, memory-hungry hash","store only the PHC string","login: re-hash the typed password with the stored salt and compare"]}
```

### Rainbow tables and the salt

A plain hash is deterministic: `sha256("hunter2")` is the same on every machine, forever. So an attacker can precompute a table of `hash -> password` for millions of common passwords once (a **rainbow table**, in the broad sense of any precomputed lookup) and reuse it against every leaked database. Your leak costs them a table lookup, not a cracking job.

A **salt** is random data, fresh for each password, mixed into the hash. `examples/01-hash-and-verify.rs` hashes the same password twice:

```text
first : $argon2id$v=19$m=19456,t=2,p=1$StrpfuYFfUCnPJmCaN1u8g$xl7Dv7aJqja+rp26ugAItu08nxRhC/P1ZS80YH3Fl7M
second: $argon2id$v=19$m=19456,t=2,p=1$fuye9QoksAjIGHYCjr9Fkg$rdrENzbLC7spjBvpGHyYIlUDyKpwAn5WyoBkusFXggk
equal strings? false
right password: Ok(())
wrong password: Err(Password)
```

(Your salts and hashes will differ; that is the point.) The same input gives two different strings, and both still verify. Two users with the same password get different rows, so a precomputed table is useless: the attacker would need a new one per salt, which costs as much as cracking each row directly. The salt is not secret. It sits in the stored string, and its only job is to be unique and unpredictable.

### Why Argon2id, not `SHA-256` or `bcrypt`

- **`SHA-256`** is built to be fast, which is right for checksums and wrong for passwords. A GPU tries billions of guesses a second against a fast hash.
- **`bcrypt`** is deliberately slow, with a tunable cost, and was a big step forward. But it needs only a small, fixed amount of memory per guess, so cracking hardware (GPUs, FPGAs, ASICs) can run huge numbers of guesses side by side cheaply.
- **`Argon2id`** is **memory-hard**: each guess must hold a configurable amount of RAM while it runs. RAM is the resource that does not parallelise cheaply, so every guess an attacker runs in parallel needs its own block of it. Argon2 won the Password Hashing Competition in 2015, and the `Argon2id` variant is the one OWASP's Password Storage Cheat Sheet recommends first. This lesson does not quote that sheet's numbers, because they get revised; read it for the current ones.

Django's default hasher is still PBKDF2, but `PASSWORD_HASHERS` accepts `Argon2PasswordHasher` (with `pip install django[argon2]`), and Django's docs recommend it first. Where the analogy stops: Django picks the algorithm and parameters for you from settings. In Rust you pass them yourself, which is why `hash_password` in this lesson takes the cost as an argument.

```senpai-visual
{"kind":"concept","labels":["SHA-256: fast, so billions of guesses per second","bcrypt: slow, but little memory per guess","Argon2id: slow and memory-hard","cost per guess: low, medium, high"]}
```

### Reading a PHC string

The string `hash_password` returns is the **PHC string format**. `examples/02-read-the-phc-string.rs` takes one apart:

```text
algorithm  : argon2id
version    : Some(19)
memory KiB : 19456
iterations : 2
lanes      : 1
salt       : c29tZXNhbHQ
hash bytes : 24
```

The input was `$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$RdescudvJCsgt3ub+b+dWRWJTmaaJObG`, five `$`-separated fields:

- `argon2id` is the variant, and `v=19` is the Argon2 version (`0x13`).
- `m=19456,t=2,p=1` is the cost: memory in KiB, passes over memory, and lanes (parallelism). The `argon2` 0.5 crate's own default is exactly these three numbers.
- The salt and the hash itself, both base64 without padding. `c29tZXNhbHQ` decodes to the bytes `somesalt`.

Everything needed to verify the password travels inside the string, so one `TEXT` column is enough. It also means you can raise your cost settings next year: old hashes keep verifying under the settings written in them, and new hashes use the new ones. `needs_rehash` in this lesson is the question "is this stored hash weaker than what I would make today?", and a successful login is the moment you can answer it, because that is the only time you hold the plaintext.

### Cost is the whole point, and tests must not pay it

`examples/03-cost-and-parameters.rs` hashes one password under three settings:

```text
m=8      t=1  took    1 ms  $argon2id$v=19$m=8,t=1,p=1$IoUnv
m=19456  t=2  took  439 ms  $argon2id$v=19$m=19456,t=2,p=1$l
m=65536  t=3  took 2238 ms  $argon2id$v=19$m=65536,t=3,p=1$h
cheap hash, default hasher: Ok(())
```

These are debug-build timings on one laptop and will differ on yours, but the shape holds: the cost grows with memory and passes, and a hash made with `m=8` still verifies through a hasher built with different defaults, because it reads its parameters from the string.

That slowness is a feature at login and a nuisance in a test suite. So every function in this lesson takes the cost as a `Params` argument, and every test passes `cheap_params()`: 8 KiB of memory, one pass, one lane. That is the smallest memory `Argon2` accepts for one lane (8 times the number of lanes). The 18 tests finish in about a tenth of a second. Never use those settings for real users; they exist so tests can stay fast.

### One answer for "no such user" and "wrong password"

If login says "no such user" for one and "wrong password" for the other, an attacker can test which usernames exist on your site without knowing a single password. Return the same error for both. That is not enough, though. `examples/04-unknown-user-timing.rs` times three paths:

```text
known user, wrong password :  432 ms
unknown user, early return :    0 ms
unknown user, dummy verify :  437 ms
```

A response that returns instantly for unknown users and after 400 ms for known ones tells the attacker the same thing, only through a clock. The fix is to spend the same work on both paths. For an unknown user, verify the typed password against a **dummy hash**, built once when the store starts, and throw the result away. Django does the equivalent: when `ModelBackend` finds no user, it runs the password hasher on the input anyway before returning.

```senpai-visual
{"kind":"result","labels":["login(username, password)","user found: verify against its hash","user not found: verify against the dummy hash","both paths: Err(InvalidCredentials), similar time"]}
```

### Comparing secrets without an early exit

Ordinary `==` on two byte slices stops at the first difference. If the first byte differs it returns right away, and if the first thousand match it takes longer. Measured precisely enough, the time leaks how much of a guess was right. A **constant-time comparison** looks at every byte whatever the data is: XOR each pair, OR the results together, and test the total once at the end.

You do not need to call it for passwords: the `password-hash` crate that `argon2` builds on compares the hash outputs in constant time (it implements `subtle`'s `ConstantTimeEq`) inside `verify_password`. You will write `constant_time_eq` anyway, because the same idea matters for any secret you compare yourself, such as an API key or a token. In real code you would reach for the `subtle` crate rather than your own loop.

---

## Hands on

```sh
cargo run -p p3-07-01-password-hashing-argon2 --example 01-hash-and-verify
cargo run -p p3-07-01-password-hashing-argon2 --example 02-read-the-phc-string
cargo run -p p3-07-01-password-hashing-argon2 --example 03-cost-and-parameters
cargo run -p p3-07-01-password-hashing-argon2 --example 04-unknown-user-timing
```

Then the three broken ones. Two need the `broken` feature; the third runs normally and is simply wrong:

```sh
cargo build -p p3-07-01-password-hashing-argon2 --example 05-missing-trait-import-broken --features broken
cargo run -p p3-07-01-password-hashing-argon2 --example 06-unwrap-malformed-hash-broken --features broken
cargo run -p p3-07-01-password-hashing-argon2 --example 07-compare-hashes-trap
```

The five functions and the store in `src/lib.rs` start as `todo!()`. Run the tests before you write anything, and watch them fail for the right reason:

```sh
cargo test -p p3-07-01-password-hashing-argon2 --test hashing hash_then_verify
```

```text
running 1 test
test hash_then_verify_round_trips ... FAILED

failures:

---- hash_then_verify_round_trips stdout ----

thread 'hash_then_verify_round_trips' (29316) panicked at phase3-backend-foundations\07-auth-and-security\01-password-hashing-argon2\src\lib.rs:22:5:
not yet implemented: return the encoded Argon2id PHC string for this password, using a fresh random salt and the given cost settings
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    hash_then_verify_round_trips

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p p3-07-01-password-hashing-argon2 --test hashing`
```

(The number in parentheses is the thread's id and changes every run. Before `running 1 test`, the compiler also prints "unused variable" warnings for each `todo!()` body; they go away as you implement.)

Then try these:

1. In `01-hash-and-verify`, hash with `Params::new(8, 1, 1, None)` instead of the default. What does the start of the string say now?
2. In `02-read-the-phc-string`, change one character of the salt. Does `PasswordHash::new` still accept it? Does the hash still "belong" to it?
3. In `04-unknown-user-timing`, make the dummy verify use `m=8`. Do the two paths still take the same time? What would an attacker see?

---

## Errors you will meet

### `E0599` — a trait method that is not in scope

```text
error[E0599]: no method named `hash_password` found for struct `Argon2<'key>` in the current scope
   --> phase3-backend-foundations\07-auth-and-security\01-password-hashing-argon2\examples\05-missing-trait-import-broken.rs:10:34
    |
 10 |     let hash = Argon2::default().hash_password(b"hunter2", &salt).unwrap();
    |                                  ^^^^^^^^^^^^^
    |
   ::: C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\password-hash-0.5.0\src\traits.rs:33:8
    |
 33 |     fn hash_password<'a>(
    |        ------------- the method is available for `Argon2<'_>` here
    |
    = help: items from traits can only be used if the trait is in scope
help: there is a method `hash_password_into` with a similar name, but with different arguments
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\argon2-0.5.3\src\lib.rs:229:5
    |
229 |     pub fn hash_password_into(&self, pwd: &[u8], salt: &[u8], out: &mut [u8]) -> Result<()> {
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
help: trait `PasswordHasher` which provides `hash_password` is implemented but not in scope; perhaps you want to import it
    |
  5 + use argon2::PasswordHasher;
    |

For more information about this error, try `rustc --explain E0599`.
error: could not compile `p3-07-01-password-hashing-argon2` (example "05-missing-trait-import-broken") due to 1 previous error
```

(The `todo!()` warnings from `src/lib.rs` print before this and are left out. The `.cargo\registry` paths and hash suffixes depend on your machine.)

**What the compiler is objecting to:** `hash_password` is not a method of `Argon2` itself; it comes from the `PasswordHasher` trait. A trait's methods only exist on a type while the trait is in scope. The error says that outright (`items from traits can only be used if the trait is in scope`). Its first `help:` is a distraction: `hash_password_into` is a different, lower-level method with a different signature.

**The fix:** follow the second `help:` and import the trait.

```rust
use argon2::password_hash::{rand_core::OsRng, PasswordHasher, SaltString};
```

**Why this is the fix:** `PasswordHasher` and `PasswordVerifier` are the two traits you need for hashing and for verifying, and neither is in the prelude. Add both and the crate's methods appear.

### A run-time panic: unwrapping a stored hash that does not parse

```text
thread 'main' (24420) panicked at phase3-backend-foundations\07-auth-and-security\01-password-hashing-argon2\examples\06-unwrap-malformed-hash-broken.rs:8:51:
called `Result::unwrap()` on an `Err` value: PhcStringField
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

(The thread id changes every run.)

**What's actually broken:** `PasswordHash::new(from_database).unwrap()`. The stored value came from a database column that someone migrated badly, truncated, or filled with a legacy hash from another system. That is data from outside your function, and you let it crash the request. One bad row now takes down a whole login path, and an attacker who can influence that value controls a crash.

**The fix:** decide what a bad stored hash means. For login, it means "reject":

```rust
let Ok(parsed) = PasswordHash::new(from_database) else {
    println!("stored hash is not a PHC string: reject the login");
    return;
};
```

**Why this is the fix:** the `let ... else` turns the error into a value you chose. That is also why your `verify_password` returns `bool` and never panics: the caller's decision is "let in" or "reject", and a corrupt row and a wrong password both mean "reject".

### No error at all: logging in by hashing again

```text
correct password accepted? false
```

**What's actually broken:** `examples/07-compare-hashes-trap.rs` hashes the typed password again and compares the two strings with `==`. It compiles and runs, and it rejects the right password every time. A fresh salt makes the new string different from the stored one, however correct the password is. The same mistake in a rush "fix" is worse: someone notices logins fail, removes the random salt to make strings match, and ships the rainbow-table weakness.

**The fix:** never compare hashes yourself. Parse the stored string and ask the library to re-derive and compare with the salt that is inside it:

```rust
let parsed = PasswordHash::new(&stored).expect("stored hash");
let login_ok = Argon2::default().verify_password(b"hunter2", &parsed).is_ok();
```

**Why this is the fix:** the salt must be the one the password was originally hashed with, which is why it lives inside the stored string. `verify_password` reads it from there, uses the parameters in the string, and compares in constant time. No compiler catches this bug; a test that registers a user and then logs in as that user does.

---

## Exercises

### Warm up

<details>
<summary>Two users register with the same password. Are their stored hashes equal?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

No. Each hash gets its own random salt, so the strings differ. Equal stored hashes would also tell anyone who can read the table that two accounts share a password.

</details>

<details>
<summary>A hash was made with <code>m=8,t=1,p=1</code>. Your server's current settings are much stronger. Will <code>verify_password</code> still accept the right password?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

Yes. The cost settings are inside the string and verification uses those, not the current ones. That is also why `needs_rehash` exists: the old hash keeps working, so something has to notice it is weak.

</details>

<details>
<summary>Login answers an unknown username in 0 ms and a known one in 430 ms, with the same error text. What leaks?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

Which usernames exist. The error text is identical but the timing is not, so an attacker reads the clock instead of the message. The fix is to verify against a dummy hash when the user is unknown.

</details>

<details>
<summary>Why does comparing two secrets with <code>==</code> leak information, even though the answer is only "equal" or "not equal"?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

`==` can stop at the first differing byte, so the running time depends on how many leading bytes matched. Measured many times, that tells an attacker how close a guess is. A constant-time comparison always looks at every byte.

</details>

### Repair

Fix all three broken examples:

1. `examples/05-missing-trait-import-broken.rs` builds and prints a PHC string.
2. `examples/06-unwrap-malformed-hash-broken.rs` prints a rejection message instead of panicking.
3. `examples/07-compare-hashes-trap.rs` prints `true` for the correct password.

### Implement

Five functions in `src/lib.rs`:

```sh
cargo test -p p3-07-01-password-hashing-argon2 --test hashing
```

Each one is fully specified in its doc comment, so you should not need to read the tests to know what to build:

- `hash_password(password, params)`: an Argon2id PHC string with a fresh random salt each call.
- `verify_password(password, phc)`: `true` only for a match, and `false` (never a panic) for a wrong password or a stored value that is not a PHC string.
- `describe_hash(phc)`: read a PHC string into a `HashInfo`, or `None` when it is not a complete, valid Argon2 hash.
- `needs_rehash(phc, target)`: is this stored hash a different algorithm from, or weaker than, the target settings?
- `constant_time_eq(a, b)`: byte equality that never exits early.

All of them use `cheap_params()` in the tests, so the 12 tests in `tests/hashing.rs` finish in a fraction of a second.

### Build

`UserStore` in the same file: an in-memory user table that never holds a password.

```sh
cargo test -p p3-07-01-password-hashing-argon2 --test store
```

- `new(params)` builds an empty store and a dummy hash ready for login.
- `register(username, password)` rejects a short password (fewer than 8 characters, counting `char`s and not bytes) with `PasswordTooShort` before it checks the username, then a taken name with `UsernameTaken`, and otherwise stores only the hash.
- `login(username, password)` returns `Ok(())` only for a known user with the right password. An unknown user and a wrong password both return `InvalidCredentials`, and an unknown user still pays for one full verification against the dummy hash.

The six tests in `tests/store.rs` check this, including that the stored value starts with `$argon2id$` and never contains the password.

### Challenge (optional)

Make a successful `login` upgrade the stored hash when `needs_rehash` says it is weaker than the store's current params, using the plaintext you hold at that moment. A failed login must change nothing. Add your own test to `tests/store.rs`: register under `cheap_params()`, call `set_params` with stronger settings, log in once, and read the cost back with `describe_hash`.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Password hash | a one-way, deliberately slow digest of a password | the one column your `users` table keeps |
| Salt | random data, unique per hash, stored inside the hash string | defeating precomputed tables |
| Rainbow table | a precomputed lookup from hash to password | why unsalted or fast hashes fail |
| Memory-hard | each guess must hold real RAM while it runs | why `Argon2id` resists GPU and ASIC cracking |
| PHC string | `$alg$v=..$params$salt$hash`, everything needed to verify | storing and reading hashes |
| Cost parameters | `m` (memory), `t` (passes), `p` (lanes) | tuning, and `needs_rehash` |
| Dummy hash | a throwaway hash verified when the user does not exist | login that takes the same time either way |
| Constant-time comparison | comparing without an early exit | any secret you compare yourself |

### What you now know

- Passwords are hashed, never encrypted and never stored in the clear, and OWASP files storing them badly under Cryptographic Failures.
- A fresh salt per hash makes equal passwords look different and makes precomputed tables worthless.
- `Argon2id` is memory-hard, `bcrypt` is only slow, and `SHA-256` is fast, which is the wrong property here.
- A PHC string carries its own algorithm, version, cost, salt and hash, so costs can rise later and old hashes still verify.
- Login must give the same error and take about the same time for an unknown user as for a wrong password.
- Tests pass the cost in as `cheap_params()` so a real Argon2 run does not make the suite slow.

### What comes back later

- **Where the `users` table and its `TEXT` hash column actually live** — [3.5 — PostgreSQL and `sqlx`](../../05-postgres-and-sqlx/README.md)
- **Reading the cost settings and secrets from configuration** — [3.4.1 — 12-factor config and secrets](../../04-configuration-and-app-structure/01-config-and-secrets/README.md)
- **What a logged-in user holds after the password check, stateful or stateless** — [3.7.2 — Sessions vs. JWT: the real trade-off](../02-sessions-vs-jwt/README.md)
- **Issuing and checking signed tokens in middleware** — [3.7.3 — JWTs and `tower` middleware](../03-jwt-and-tower-middleware/README.md)
- **Turning `InvalidCredentials` into one consistent HTTP error** — [3.8.1 — Consistent error envelopes](../../08-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.md)

### Can you explain?

- Why is a password hashed and not encrypted, and what does the attacker hold after a leak in each design?
- What does a salt change, and why does it not need to be secret?
- Why is `SHA-256` the wrong hash for passwords, and what does `Argon2id` add over `bcrypt`?
- What does each field of `$argon2id$v=19$m=19456,t=2,p=1$salt$hash` tell a verifier?
- Why must login do real hashing work even when the username does not exist?
- Why does `==` on secrets leak, and how does `constant_time_eq` avoid it?

---

## Going further

- [OWASP Password Storage Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html): the current recommended Argon2id settings and the reasoning behind them.
- [OWASP Top 10 — Cryptographic Failures](https://owasp.org/Top10/A02_2021-Cryptographic_Failures/): the category this lesson's failure mode belongs to.
- [Password Hashing Competition](https://www.password-hashing.net/): where Argon2 won in 2015.
- [`argon2` crate documentation](https://docs.rs/argon2/0.5.3/argon2/): `Argon2`, `Params`, and the re-exported `password_hash` types used here.
- [Django — Password management](https://docs.djangoproject.com/en/stable/topics/auth/passwords/): `make_password`, `check_password`, `PASSWORD_HASHERS` and how Django upgrades old hashes.

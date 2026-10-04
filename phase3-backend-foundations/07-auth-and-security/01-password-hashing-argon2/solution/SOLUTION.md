# Solution — 3.7.1 Password hashing with `argon2`

The full, tested code is in `src/lib.rs` next to this file; `cargo test` in this directory runs all 19 tests. Here is why each piece looks the way it does.

## `hash_password`

```rust
pub fn hash_password(password: &str, params: &Params) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::from(params.clone());
    argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("hashing an in-memory password with valid params cannot fail")
        .to_string()
}
```

`Argon2::from(params)` gives an `Argon2id`, version `0x13` hasher with your cost. A fresh `SaltString` each call is what makes the same password hash differently. `.expect(...)` is honest here: the input is an in-memory `&str` and the `Params` were already validated when they were built, so there is no recoverable failure left. Compare that with `verify_password`, where the stored string comes from outside the function.

## `verify_password`

```rust
pub fn verify_password(password: &str, phc: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(phc) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}
```

`Argon2::default()` is fine as the verifier because the algorithm, version, cost and salt are read from `parsed`; the defaults only fill in what the string does not say. The return type is `bool` on purpose: a corrupt row and a wrong password lead to the same decision, "reject", so the caller has nothing to do with the difference. This is the fix for example `06`. The comparison inside is constant-time (the `password-hash` crate's `Output` compares with `subtle`).

## `describe_hash` and `needs_rehash`

```rust
let parsed = PasswordHash::new(phc).ok()?;
let params = Params::try_from(&parsed).ok()?;
Some(HashInfo {
    algorithm: parsed.algorithm.to_string(),
    version: parsed.version?,
    memory_kib: params.m_cost(),
    /* iterations, lanes, salt, hash_len the same way */
})
```

`?` works on `Option` here, so a string with no version, salt or hash simply gives `None`. `needs_rehash` is then three comparisons plus "can't describe it, or not argon2id, means yes". It uses `<`, not `!=`: a hash that is already stronger than the target is fine, and downgrading it would be a bug.

## `constant_time_eq`

```rust
if a.len() != b.len() {
    return false;
}
let mut diff = 0u8;
for (x, y) in a.iter().zip(b) {
    diff |= x ^ y;
}
diff == 0
```

The length check may exit early: the length of a hash or token is not secret. After that, every pair is XORed and ORed into one accumulator, so the loop does the same work whether the first byte or none differs. This lesson's version is for learning; production code uses the `subtle` crate, which also stops the compiler from optimising the pattern back into an early exit.

## `UserStore`

```rust
pub fn login(&mut self, username: &str, password: &str) -> Result<(), AuthError> {
    let Some(stored) = self.users.get(username) else {
        verify_password(password, &self.dummy_hash);
        return Err(AuthError::InvalidCredentials);
    };
    if !verify_password(password, stored) {
        return Err(AuthError::InvalidCredentials);
    }
    // challenge: upgrade a weak stored hash here
    Ok(())
}
```

The unknown-user branch does one real verification and throws the answer away, so both failures return `InvalidCredentials` in about the same time. The dummy hash is built once in `new` under the store's own params, so its cost matches a real row. `register` checks the length first (`chars().count()`, not `len()`, because Persian letters are two bytes each), then the name, and stores only `hash_password(...)`.

The challenge version in `src/lib.rs` adds, after the successful check, a `needs_rehash(stored, &self.params)` test and, if it is true, replaces the row with a fresh hash of the plaintext you are holding at that moment. A failed login returns earlier, so it can never upgrade anything.

## On the "Can you explain?" questions

- **Hash, not encrypt:** login only asks "does it match?", so you never need the plaintext back. Encryption needs a key, and the key is one more secret whose theft exposes every password at once.
- **Salt:** random per hash, so equal passwords differ and a precomputed table would need rebuilding per row. It is not secret, which is why it is stored in the string.
- **`SHA-256` vs `bcrypt` vs `Argon2id`:** fast, slow with little memory, slow and memory-hard. RAM does not parallelise cheaply on a GPU or ASIC.
- **Unknown user:** the same error text is not enough; the same work is needed too, or the clock gives it away.
- **`==` on secrets:** the time depends on how many leading bytes matched. Constant time looks at every byte.

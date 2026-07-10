//! Never store a plaintext password. See `README.md` for the theory (salts,
//! rainbow tables, why Argon2 specifically) before touching the `todo!()`s
//! below — this lesson is pure computation, no infrastructure needed at
//! all: `cargo test -p p3-06-01-password-hashing-argon2` should just work.

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;

/// Hashes `password` with a fresh, random salt using Argon2 (with argon2's
/// default algorithm/parameters — Argon2id, a sensible general-purpose
/// choice) and returns the encoded PHC string
/// (`$argon2id$v=19$m=...,t=...,p=...$<salt>$<hash>`) — safe to store
/// directly in a `password_hash` column. Never store the raw `password`
/// argument anywhere; this function's whole job is to make sure you never
/// have to.
pub fn hash_password(password: &str) -> String {
    todo!(
        "generate a random salt with SaltString::generate(&mut OsRng); construct an \
         Argon2::default(); call .hash_password(password.as_bytes(), &salt), which returns a \
         Result<PasswordHash, argon2::password_hash::Error> — .unwrap() it (hashing a valid \
         in-memory password should never realistically fail) and call .to_string() on the \
         resulting PasswordHash to get the encoded PHC string to return"
    )
}

/// Verifies `password` against a previously-hashed PHC string produced by
/// `hash_password`. Returns `false` both for a wrong password AND for a
/// malformed/corrupt hash string — a caller checking a login never needs to
/// tell those two cases apart, and treating a parse failure as "reject the
/// login" rather than panicking is what keeps a malformed value in the
/// database from taking your auth endpoint down.
pub fn verify_password(password: &str, hash: &str) -> bool {
    todo!(
        "parse `hash` with PasswordHash::new(hash), which returns a Result — use a `let-else` \
         (let Ok(parsed) = PasswordHash::new(hash) else to return false early) or a match, \
         rather than .unwrap(), so a malformed hash returns false instead of panicking; then call \
         Argon2::default().verify_password(password.as_bytes(), &parsed_hash) and return true if \
         that's Ok(()), false if it's Err(_) — .is_ok() does exactly that in one call"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_then_verify_with_the_correct_password_succeeds() {
        let hash = hash_password("correct horse battery staple");
        assert!(verify_password("correct horse battery staple", &hash));
    }

    #[test]
    fn verify_with_the_wrong_password_fails() {
        let hash = hash_password("correct horse battery staple");
        assert!(!verify_password("wrong password", &hash));
    }

    /// The whole point of a random salt: hashing the same password twice
    /// produces two *different* encoded hashes, so an attacker with a
    /// precomputed table of common-password hashes gets no shortcut, and
    /// two users who happen to share a password don't end up with
    /// identical rows in your database either.
    #[test]
    fn hashing_the_same_password_twice_produces_different_hashes() {
        let first = hash_password("correct horse battery staple");
        let second = hash_password("correct horse battery staple");

        assert_ne!(first, second, "each call should use a fresh random salt");
        assert!(verify_password("correct horse battery staple", &first));
        assert!(verify_password("correct horse battery staple", &second));
    }

    #[test]
    fn verifying_against_a_malformed_hash_fails_instead_of_panicking() {
        assert!(!verify_password("anything", "not-a-real-phc-hash-string"));
    }

    #[test]
    fn empty_password_still_round_trips() {
        let hash = hash_password("");
        assert!(verify_password("", &hash));
        assert!(!verify_password("not empty", &hash));
    }
}

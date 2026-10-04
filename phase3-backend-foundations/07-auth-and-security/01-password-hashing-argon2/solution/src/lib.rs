//! 3.7.1 — Password hashing with `argon2`.
//!
//! Everything here is pure computation: no database, no server. Costs are
//! passed in as [`Params`] so the tests can use [`cheap_params`] and stay fast.

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::{Argon2, Params};
use std::collections::HashMap;

/// Deliberately weak cost settings (8 KiB of memory, 1 pass, 1 lane) for tests
/// and examples. Never use these for real users.
pub fn cheap_params() -> Params {
    Params::new(8, 1, 1, None).expect("8 KiB / 1 / 1 is a valid Argon2 configuration")
}

/// Hash `password` with Argon2id (version 0x13) using the cost settings in
/// `params` and a fresh random salt each call. Return the encoded PHC string
/// (`$argon2id$v=19$m=..,t=..,p=..$<salt>$<hash>`), which is safe to store as-is.
/// The empty password is allowed. Never fails for valid `params`.
pub fn hash_password(password: &str, params: &Params) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::from(params.clone());
    argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("hashing an in-memory password with valid params cannot fail")
        .to_string()
}

/// Check `password` against a stored PHC string. The salt and cost settings are
/// read from `phc` itself. Return `true` only when the password matches; return
/// `false` for a wrong password AND for a `phc` that is not a valid PHC string
/// (never panic on bad stored data).
pub fn verify_password(password: &str, phc: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(phc) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

/// What a stored hash says about itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashInfo {
    /// Algorithm identifier, e.g. `"argon2id"`.
    pub algorithm: String,
    /// Argon2 version number as printed after `v=`, e.g. `19`.
    pub version: u32,
    /// The `m` parameter, in KiB.
    pub memory_kib: u32,
    /// The `t` parameter (passes over memory).
    pub iterations: u32,
    /// The `p` parameter (lanes).
    pub lanes: u32,
    /// The salt exactly as it appears in the string (base64 text).
    pub salt: String,
    /// Length of the hash output in bytes (after decoding), e.g. `32`.
    pub hash_len: usize,
}

/// Read a PHC string into a [`HashInfo`]. Return `None` when `phc` does not
/// parse, or when it has no version, salt or hash field, or its parameters are
/// not valid Argon2 parameters.
pub fn describe_hash(phc: &str) -> Option<HashInfo> {
    let parsed = PasswordHash::new(phc).ok()?;
    let params = Params::try_from(&parsed).ok()?;
    Some(HashInfo {
        algorithm: parsed.algorithm.to_string(),
        version: parsed.version?,
        memory_kib: params.m_cost(),
        iterations: params.t_cost(),
        lanes: params.p_cost(),
        salt: parsed.salt?.to_string(),
        hash_len: parsed.hash?.len(),
    })
}

/// Should this stored hash be replaced with a fresh one made under `target`?
/// Return `true` when `phc` cannot be described (see [`describe_hash`]), when
/// its algorithm is not `"argon2id"`, or when its memory, iterations or lanes
/// is LOWER than the same setting in `target`. Otherwise `false` (equal or
/// stronger settings are fine).
pub fn needs_rehash(phc: &str, target: &Params) -> bool {
    let Some(info) = describe_hash(phc) else {
        return true;
    };
    info.algorithm != "argon2id"
        || info.memory_kib < target.m_cost()
        || info.iterations < target.t_cost()
        || info.lanes < target.p_cost()
}

/// Compare two byte slices for equality. Return `false` when the lengths
/// differ; when they are equal, look at EVERY byte (no early exit on the first
/// difference) and return `true` only if all bytes match.
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Why a register or login call failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthError {
    /// Login failed. Used for BOTH an unknown username and a wrong password.
    InvalidCredentials,
    /// `register` was called with a username that already exists.
    UsernameTaken,
    /// `register` was called with a password shorter than 8 characters.
    PasswordTooShort,
}

/// An in-memory user table: username to PHC hash. Never stores a password.
pub struct UserStore {
    params: Params,
    users: HashMap<String, String>,
    dummy_hash: String,
}

impl UserStore {
    /// An empty store whose new hashes use `params`. Also prepare a dummy hash
    /// (of any password) under `params`, for [`UserStore::login`].
    pub fn new(params: Params) -> Self {
        let dummy_hash = hash_password("dummy password, never matches", &params);
        Self {
            params,
            users: HashMap::new(),
            dummy_hash,
        }
    }

    /// Change the cost settings used for NEW hashes. Existing rows are untouched.
    pub fn set_params(&mut self, params: Params) {
        self.params = params;
    }

    /// Add a user. Errors, checked in this order: `PasswordTooShort` if
    /// `password` has fewer than 8 characters (count `char`s, not bytes);
    /// `UsernameTaken` if `username` exists (exact match). On success store
    /// only the PHC hash of the password under the store's current params.
    pub fn register(&mut self, username: &str, password: &str) -> Result<(), AuthError> {
        if password.chars().count() < 8 {
            return Err(AuthError::PasswordTooShort);
        }
        if self.users.contains_key(username) {
            return Err(AuthError::UsernameTaken);
        }
        let phc = hash_password(password, &self.params);
        self.users.insert(username.to_string(), phc);
        Ok(())
    }

    /// Check a login. `Ok(())` only for a known user with the right password.
    /// An unknown user and a wrong password must both return
    /// `Err(AuthError::InvalidCredentials)`, and an unknown user must still do
    /// one full password verification (against the dummy hash) so both paths
    /// cost about the same time.
    pub fn login(&mut self, username: &str, password: &str) -> Result<(), AuthError> {
        let Some(stored) = self.users.get(username) else {
            verify_password(password, &self.dummy_hash);
            return Err(AuthError::InvalidCredentials);
        };
        if !verify_password(password, stored) {
            return Err(AuthError::InvalidCredentials);
        }
        // Challenge: upgrade the stored hash when it is weaker than the current params.
        if needs_rehash(stored, &self.params) {
            let fresh = hash_password(password, &self.params);
            self.users.insert(username.to_string(), fresh);
        }
        Ok(())
    }

    /// The stored PHC string for `username`, or `None` if there is no such user.
    pub fn stored_hash(&self, username: &str) -> Option<&str> {
        self.users.get(username).map(String::as_str)
    }
}

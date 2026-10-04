//! Refresh-token rotation, reuse detection and revocation, with an
//! in-memory store and an injectable clock. See `README.md`.
//!
//! Note: `sha2::{Digest, Sha256}` is deliberately NOT imported yet, because only
//! `hash_token`'s body needs it. Add the `use` yourself when you fill it in.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use uuid::Uuid;

/// Lifetime of an access token, in seconds (15 minutes).
pub const ACCESS_TTL: u64 = 900;
/// Lifetime of a refresh token, in seconds (7 days).
pub const REFRESH_TTL: u64 = 7 * 24 * 3600;

/// A source of "now" as Unix seconds. Injected so tests never sleep.
pub trait Clock {
    fn now(&self) -> u64;
}

/// The real clock: seconds since the Unix epoch.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is before 1970")
            .as_secs()
    }
}

/// A clock a test moves by hand. Clones share one counter, so a test can keep
/// a clone and advance time after handing the original to a service.
#[derive(Clone)]
pub struct ManualClock(Arc<AtomicU64>);

impl ManualClock {
    pub fn new(start: u64) -> Self {
        ManualClock(Arc::new(AtomicU64::new(start)))
    }

    pub fn advance(&self, seconds: u64) {
        self.0.fetch_add(seconds, Ordering::SeqCst);
    }
}

impl Clock for ManualClock {
    fn now(&self) -> u64 {
        self.0.load(Ordering::SeqCst)
    }
}

/// What a successful login or refresh hands back to the client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
}

/// Why a refresh was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshError {
    Unknown,
    Expired,
    ReuseDetected,
}

/// What the server remembers about one refresh token. Keyed by the token's
/// hash in the store; the raw token itself is never stored.
#[derive(Debug, Clone)]
struct Record {
    user_id: String,
    family_id: String,
    expires_at: u64,
    used: bool,
}

/// Lowercase hexadecimal SHA-256 of the UTF-8 bytes of `raw`: always exactly
/// 64 characters, each one of `0-9a-f`. Deterministic: the same input always
/// gives the same output. For example `hash_token("abc")` is
/// `"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"`.
pub fn hash_token(raw: &str) -> String {
    todo!("return the SHA-256 digest of `raw`, written as 64 lowercase hex characters")
}

/// A fresh random token: 64 lowercase hex characters built from two v4 UUIDs.
fn generate_token() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

/// Issues refresh tokens, rotates them, and revokes them. In-memory only.
pub struct RefreshService<C: Clock> {
    clock: C,
    records: HashMap<String, Record>,
}

impl<C: Clock> RefreshService<C> {
    pub fn new(clock: C) -> Self {
        RefreshService {
            clock,
            records: HashMap::new(),
        }
    }

    /// Starts a new token family for `user_id` and returns its first pair.
    pub fn login(&mut self, user_id: &str) -> TokenPair {
        let family_id = Uuid::new_v4().to_string();
        self.issue(user_id, &family_id)
    }

    /// Mints a pair in `family_id` and stores the hash of its refresh token.
    /// The access token is a stand-in for the signed JWT of 3.7.3: the text
    /// `access.<user_id>.<expires_at>` where `expires_at` is now + `ACCESS_TTL`.
    fn issue(&mut self, user_id: &str, family_id: &str) -> TokenPair {
        let now = self.clock.now();
        let raw = generate_token();
        self.records.insert(
            hash_token(&raw),
            Record {
                user_id: user_id.to_string(),
                family_id: family_id.to_string(),
                expires_at: now + REFRESH_TTL,
                used: false,
            },
        );
        TokenPair {
            access_token: format!("access.{}.{}", user_id, now + ACCESS_TTL),
            refresh_token: raw,
        }
    }

    /// Removes every stored token of `family_id`; returns how many were removed.
    fn revoke_family(&mut self, family_id: &str) -> usize {
        let before = self.records.len();
        self.records.retain(|_, r| r.family_id != family_id);
        before - self.records.len()
    }

    /// How many refresh-token records are stored.
    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    /// Every stored key: what a database leak would expose.
    pub fn stored_hashes(&self) -> Vec<String> {
        self.records.keys().cloned().collect()
    }

    /// Exchanges the refresh token `presented` for a new pair.
    ///
    /// The token is looked up by `hash_token(presented)`. The outcomes, checked
    /// in exactly this order:
    ///
    /// 1. No record has that hash: `Err(RefreshError::Unknown)`. Nothing changes.
    /// 2. The record is already marked used: someone is replaying a rotated-out
    ///    token. Remove every record of that token's family and return
    ///    `Err(RefreshError::ReuseDetected)`. This check comes before expiry.
    /// 3. The record's `expires_at` is less than or equal to the clock's now:
    ///    `Err(RefreshError::Expired)`. Nothing changes; the family stays.
    /// 4. Otherwise mark the record used and return `Ok` with a new pair for
    ///    the same user and the same family: a brand-new refresh token whose
    ///    own expiry is now + `REFRESH_TTL`, and an access token as described
    ///    on `login`.
    pub fn rotate(&mut self, presented: &str) -> Result<TokenPair, RefreshError> {
        todo!("exchange the presented refresh token for a new pair, following the four outcomes above")
    }

    /// Logs out the session `presented` belongs to: removes every record of its
    /// family, whether or not that token was already used or has expired.
    /// Returns `true` if a record with that hash existed, `false` otherwise
    /// (and then changes nothing).
    pub fn logout(&mut self, presented: &str) -> bool {
        todo!(
            "revoke the family the presented token belongs to, and say whether the token was known"
        )
    }

    /// Logs `user_id` out everywhere: removes every record of every family that
    /// belongs to that user and returns how many distinct families were
    /// removed (0 if the user has none). Other users are untouched.
    pub fn logout_all(&mut self, user_id: &str) -> usize {
        todo!("revoke every family of this user and return how many families that was")
    }
}

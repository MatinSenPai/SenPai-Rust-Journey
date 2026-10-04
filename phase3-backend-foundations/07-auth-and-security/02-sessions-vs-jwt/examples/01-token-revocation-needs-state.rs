//! A stateless token is checked by looking only at the token. That is its
//! whole appeal, and it is why "log out" cannot work without adding state
//! back. The tokens here are FAKE: "user.expires_at" with a made-up check
//! value standing in for a signature. 3.7.3 does the real thing.
//!
//!     cargo run -p p3-07-02-sessions-vs-jwt --example 01-token-revocation-needs-state

use std::collections::HashSet;

const SECRET: u64 = 0xC0FFEE; // pretend signing key

fn fake_sign(user: &str, exp: u64) -> u64 {
    user.bytes()
        .fold(SECRET ^ exp, |h, b| h.wrapping_mul(31) ^ b as u64)
}

fn issue(user: &str, exp: u64) -> String {
    format!("{user}.{exp}.{:x}", fake_sign(user, exp))
}

/// Stateless check: no lookup, only the token and the clock.
fn verify(token: &str, now: u64) -> Option<String> {
    let mut parts = token.split('.');
    let (user, exp, sig) = (parts.next()?, parts.next()?, parts.next()?);
    let exp: u64 = exp.parse().ok()?;
    let ok = u64::from_str_radix(sig, 16).ok()? == fake_sign(user, exp);
    (ok && now < exp).then(|| user.to_string())
}

fn main() {
    let token = issue("matin", 1_000);
    println!("token        : {token}");
    println!("verify @ 100 : {:?}", verify(&token, 100));

    // "Logout" with nothing but the token: there is nothing to delete.
    // The server kept no record, so the stolen copy still works.
    println!(
        "after logout : {:?}  (a stolen copy still works)",
        verify(&token, 200)
    );

    // The fix is a deny-list: a set of revoked tokens the server must
    // remember and check on EVERY request. That is state again.
    let mut denied: HashSet<String> = HashSet::new();
    denied.insert(token.clone());
    let checked = |t: &str, now| {
        if denied.contains(t) {
            None
        } else {
            verify(t, now)
        }
    };
    println!("with deny-list: {:?}", checked(&token, 300));
    println!("deny-list entries to keep until expiry: {}", denied.len());
}

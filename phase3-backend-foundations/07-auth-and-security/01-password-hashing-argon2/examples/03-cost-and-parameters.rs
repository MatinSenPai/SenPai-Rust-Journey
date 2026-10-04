//! Same password, two cost settings; then verify the cheap hash with a default hasher.
//! Run: `cargo run -p p3-07-01-password-hashing-argon2 --example 03-cost-and-parameters`

use argon2::password_hash::{
    rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
};
use argon2::{Argon2, Params};
use std::time::Instant;

fn hash_with(m: u32, t: u32) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::from(Params::new(m, t, 1, None).expect("bad params"));
    let hash = argon2
        .hash_password(b"hunter2", &salt)
        .expect("hash failed");
    hash.to_string()
}

fn main() {
    for (m, t) in [(8, 1), (19456, 2), (65536, 3)] {
        let start = Instant::now();
        let phc = hash_with(m, t);
        println!(
            "m={m:<6} t={t}  took {:>4} ms  {}",
            start.elapsed().as_millis(),
            &phc[..32]
        );
    }
    let old = hash_with(8, 1);
    let parsed = PasswordHash::new(&old).unwrap();
    let verdict = Argon2::default().verify_password(b"hunter2", &parsed);
    println!("cheap hash, default hasher: {verdict:?}");
}

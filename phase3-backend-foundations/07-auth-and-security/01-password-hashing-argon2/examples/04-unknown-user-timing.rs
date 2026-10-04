//! Why "user not found" must still do the expensive work.
//! Run: `cargo run -p p3-07-01-password-hashing-argon2 --example 04-unknown-user-timing`

use argon2::password_hash::{
    rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
};
use argon2::Argon2;
use std::time::Instant;

fn verify(password: &str, phc: &str) -> bool {
    let parsed = PasswordHash::new(phc).unwrap();
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

fn main() {
    let salt = SaltString::generate(&mut OsRng);
    let stored = Argon2::default()
        .hash_password(b"hunter2", &salt)
        .unwrap()
        .to_string();

    let start = Instant::now();
    let _ = verify("wrong guess", &stored);
    println!(
        "known user, wrong password : {:>4} ms",
        start.elapsed().as_millis()
    );

    let start = Instant::now();
    let _ = None::<&str>.is_some_and(|phc| verify("wrong guess", phc));
    println!(
        "unknown user, early return : {:>4} ms",
        start.elapsed().as_millis()
    );

    let start = Instant::now();
    let _ = verify("wrong guess", &stored);
    println!(
        "unknown user, dummy verify : {:>4} ms",
        start.elapsed().as_millis()
    );
}

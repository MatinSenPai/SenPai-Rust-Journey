//! Hash a password twice, look at both results, then verify.
//! Run: `cargo run -p p3-07-01-password-hashing-argon2 --example 01-hash-and-verify`

use argon2::password_hash::{
    rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
};
use argon2::Argon2;

fn hash(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default().hash_password(password.as_bytes(), &salt);
    hash.expect("hashing failed").to_string()
}

fn main() {
    let first = hash("hunter2");
    let second = hash("hunter2");
    println!("first : {first}");
    println!("second: {second}");
    println!("equal strings? {}", first == second);

    let parsed = PasswordHash::new(&first).expect("not a PHC string");
    let argon2 = Argon2::default();
    println!(
        "right password: {:?}",
        argon2.verify_password(b"hunter2", &parsed)
    );
    println!(
        "wrong password: {:?}",
        argon2.verify_password(b"hunter3", &parsed)
    );
}

//! Compiles and runs, and is wrong: login by hashing again and comparing strings.
//! Run: `cargo run -p p3-07-01-password-hashing-argon2 --example 07-compare-hashes-trap`

use argon2::password_hash::{rand_core::OsRng, PasswordHasher, SaltString};
use argon2::Argon2;

fn hash(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default().hash_password(password.as_bytes(), &salt);
    hash.unwrap().to_string()
}

fn main() {
    let stored = hash("hunter2");
    let login_ok = hash("hunter2") == stored;
    println!("correct password accepted? {login_ok}");
}

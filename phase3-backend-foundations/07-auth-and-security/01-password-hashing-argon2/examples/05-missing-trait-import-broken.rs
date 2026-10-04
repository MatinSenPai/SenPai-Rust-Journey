//! DELIBERATELY BROKEN — expected: E0599
//! Run `cargo build -p p3-07-01-password-hashing-argon2 --example 05-missing-trait-import-broken --features broken`
//! and read the error.

use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::Argon2;

fn main() {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default().hash_password(b"hunter2", &salt).unwrap();
    println!("{hash}");
}

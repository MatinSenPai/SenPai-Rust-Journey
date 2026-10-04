//! DELIBERATELY BROKEN — expected: a run-time panic, `PasswordHash::new` on a value that is not a PHC string.
//! Run `cargo run -p p3-07-01-password-hashing-argon2 --example 06-unwrap-malformed-hash-broken --features broken`.

use argon2::password_hash::PasswordHash;

fn main() {
    let from_database = "not-a-real-phc-hash-string";
    let parsed = PasswordHash::new(from_database).unwrap();
    println!("{parsed}");
}

//! DELIBERATELY BROKEN — expected: E0277 (`SecretString` has no `Display`).
//!
//! Trying to print a secret directly, as if it were a `String`.
//!
//!     cargo run -p p3-04-01-config-and-secrets --example 06-print-secret-broken --features broken

use secrecy::SecretString;

fn main() {
    let password = SecretString::from("hunter2");
    println!("connecting with password {}", password);
}

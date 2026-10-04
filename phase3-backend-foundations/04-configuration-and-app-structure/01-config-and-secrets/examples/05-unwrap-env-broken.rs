//! DELIBERATELY BROKEN — expected: a run-time panic at startup.
//!
//! The "just unwrap it" way to read a setting: a missing variable becomes a
//! panic with a message about `VarError`, not about what the operator should
//! do.
//!
//!     cargo run -p p3-04-01-config-and-secrets --example 05-unwrap-env-broken --features broken

fn main() {
    let port: u16 = std::env::var("APP_PORT").unwrap().parse().unwrap();
    println!("listening on port {port}");
}

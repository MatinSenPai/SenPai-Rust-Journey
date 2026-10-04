//! The same settings struct twice: once with a plain `String` password, once
//! with `SecretString`. Both derive `Debug`, and that is all it takes to leak.
//!
//!     cargo run -p p3-04-01-config-and-secrets --example 01-debug-leak

use secrecy::{ExposeSecret, SecretString};

// The fields are only ever read by `Debug`, which dead-code analysis ignores.
#[allow(dead_code)]
#[derive(Debug)]
struct Leaky {
    port: u16,
    db_password: String,
}

#[allow(dead_code)]
#[derive(Debug)]
struct Careful {
    port: u16,
    db_password: SecretString,
}

fn main() {
    let leaky = Leaky {
        port: 8080,
        db_password: "hunter2".to_string(),
    };
    let careful = Careful {
        port: 8080,
        db_password: SecretString::from("hunter2"),
    };

    println!("{leaky:?}");
    println!("{careful:?}");
    println!("on purpose: {}", careful.db_password.expose_secret());
}

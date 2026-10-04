//! The whole pipeline on in-memory inputs: a file, a made-up environment, and
//! the config that comes out. Needs `Layer::from_env`, `over`, `finish` and
//! `Config::load` from the exercises.
//!
//!     cargo run -p p3-04-01-config-and-secrets --example 03-load-layers

use std::collections::HashMap;

use p3_04_01_config_and_secrets::Config;

fn env(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

fn main() {
    let file = "port = 7000\ndebug = true\ndb_password = \"from-the-file\"\n";

    println!("{:?}", Config::load(Some(file), &env(&[])));
    println!(
        "{:?}",
        Config::load(Some(file), &env(&[("APP_PORT", "9000")]))
    );
    println!("{:?}", Config::load(None, &env(&[])));
    println!("{:?}", Config::load(None, &env(&[("APP_PORT", "eighty")])));
    println!("{}", Config::load(None, &env(&[])).unwrap_err());
}

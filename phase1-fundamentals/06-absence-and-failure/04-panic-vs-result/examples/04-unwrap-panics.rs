//! DELIBERATELY BROKEN — expected: a panic at run time (this one compiles)
//!
//! A bare `.unwrap()`. It panics with the same generic message you first
//! saw in 1.6.1, no matter which of a hundred `None`s produced it.
//!
//!     cargo run -p p1-06-04-panic-vs-result --example 04-unwrap-panics --features broken

fn find_config_path<'a>(candidates: &[&'a str], wanted: &str) -> Option<&'a str> {
    candidates
        .iter()
        .find(|&&candidate| candidate == wanted)
        .copied()
}

fn main() {
    let candidates = ["dev.toml", "staging.toml"];
    println!("looking for prod.toml among {candidates:?}");
    let path = find_config_path(&candidates, "prod.toml").unwrap();
    println!("using config at {path}");
}

//! DELIBERATELY BROKEN — expected: a panic at run time (this one compiles)
//!
//! The exact same failure as 04-unwrap-panics — but `.expect()` carries a
//! message naming the assumption that broke, not a description of `None`.
//!
//!     cargo run -p p1-06-04-panic-vs-result --example 05-expect-with-a-good-message --features broken

fn find_config_path<'a>(candidates: &[&'a str], wanted: &str) -> Option<&'a str> {
    candidates
        .iter()
        .find(|&&candidate| candidate == wanted)
        .copied()
}

fn main() {
    let candidates = ["dev.toml", "staging.toml"];
    println!("looking for prod.toml among {candidates:?}");
    let path = find_config_path(&candidates, "prod.toml")
        .expect("prod.toml must be listed in `candidates` — deploy config is incomplete");
    println!("using config at {path}");
}

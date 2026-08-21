//! DELIBERATELY BROKEN — expected: a panic at run time (this one compiles)
//!
//! It builds fine. The table below is supposed to cover every region this
//! program deploys to — that's an invariant *this codebase* owns, not
//! something a caller typed. When the table and reality disagree, that's a
//! bug, so it panics instead of returning a `Result` nobody would expect.
//!
//!     cargo run -p p1-06-04-panic-vs-result --example 02-broken-invariant --features broken

fn region_code(name: &str) -> u8 {
    match name {
        "ir" => 98,
        "us" => 1,
        "de" => 49,
        other => panic!(
            "region_code: no dialing code registered for {other:?} — this table is supposed \
             to cover every region this program deploys to"
        ),
    }
}

fn main() {
    println!("ir -> {}", region_code("ir"));
    println!("us -> {}", region_code("us"));
    println!("fr -> {}", region_code("fr"));
}

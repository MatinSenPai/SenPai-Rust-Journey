//! HTTP header *names* are case-insensitive by spec — `Host` and `host` name
//! the same header. `str::eq_ignore_ascii_case` compares them without
//! allocating a lowercased copy of either side.
//!
//!     cargo run -p p3-01-02-hand-rolled-http-parser --example 03-case-insensitive-header-names

fn main() {
    let sent_by_client = "Host";
    for candidate in ["host", "HOST", "Host", "Content-Type"] {
        println!(
            "{candidate:?} matches {sent_by_client:?}: {}",
            candidate.eq_ignore_ascii_case(sent_by_client)
        );
    }
}

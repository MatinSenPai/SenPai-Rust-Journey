//! DELIBERATELY BROKEN — expected: E0004
//!
//! A Rust client matches every `WatchStatus` the server used to send. The
//! server adds `OnHold`; the client crate, rebuilt against the new type, no
//! longer covers every case. (A client in another language fails later, at
//! run time, on the first `"on_hold"` it sees.)
//!
//!     cargo build -p p3-03-04-api-versioning-and-evolution --example 05-new-enum-value-broken --features broken

#[allow(dead_code)]
enum WatchStatus {
    Watching,
    Completed,
    PlanToWatch,
    Dropped,
    OnHold, // added in a "minor" release
}

fn badge(s: &WatchStatus) -> &'static str {
    match s {
        WatchStatus::Watching => "playing",
        WatchStatus::Completed => "done",
        WatchStatus::PlanToWatch => "planned",
        WatchStatus::Dropped => "dropped",
    }
}

fn main() {
    println!("{}", badge(&WatchStatus::Watching));
}

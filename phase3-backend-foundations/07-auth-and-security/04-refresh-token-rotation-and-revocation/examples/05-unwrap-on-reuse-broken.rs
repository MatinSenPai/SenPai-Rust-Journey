//! DELIBERATELY BROKEN — expected: a run-time panic (`unwrap` on `ReuseDetected`)
//! Run `cargo run --example 05-unwrap-on-reuse-broken --features broken` and read the panic.

#[path = "common/store.rs"]
mod store;

use store::{ManualClock, RefreshService};

fn main() {
    let mut svc = RefreshService::new(ManualClock::new(1_700_000_000));
    let first = svc.login("matin");
    svc.rotate(&first.refresh_token).unwrap();
    // A retrying client sends the same refresh token twice.
    let again = svc.rotate(&first.refresh_token).unwrap();
    println!("{again:?}");
}

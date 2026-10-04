//! A thief replays a token the real client already rotated away.

#[path = "common/store.rs"]
mod store;

use store::{ManualClock, RefreshService};

fn main() {
    let mut svc = RefreshService::new(ManualClock::new(1_700_000_000));
    let stolen = svc.login("matin");
    let legit = svc.rotate(&stolen.refresh_token).unwrap();
    println!("client rotated, records: {}", svc.record_count());

    println!(
        "thief replays old token:  {:?}",
        svc.rotate(&stolen.refresh_token)
    );
    println!("records after the replay: {}", svc.record_count());
    println!(
        "client's newest token:    {:?}",
        svc.rotate(&legit.refresh_token)
    );
}

//! DELIBERATELY BROKEN — expected: E0382
//! Run `cargo build --example 04-token-moved-broken --features broken` and read the error.

#[path = "common/store.rs"]
mod store;

use store::{ManualClock, RefreshService};

fn save_to_cookie(token: String) {
    println!("set-cookie: refresh={}", &token[..8]);
}

fn main() {
    let mut svc = RefreshService::new(ManualClock::new(1_700_000_000));
    let pair = svc.login("matin");
    save_to_cookie(pair.refresh_token);
    let next = svc.rotate(&pair.refresh_token);
    println!("{next:?}");
}

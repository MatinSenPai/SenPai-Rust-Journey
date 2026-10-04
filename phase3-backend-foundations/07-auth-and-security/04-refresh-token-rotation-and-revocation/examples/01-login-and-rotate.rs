//! One login, then three rotations. Every refresh hands back a new token.

#[path = "common/store.rs"]
mod store;

use store::{hash_token, ManualClock, RefreshService};

fn main() {
    let clock = ManualClock::new(1_700_000_000);
    let mut svc = RefreshService::new(clock.clone());

    let mut pair = svc.login("matin");
    println!("login   access={}", pair.access_token);
    println!("        refresh={}", &pair.refresh_token[..16]);
    for step in 1..=3 {
        clock.advance(600);
        pair = svc
            .rotate(&pair.refresh_token)
            .expect("fresh token rotates");
        println!("rotate{step} access={}", pair.access_token);
        println!("        refresh={}", &pair.refresh_token[..16]);
    }
    println!("records stored: {}", svc.record_count());
    println!("hash of newest: {}", &hash_token(&pair.refresh_token)[..16]);
}

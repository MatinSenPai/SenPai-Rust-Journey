//! What an attacker holding a copy of the store sees: hashes, not tokens.

#[path = "common/store.rs"]
mod store;

use store::{hash_token, ManualClock, RefreshService};

fn main() {
    let mut svc = RefreshService::new(ManualClock::new(1_700_000_000));
    let pair = svc.login("matin");

    let leaked = svc.stored_hashes();
    println!("leaked rows:     {}", leaked.len());
    println!("leaked value:    {}", leaked[0]);
    println!("real token:      {}", pair.refresh_token);
    println!("leak == token?   {}", leaked[0] == pair.refresh_token);
    println!(
        "hash of token:   {}",
        hash_token(&pair.refresh_token) == leaked[0]
    );
    // The attacker can only try the hash as if it were a token:
    println!("replay the hash: {:?}", svc.rotate(&leaked[0]));
}

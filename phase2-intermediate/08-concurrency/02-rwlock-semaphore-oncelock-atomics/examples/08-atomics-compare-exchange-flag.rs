//! `compare_exchange` is the primitive every one-shot flag is built from:
//! "if it's still `false`, make it `true` — and tell me whether I was the one
//! who did it."

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

fn main() {
    let started = AtomicBool::new(false);
    let first = started.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst);
    let second = started.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst);
    println!("first attempt:  {first:?}");
    println!("second attempt: {second:?}");

    let claimed = Arc::new(AtomicBool::new(false));
    let winners = Arc::new(AtomicUsize::new(0));
    let handles: Vec<_> = (0..6)
        .map(|_| {
            let claimed = Arc::clone(&claimed);
            let winners = Arc::clone(&winners);
            thread::spawn(move || {
                let won = claimed
                    .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok();
                if won {
                    winners.fetch_add(1, Ordering::SeqCst);
                }
            })
        })
        .collect();
    for handle in handles {
        handle.join().unwrap();
    }
    println!(
        "threads that won the race to initialize: {}",
        winners.load(Ordering::SeqCst)
    );
}

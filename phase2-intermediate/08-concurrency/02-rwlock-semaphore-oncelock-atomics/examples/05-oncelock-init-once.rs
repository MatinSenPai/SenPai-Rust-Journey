//! `OnceLock::get_or_init` runs its closure at most once, no matter how many
//! threads race to be first — the rest just get the already-computed value.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;
use std::thread;

static INIT_COUNT: AtomicUsize = AtomicUsize::new(0);
static CONFIG: OnceLock<String> = OnceLock::new();

fn config() -> &'static str {
    CONFIG.get_or_init(|| {
        INIT_COUNT.fetch_add(1, Ordering::SeqCst);
        String::from("max_connections=64")
    })
}

fn main() {
    let handles: Vec<_> = (0..8).map(|_| thread::spawn(config)).collect();
    for handle in handles {
        handle.join().unwrap();
    }
    println!("config: {}", config());
    println!(
        "init closure ran {} time(s)",
        INIT_COUNT.load(Ordering::SeqCst)
    );
}

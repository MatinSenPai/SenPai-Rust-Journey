//! DELIBERATELY BROKEN — expected: E0277.
//!
//! Swapping `Rc` for `Arc` alone is not enough. `RefCell<i32>` still isn't
//! `Sync`, and `Arc<T>` is only `Send` when `T` is both `Send` *and* `Sync`
//! — so `Arc<RefCell<i32>>` still can't cross a thread boundary.
//!
//!     cargo run -p p2-08-04-send-and-sync --example 05-arc-refcell-not-sync-broken --features broken

use std::cell::RefCell;
use std::sync::Arc;
use std::thread;

fn main() {
    let shared = Arc::new(RefCell::new(0));
    let clone_for_thread = Arc::clone(&shared);

    let handle = thread::spawn(move || {
        *clone_for_thread.borrow_mut() += 1;
    });

    handle.join().unwrap();
}

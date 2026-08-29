//! DELIBERATELY BROKEN — expected: E0277.
//!
//! `Rc<RefCell<i32>>` moved into a spawned thread's closure — the classic
//! mistake this whole lesson explains.
//!
//!     cargo run -p p2-08-04-send-and-sync --example 04-rc-refcell-thread-spawn-broken --features broken

use std::cell::RefCell;
use std::rc::Rc;
use std::thread;

fn main() {
    let shared = Rc::new(RefCell::new(0));

    let handle = thread::spawn(move || {
        *shared.borrow_mut() += 1;
    });

    handle.join().unwrap();
}

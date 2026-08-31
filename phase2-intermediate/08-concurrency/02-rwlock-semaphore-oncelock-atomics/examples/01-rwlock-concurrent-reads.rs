//! `RwLock` lets any number of readers hold the lock at the same time — the
//! one real difference from `Mutex`, which never allows that.

use std::sync::RwLock;

fn main() {
    let value = RwLock::new(10);

    let r1 = value.read().unwrap();
    let r2 = value.read().unwrap(); // a second reader is fine, at the same time
    println!("two readers at once: {} and {}", *r1, *r2);

    let busy = value.try_write();
    println!("try_write while readers are alive: {}", busy.is_err());

    drop(r1);
    drop(r2);

    let mut w = value.write().unwrap();
    *w += 1;
    println!("after write: {w}");
}

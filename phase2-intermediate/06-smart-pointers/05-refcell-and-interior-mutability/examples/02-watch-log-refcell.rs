//! `RefCell<T>` — the same aliasing rule Phase 1 taught, checked at run time.

use std::cell::RefCell;

struct WatchLog {
    entries: RefCell<Vec<String>>,
}

impl WatchLog {
    fn log(&self, title: &str) {
        self.entries.borrow_mut().push(title.to_string());
    }
}

fn main() {
    let log = WatchLog {
        entries: RefCell::new(Vec::new()),
    };
    log.log("Frieren");
    log.log("Bocchi the Rock!");

    let peek = log.entries.borrow(); // Ref<Vec<String>>: a shared, read-only view
    println!("so far: {peek:?}");
    drop(peek); // must end before borrow_mut, or the next log() panics

    log.log("Made in Abyss");
    println!("now:    {:?}", log.entries.borrow());
}

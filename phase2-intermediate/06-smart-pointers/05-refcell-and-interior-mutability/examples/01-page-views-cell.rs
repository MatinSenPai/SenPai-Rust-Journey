//! `Cell<T>` — no borrowing, no guards, just copy in and copy out.

use std::cell::Cell;

struct PageViews {
    count: Cell<u32>,
}

impl PageViews {
    fn record_view(&self) {
        self.count.set(self.count.get() + 1);
    }
}

fn main() {
    let page = PageViews {
        count: Cell::new(0),
    };
    page.record_view();
    page.record_view();
    page.record_view();
    println!("views: {}", page.count.get());
}
